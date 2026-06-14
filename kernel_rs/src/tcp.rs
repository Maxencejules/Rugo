// Minimal wire TCP (gap-analysis item 6): a single outbound IPv4
// connection over the VirtIO NIC, driven by the PIT-tick RX pump.
//
// v1 contract: client-side handshake, PSH/ACK data both ways, FIN
// teardown, RST handling. One connection at a time; no retransmission
// (QEMU's user-mode network is loss-free) - documented carry-forward
// with window management and listeners.

#![allow(dead_code)]

use crate::net;
use crate::serial_write;

pub(crate) const ST_CLOSED: u8 = 0;
pub(crate) const ST_ARP_WAIT: u8 = 1;
pub(crate) const ST_SYN_SENT: u8 = 2;
pub(crate) const ST_ESTABLISHED: u8 = 3;
pub(crate) const ST_FIN_WAIT: u8 = 4;
pub(crate) const ST_LISTEN: u8 = 5;
pub(crate) const ST_SYN_RCVD: u8 = 6;

const RX_RING: usize = 1024;
const GUEST_IP: [u8; 4] = [10, 0, 2, 15];

// Retransmission / RTO (full-os guide Part II.6): the single outstanding segment
// is held until its sequence span is acknowledged. The timer is a tick
// countdown (decremented once per PIT tick by tcp_rt_tick), so it needs no wall
// clock and stays deterministic in the self-test. PIT is 100 Hz, so 50 ticks is
// a ~500 ms initial RTO; it backs off exponentially and gives up after MAX.
const TCP_RTO_TICKS: u32 = 50;
const TCP_MAX_RETRIES: u32 = 5;
const RT_DATA_MAX: usize = 512;

struct TcpConn {
    state: u8,
    peer_ip: [u8; 4],
    peer_mac: [u8; 6],
    have_mac: bool,
    local_port: u16,
    remote_port: u16,
    snd_nxt: u32,
    snd_una: u32,
    rcv_nxt: u32,
    peer_fin: bool,
    rx_len: usize,
    rx: [u8; RX_RING],
    // Retransmit slot: the oldest unacknowledged segment.
    rt_active: bool,
    rt_flags: u8,
    rt_seq: u32,
    rt_len: usize,
    rt_data: [u8; RT_DATA_MAX],
    rt_ticks_left: u32,
    rt_retries: u32,
    rt_last_send_ok: bool, // did the most recent retransmit actually emit?
    pending_close: bool,   // a close() deferred because data was outstanding
}

static mut CONN: TcpConn = TcpConn {
    state: ST_CLOSED,
    peer_ip: [0; 4],
    peer_mac: [0; 6],
    have_mac: false,
    local_port: 0,
    remote_port: 0,
    snd_nxt: 0,
    snd_una: 0,
    rcv_nxt: 0,
    peer_fin: false,
    rx_len: 0,
    rx: [0; RX_RING],
    rt_active: false,
    rt_flags: 0,
    rt_seq: 0,
    rt_len: 0,
    rt_data: [0; RT_DATA_MAX],
    rt_ticks_left: 0,
    rt_retries: 0,
    rt_last_send_ok: false,
    pending_close: false,
};

pub(crate) unsafe fn tcp_state() -> u8 {
    CONN.state
}

pub(crate) unsafe fn tcp_active() -> bool {
    CONN.state != ST_CLOSED
}

fn csum_words(sum: &mut u32, data: &[u8]) {
    let mut i = 0;
    while i + 1 < data.len() {
        *sum += u16::from_be_bytes([data[i], data[i + 1]]) as u32;
        i += 2;
    }
    if i < data.len() {
        *sum += (data[i] as u32) << 8;
    }
}

fn csum_fold(mut sum: u32) -> u16 {
    while sum > 0xFFFF {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}

/// Compose and transmit one TCP segment with the given flags + payload.
unsafe fn tcp_tx(flags: u8, seq: u32, ack: u32, payload: &[u8]) -> bool {
    let tcp_len = 20 + payload.len();
    let ip_len = 20 + tcp_len;
    let total = 14 + ip_len;
    if total > 1514 {
        return false;
    }
    let mut f = [0u8; 1514];
    f[0..6].copy_from_slice(&CONN.peer_mac);
    f[6..12].copy_from_slice(&net::net_mac());
    f[12] = 0x08;
    f[13] = 0x00;

    let ip = &mut f[14..];
    ip[0] = 0x45;
    ip[2] = (ip_len >> 8) as u8;
    ip[3] = (ip_len & 0xFF) as u8;
    ip[5] = 0x01; // id
    ip[8] = 64; // ttl
    ip[9] = 6; // proto TCP
    ip[12..16].copy_from_slice(&GUEST_IP);
    ip[16..20].copy_from_slice(&CONN.peer_ip);
    let mut s = 0u32;
    csum_words(&mut s, &ip[..20]);
    let c = csum_fold(s);
    ip[10] = (c >> 8) as u8;
    ip[11] = (c & 0xFF) as u8;

    let t = &mut ip[20..];
    t[0..2].copy_from_slice(&CONN.local_port.to_be_bytes());
    t[2..4].copy_from_slice(&CONN.remote_port.to_be_bytes());
    t[4..8].copy_from_slice(&seq.to_be_bytes());
    t[8..12].copy_from_slice(&ack.to_be_bytes());
    t[12] = 5 << 4; // data offset
    t[13] = flags;
    t[14] = 0x10; // window 4096
    t[15] = 0x00;
    t[20..20 + payload.len()].copy_from_slice(payload);

    // Pseudo-header checksum.
    let mut s = 0u32;
    csum_words(&mut s, &GUEST_IP);
    csum_words(&mut s, &CONN.peer_ip);
    s += 6;
    s += tcp_len as u32;
    csum_words(&mut s, &t[..tcp_len]);
    let c = csum_fold(s);
    t[16] = (c >> 8) as u8;
    t[17] = (c & 0xFF) as u8;

    net::wire_send(&f[..total])
}

/// Number of sequence numbers a segment with these flags + payload consumes
/// (payload bytes, plus one each for SYN / FIN).
fn seq_span(flags: u8, payload_len: usize) -> u32 {
    let mut span = payload_len as u32;
    if flags & 0x02 != 0 {
        span += 1; // SYN
    }
    if flags & 0x01 != 0 {
        span += 1; // FIN
    }
    span
}

/// Arm the retransmit timer for a just-sent segment (the single outstanding
/// segment in this v1 connection). Records the bytes so it can be re-sent
/// verbatim, sets snd_una to its sequence, and starts the RTO countdown.
unsafe fn tcp_rt_arm(flags: u8, seq: u32, payload: &[u8]) {
    let n = payload.len().min(RT_DATA_MAX);
    CONN.snd_una = seq;
    CONN.rt_active = true;
    CONN.rt_flags = flags;
    CONN.rt_seq = seq;
    CONN.rt_len = n;
    CONN.rt_data[..n].copy_from_slice(&payload[..n]);
    CONN.rt_ticks_left = TCP_RTO_TICKS;
    CONN.rt_retries = 0;
}

/// Clear the retransmit timer once the peer's cumulative ACK covers the whole
/// outstanding segment. Uses wrapping sequence arithmetic so it is correct
/// across the 32-bit wrap.
unsafe fn tcp_rt_ack(ack: u32) {
    if !CONN.rt_active {
        return;
    }
    let span = seq_span(CONN.rt_flags, CONN.rt_len);
    if span == 0 {
        return;
    }
    // Fully acknowledged when ack - rt_seq >= span (wrapping). A duplicate or
    // stale ACK (ack <= rt_seq, i.e. distance >= 2^31) leaves the timer armed.
    let dist = ack.wrapping_sub(CONN.rt_seq);
    if dist >= span && dist < 0x8000_0000 {
        CONN.snd_una = CONN.rt_seq.wrapping_add(span);
        CONN.rt_active = false;
        CONN.rt_retries = 0;
    }
}

/// One PIT tick of the retransmit timer: retransmit the oldest unacknowledged
/// segment when its RTO elapses (exponential backoff, capped), and tear the
/// connection down after TCP_MAX_RETRIES. Called once per tick while a
/// connection is active (full-os guide Part II.6, TCP reliability).
pub(crate) unsafe fn tcp_rt_tick() {
    if !CONN.rt_active || CONN.state == ST_CLOSED {
        return;
    }
    if CONN.rt_ticks_left > 0 {
        CONN.rt_ticks_left -= 1;
        return;
    }
    if CONN.rt_retries >= TCP_MAX_RETRIES {
        // Peer unreachable: abort. RFC 1122 §4.2.3.5 — send an RST so the peer
        // releases its half of the connection instead of waiting for its own
        // timers, then drop all connection state.
        tcp_tx(0x04, CONN.snd_nxt, CONN.rcv_nxt, &[]); // RST
        serial_write(b"TCP: rto giveup\n");
        conn_reset();
        return;
    }
    let len = CONN.rt_len;
    let mut buf = [0u8; RT_DATA_MAX];
    buf[..len].copy_from_slice(&CONN.rt_data[..len]);
    CONN.rt_last_send_ok = tcp_tx(CONN.rt_flags, CONN.rt_seq, CONN.rcv_nxt, &buf[..len]);
    CONN.rt_retries += 1;
    // Exponential backoff, capped at 16x the base RTO.
    let shift = CONN.rt_retries.min(4);
    CONN.rt_ticks_left = TCP_RTO_TICKS << shift;
    serial_write(b"TCP: rexmit\n");
}

unsafe fn arp_request() {
    let mut f = [0u8; 42];
    f[0..6].copy_from_slice(&[0xFF; 6]);
    f[6..12].copy_from_slice(&net::net_mac());
    f[12] = 0x08;
    f[13] = 0x06;
    f[14] = 0x00;
    f[15] = 0x01;
    f[16] = 0x08;
    f[17] = 0x00;
    f[18] = 6;
    f[19] = 4;
    f[20] = 0x00;
    f[21] = 0x01; // request
    f[22..28].copy_from_slice(&net::net_mac());
    f[28..32].copy_from_slice(&GUEST_IP);
    f[38..42].copy_from_slice(&CONN.peer_ip);
    let _ = net::wire_send(&f);
}

/// Begin an outbound connection. Returns false when one is already live.
pub(crate) unsafe fn tcp_connect(dst: [u8; 4], port: u16) -> bool {
    if CONN.state != ST_CLOSED {
        return false;
    }
    CONN.peer_ip = dst;
    CONN.remote_port = port;
    CONN.local_port = 0xC000 | (port & 0x0FFF); // deterministic ephemeral
    CONN.snd_nxt = 0x0001_0000;
    CONN.snd_una = 0x0001_0000;
    CONN.rcv_nxt = 0;
    CONN.rx_len = 0;
    CONN.peer_fin = false;
    CONN.rt_active = false;
    if CONN.have_mac {
        CONN.state = ST_SYN_SENT;
        let syn_seq = CONN.snd_nxt;
        tcp_tx(0x02, syn_seq, 0, &[]); // SYN
        tcp_rt_arm(0x02, syn_seq, &[]); // retransmit the SYN if unacked
        CONN.snd_nxt = CONN.snd_nxt.wrapping_add(1);
        serial_write(b"TCP: syn sent\n");
    } else {
        CONN.state = ST_ARP_WAIT;
        arp_request();
    }
    true
}

pub(crate) unsafe fn on_arp_reply(sender_ip: &[u8], sender_mac: &[u8]) {
    if CONN.state == ST_ARP_WAIT && sender_ip == CONN.peer_ip {
        CONN.peer_mac.copy_from_slice(&sender_mac[..6]);
        CONN.have_mac = true;
        CONN.state = ST_SYN_SENT;
        let syn_seq = CONN.snd_nxt;
        tcp_tx(0x02, syn_seq, 0, &[]);
        tcp_rt_arm(0x02, syn_seq, &[]); // retransmit the SYN if unacked
        CONN.snd_nxt = CONN.snd_nxt.wrapping_add(1);
        serial_write(b"TCP: syn sent\n");
    }
}

/// Handle one received IPv4/TCP packet (ip = the IP header onward).
pub(crate) unsafe fn tcp_input(ip: &[u8]) {
    if CONN.state == ST_CLOSED {
        return;
    }
    let ihl = ((ip[0] & 0x0F) as usize) * 4;
    if ip.len() < ihl + 20 {
        return;
    }
    if ip[12..16] != CONN.peer_ip || ip[16..20] != GUEST_IP {
        return;
    }
    let total_len = u16::from_be_bytes([ip[2], ip[3]]) as usize;
    if total_len < ihl + 20 || total_len > ip.len() {
        return;
    }
    let t = &ip[ihl..total_len];
    let src_port = u16::from_be_bytes([t[0], t[1]]);
    let dst_port = u16::from_be_bytes([t[2], t[3]]);
    if src_port != CONN.remote_port || dst_port != CONN.local_port {
        return;
    }
    let seq = u32::from_be_bytes([t[4], t[5], t[6], t[7]]);
    let flags = t[13];
    let doff = ((t[12] >> 4) as usize) * 4;
    if doff < 20 || doff > t.len() {
        return;
    }
    let payload = &t[doff..];

    if flags & 0x04 != 0 {
        // RST
        CONN.rt_active = false;
        CONN.state = ST_CLOSED;
        serial_write(b"TCP: rst\n");
        return;
    }

    // A cumulative ACK retires the oldest unacknowledged segment, stopping its
    // retransmit timer (full-os guide Part II.6, TCP reliability).
    if flags & 0x10 != 0 {
        let ack = u32::from_be_bytes([t[8], t[9], t[10], t[11]]);
        tcp_rt_ack(ack);
        // A close() deferred while data was outstanding (so it would not clobber
        // the retransmit slot) is acted on now that the slot is clear. This ACK
        // acknowledged the DATA, not the FIN we are about to send, so return
        // afterwards rather than falling through to the FIN_WAIT arm (which would
        // otherwise treat this same ACK as acking the FIN and close early).
        if CONN.pending_close && !CONN.rt_active && CONN.state == ST_ESTABLISHED {
            let fin_seq = CONN.snd_nxt;
            tcp_tx(0x11, fin_seq, CONN.rcv_nxt, &[]); // FIN|ACK
            tcp_rt_arm(0x11, fin_seq, &[]);
            CONN.snd_nxt = CONN.snd_nxt.wrapping_add(1);
            CONN.state = ST_FIN_WAIT;
            CONN.pending_close = false;
            return;
        }
    }

    match CONN.state {
        ST_LISTEN => {
            // Passive open: a bare SYN (SYN set, ACK clear) -> SYN|ACK.
            if flags & 0x12 == 0x02 {
                CONN.rcv_nxt = seq.wrapping_add(1);
                CONN.remote_port = src_port;
                tcp_tx(0x12, CONN.snd_nxt, CONN.rcv_nxt, &[]); // SYN|ACK
                CONN.snd_nxt = CONN.snd_nxt.wrapping_add(1); // SYN takes a seq
                CONN.state = ST_SYN_RCVD;
                serial_write(b"TCP: syn-rcvd\n");
            }
        }
        ST_SYN_RCVD => {
            // The client's ACK completes the three-way handshake.
            if flags & 0x10 != 0 {
                CONN.state = ST_ESTABLISHED;
                serial_write(b"TCP: established\n");
            }
        }
        ST_SYN_SENT => {
            if flags & 0x12 == 0x12 {
                // SYN|ACK
                CONN.rcv_nxt = seq.wrapping_add(1);
                tcp_tx(0x10, CONN.snd_nxt, CONN.rcv_nxt, &[]); // ACK
                CONN.state = ST_ESTABLISHED;
                serial_write(b"TCP: established\n");
            }
        }
        ST_ESTABLISHED => {
            if !payload.is_empty() && seq == CONN.rcv_nxt {
                let room = RX_RING - CONN.rx_len;
                let n = payload.len().min(room);
                CONN.rx[CONN.rx_len..CONN.rx_len + n].copy_from_slice(&payload[..n]);
                CONN.rx_len += n;
                CONN.rcv_nxt = CONN.rcv_nxt.wrapping_add(payload.len() as u32);
                tcp_tx(0x10, CONN.snd_nxt, CONN.rcv_nxt, &[]);
            }
            if flags & 0x01 != 0 {
                // FIN
                CONN.rcv_nxt = CONN.rcv_nxt.wrapping_add(1);
                tcp_tx(0x10, CONN.snd_nxt, CONN.rcv_nxt, &[]);
                CONN.peer_fin = true;
            }
        }
        ST_FIN_WAIT => {
            if flags & 0x01 != 0 {
                CONN.rcv_nxt = CONN.rcv_nxt.wrapping_add(1);
                tcp_tx(0x10, CONN.snd_nxt, CONN.rcv_nxt, &[]);
            }
            if flags & 0x10 != 0 {
                CONN.state = ST_CLOSED;
                serial_write(b"TCP: closed\n");
            }
        }
        _ => {}
    }
}

/// Send payload on the established connection. Returns bytes queued or 0.
pub(crate) unsafe fn tcp_send(data: &[u8]) -> usize {
    if CONN.state != ST_ESTABLISHED || data.is_empty() || data.len() > 512 {
        return 0;
    }
    // One outstanding segment at a time in this v1: refuse a new send while a
    // prior segment is still unacknowledged (it owns the single retransmit slot).
    if CONN.rt_active {
        return 0;
    }
    let seq = CONN.snd_nxt;
    if !tcp_tx(0x18, seq, CONN.rcv_nxt, data) {
        return 0;
    }
    tcp_rt_arm(0x18, seq, data); // retransmit the data if unacked
    CONN.snd_nxt = CONN.snd_nxt.wrapping_add(data.len() as u32);
    data.len()
}

/// Drain received bytes. Returns 0 when nothing is buffered.
pub(crate) unsafe fn tcp_recv(dst: &mut [u8]) -> usize {
    let n = CONN.rx_len.min(dst.len());
    if n == 0 {
        return 0;
    }
    dst[..n].copy_from_slice(&CONN.rx[..n]);
    CONN.rx.copy_within(n..CONN.rx_len, 0);
    CONN.rx_len -= n;
    n
}

pub(crate) unsafe fn tcp_close() {
    match CONN.state {
        ST_ESTABLISHED => {
            if CONN.rt_active {
                // A data segment is still unacknowledged and owns the single
                // retransmit slot. Sending the FIN now would clobber that slot
                // and lose the data on loss; defer the FIN until the data is
                // acked (tcp_input fires it once the slot clears).
                CONN.pending_close = true;
            } else {
                let fin_seq = CONN.snd_nxt;
                tcp_tx(0x11, fin_seq, CONN.rcv_nxt, &[]); // FIN|ACK
                tcp_rt_arm(0x11, fin_seq, &[]); // retransmit the FIN if unacked
                CONN.snd_nxt = CONN.snd_nxt.wrapping_add(1);
                CONN.state = ST_FIN_WAIT;
            }
        }
        ST_CLOSED => {}
        _ => {
            CONN.rt_active = false;
            CONN.state = ST_CLOSED;
        }
    }
}

/// Build a bare 40-byte IPv4+TCP segment (no options, no payload) into `seg`
/// for the listener self-test. tcp_input does not validate IP/TCP checksums,
/// so they are left zero.
unsafe fn build_seg(
    seg: &mut [u8; 40],
    src_ip: &[u8; 4],
    src_port: u16,
    dst_port: u16,
    seq: u32,
    ack: u32,
    flags: u8,
) {
    *seg = [0u8; 40];
    seg[0] = 0x45;
    seg[2] = 0;
    seg[3] = 40; // total length
    seg[8] = 64; // ttl
    seg[9] = 6; // proto TCP
    seg[12..16].copy_from_slice(src_ip);
    seg[16..20].copy_from_slice(&GUEST_IP);
    let t = &mut seg[20..];
    t[0..2].copy_from_slice(&src_port.to_be_bytes());
    t[2..4].copy_from_slice(&dst_port.to_be_bytes());
    t[4..8].copy_from_slice(&seq.to_be_bytes());
    t[8..12].copy_from_slice(&ack.to_be_bytes());
    t[12] = 5 << 4; // data offset = 20 bytes
    t[13] = flags;
    t[14] = 0x10; // window 4096
}

/// Restore the single global connection to its pristine CLOSED state. Used by
/// the listener self-test so it leaves NO residue (especially have_mac /
/// peer_mac) that would corrupt the next outbound tcp_connect.
unsafe fn conn_reset() {
    CONN.state = ST_CLOSED;
    CONN.peer_ip = [0; 4];
    CONN.peer_mac = [0; 6];
    CONN.have_mac = false;
    CONN.local_port = 0;
    CONN.remote_port = 0;
    CONN.snd_nxt = 0;
    CONN.snd_una = 0;
    CONN.rcv_nxt = 0;
    CONN.peer_fin = false;
    CONN.rx_len = 0;
    CONN.rt_active = false;
    CONN.rt_retries = 0;
    CONN.rt_ticks_left = 0;
    CONN.rt_last_send_ok = false;
    CONN.pending_close = false;
}

/// Passive-open (listener) self-test (full-os guide Part II.6): bind a listener
/// to :8080 with a synthetic peer, feed a SYN, expect SYN_RCVD + a SYN|ACK,
/// then feed the client's ACK and expect ESTABLISHED. Returns 1 on success.
/// Wildcard accept (any peer) and a multi-connection table are carry-forward.
pub(crate) unsafe fn tcp_listen_selftest() -> u64 {
    // Never disturb a live connection: refuse if one is in flight (mirrors
    // tcp_connect's guard). The boot self-test runs while CONN is CLOSED.
    if CONN.state != ST_CLOSED {
        return 0;
    }
    let client_ip = [10, 0, 2, 99];
    CONN.state = ST_LISTEN;
    CONN.peer_ip = client_ip;
    CONN.peer_mac = [0x52, 0x55, 0x0a, 0x00, 0x02, 0x63];
    CONN.have_mac = true;
    CONN.local_port = 8080;
    CONN.remote_port = 50000;
    CONN.snd_nxt = 0x0000_1000;
    CONN.rcv_nxt = 0;
    CONN.rx_len = 0;
    CONN.peer_fin = false;

    let client_isn = 0x0000_2000u32;
    let mut seg = [0u8; 40];
    // 1) inbound SYN from the client.
    build_seg(&mut seg, &client_ip, 50000, 8080, client_isn, 0, 0x02);
    tcp_input(&seg);
    if CONN.state != ST_SYN_RCVD {
        conn_reset();
        return 0;
    }
    // 2) inbound ACK completing the handshake (acks our ISN+1).
    build_seg(
        &mut seg,
        &client_ip,
        50000,
        8080,
        client_isn.wrapping_add(1),
        CONN.snd_nxt,
        0x10,
    );
    tcp_input(&seg);
    let ok = CONN.state == ST_ESTABLISHED;
    // Fully reset (state AND have_mac/peer_mac/...) so the live outbound client
    // path (tcp_connect) re-resolves ARP and is unaffected.
    conn_reset();
    if !ok {
        return 0;
    }
    serial_write(b"TCP: listen ok\n");
    1
}

/// Drive `n` retransmit-timer ticks.
unsafe fn rt_ticks(n: u32) {
    let mut i = 0u32;
    while i < n {
        tcp_rt_tick();
        i += 1;
    }
}

/// Build + feed an ACK segment from the synthetic peer used by the RTO self-test.
unsafe fn rto_feed_ack(peer_ip: &[u8; 4], ack: u32) {
    let mut seg = [0u8; 40];
    build_seg(&mut seg, peer_ip, 50001, 9090, CONN.rcv_nxt, ack, 0x10);
    tcp_input(&seg);
}

/// Retransmission / RTO self-test (full-os guide Part II.6) on a synthetic
/// established connection. Proves, deterministically (QEMU's user-net is
/// loss-free, so the timeout path is unobservable on the live wire): the
/// retransmit fires on exactly the right tick and actually emits; the backoff
/// doubles the interval for the second retransmit; a stale/partial ACK does NOT
/// clear the timer while a full-cover ACK does (advancing snd_una); a fresh send
/// re-arms a new segment; and a close() while data is outstanding defers the FIN
/// (rather than clobbering the data) until the data is acked. Returns 1 on ok.
pub(crate) unsafe fn tcp_rto_selftest() -> u64 {
    if CONN.state != ST_CLOSED {
        return 0;
    }
    let peer_ip = [10, 0, 2, 99];
    CONN.state = ST_ESTABLISHED;
    CONN.peer_ip = peer_ip;
    CONN.peer_mac = [0x52, 0x55, 0x0a, 0x00, 0x02, 0x63];
    CONN.have_mac = true;
    CONN.local_port = 9090;
    CONN.remote_port = 50001;
    CONN.snd_nxt = 0x0000_5000;
    CONN.snd_una = 0x0000_5000;
    CONN.rcv_nxt = 0x0000_9000;
    CONN.rx_len = 0;
    CONN.peer_fin = false;
    CONN.rt_active = false;
    CONN.rt_retries = 0;
    CONN.pending_close = false;

    let data = b"rto-test"; // 8 bytes
    let dlen = data.len() as u32;

    // 1) Send: a segment must now be outstanding (armed, zero retries) at snd_nxt.
    if tcp_send(data) != data.len() || !CONN.rt_active || CONN.rt_retries != 0 {
        conn_reset();
        return 0;
    }
    let armed_seq = CONN.rt_seq;
    if armed_seq != 0x0000_5000 || CONN.snd_una != 0x0000_5000 {
        conn_reset();
        return 0;
    }

    // 2) The retransmit must fire on EXACTLY tick TCP_RTO_TICKS+1, not before:
    // after TCP_RTO_TICKS ticks the countdown is at zero but has NOT yet fired.
    rt_ticks(TCP_RTO_TICKS);
    if CONN.rt_retries != 0 {
        conn_reset();
        return 0;
    }
    tcp_rt_tick(); // the (TCP_RTO_TICKS+1)th tick
    if CONN.rt_retries != 1 || !CONN.rt_active {
        conn_reset();
        return 0;
    }
    // The retransmit must have actually emitted a segment, and the backoff must
    // have doubled the interval (RTO << 1) for the next attempt.
    if !CONN.rt_last_send_ok || CONN.rt_ticks_left != TCP_RTO_TICKS << 1 {
        conn_reset();
        return 0;
    }

    // 3) Backoff honored: the SECOND retransmit must take RTO<<1 ticks. After
    // RTO<<1 ticks it must still be at 1 retry; the next tick makes it 2.
    rt_ticks(TCP_RTO_TICKS << 1);
    if CONN.rt_retries != 1 {
        conn_reset();
        return 0;
    }
    tcp_rt_tick();
    if CONN.rt_retries != 2 || CONN.rt_ticks_left != TCP_RTO_TICKS << 2 {
        conn_reset();
        return 0;
    }

    // 4) A stale ACK (at the segment's own seq) and a partial ACK (one byte in)
    // must NOT clear the timer or advance snd_una.
    rto_feed_ack(&peer_ip, armed_seq); // stale: ack == rt_seq
    if !CONN.rt_active || CONN.snd_una != armed_seq {
        conn_reset();
        return 0;
    }
    rto_feed_ack(&peer_ip, armed_seq.wrapping_add(1)); // partial: < full span
    if !CONN.rt_active {
        conn_reset();
        return 0;
    }

    // 5) A full-cover ACK clears the timer and advances snd_una.
    let ack = armed_seq.wrapping_add(dlen);
    rto_feed_ack(&peer_ip, ack);
    if CONN.rt_active || CONN.snd_una != ack {
        conn_reset();
        return 0;
    }

    // 6) A fresh send must re-arm a NEW segment at the advanced snd_nxt and be
    // the one retransmitted (proves the slot truly retired, not stuck on stale).
    let snd6 = CONN.snd_nxt;
    if tcp_send(data) != data.len() || CONN.rt_seq != snd6 || CONN.rt_flags != 0x18 {
        conn_reset();
        return 0;
    }
    rt_ticks(TCP_RTO_TICKS + 1);
    if CONN.rt_retries != 1 || CONN.rt_seq != snd6 {
        conn_reset();
        return 0;
    }
    rto_feed_ack(&peer_ip, snd6.wrapping_add(dlen));
    if CONN.rt_active {
        conn_reset();
        return 0;
    }

    // 7) Deferred close: closing while data is outstanding must NOT clobber the
    // retransmit slot — the data stays armed and the FIN is deferred until the
    // data is acked, then the FIN takes the slot and the state moves to FIN_WAIT.
    let snd7 = CONN.snd_nxt;
    if tcp_send(data) != data.len() || CONN.rt_seq != snd7 || CONN.rt_flags != 0x18 {
        conn_reset();
        return 0;
    }
    tcp_close(); // data still unacked -> FIN deferred, slot keeps the data
    if CONN.state != ST_ESTABLISHED
        || !CONN.pending_close
        || CONN.rt_flags != 0x18
        || CONN.rt_seq != snd7
    {
        conn_reset();
        return 0;
    }
    rto_feed_ack(&peer_ip, snd7.wrapping_add(dlen)); // ack the data -> FIN goes
    if CONN.state != ST_FIN_WAIT || !CONN.rt_active || CONN.rt_flags != 0x11 {
        conn_reset();
        return 0;
    }

    conn_reset();
    serial_write(b"TCP: rto ok\n");
    1
}
