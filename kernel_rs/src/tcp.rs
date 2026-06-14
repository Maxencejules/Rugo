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
// Adaptive-RTO bounds in PIT ticks (full-os guide Part II.6, RTT estimation).
// RFC 6298 mandates a 1 s floor; this deterministic tick model keeps a modest
// floor (40 ms) so the self-test can exercise small RTTs, and a 60 s ceiling.
const TCP_RTO_MIN: u32 = 4;
const TCP_RTO_MAX: u32 = 6000;

// Free-running PIT-tick counter for RTT measurement (advanced once per
// tcp_rt_tick). Only deltas (ack tick - send tick) are used, so its absolute
// value and persistence across connections are irrelevant.
static mut TCP_TICK: u64 = 0;

// Congestion control (full-os guide Part II.6, RFC 5681): slow start + congestion
// avoidance, driven by the same cumulative-ACK and RTO-timeout events as the
// retransmit machinery. MSS-quantised, integer-only.
const TCP_MSS: u32 = 512;
const TCP_IW: u32 = TCP_MSS; // initial congestion window = 1 SMSS
const TCP_INIT_SSTHRESH: u32 = 65535; // start large -> begin in slow start

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
    // RTT estimation (full-os guide Part II.6, RFC 6298 + Karn's algorithm).
    // srtt8 = 8*SRTT, rttvar4 = 4*RTTVAR (integer fixed-point, no FP); rto_ticks
    // is the SRTT-derived base RTO that drives the retransmit timer. rt_send_tick
    // is the TCP_TICK at which the outstanding segment was first sent, so an ACK
    // can measure its RTT — but ONLY when the segment was never retransmitted
    // (Karn: a retransmitted segment's RTT is ambiguous and must not be sampled).
    srtt8: u32,
    rttvar4: u32,
    rto_ticks: u32,
    rtt_valid: bool,
    rt_send_tick: u64,
    // Congestion control (full-os guide Part II.6): congestion window + slow-start
    // threshold, both in bytes. cwnd < ssthresh => slow start (exponential);
    // else congestion avoidance (additive). An RTO timeout collapses cwnd to one
    // segment and halves ssthresh.
    cwnd: u32,
    ssthresh: u32,
    // Fast retransmit (RFC 5681 §3.2): consecutive duplicate ACKs for the
    // outstanding segment; the 3rd triggers an immediate retransmit + fast
    // recovery, without waiting for the RTO.
    dup_acks: u32,
    // Set once the outstanding segment has been retransmitted by fast retransmit
    // (rt_retries stays 0 so fast retransmit is distinguishable from an RTO
    // timeout). Karn: a re-sent segment's ACK must not produce an RTT sample.
    rt_resent: bool,
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
    srtt8: 0,
    rttvar4: 0,
    rto_ticks: TCP_RTO_TICKS,
    rtt_valid: false,
    rt_send_tick: 0,
    cwnd: TCP_IW,
    ssthresh: TCP_INIT_SSTHRESH,
    dup_acks: 0,
    rt_resent: false,
};

/// Congestion control on three duplicate ACKs (RFC 5681 §3.2, fast recovery):
/// set ssthresh to half the window (floored at 2·SMSS) and inflate cwnd to
/// ssthresh + 3·SMSS for the three segments that left the network.
unsafe fn cc_fast_recovery() {
    CONN.ssthresh = (CONN.cwnd / 2).max(2 * TCP_MSS);
    CONN.cwnd = CONN.ssthresh + 3 * TCP_MSS;
}

/// Congestion control on a cumulative ACK of `acked` new data bytes (RFC 5681).
/// In slow start (cwnd < ssthresh) cwnd grows by up to one SMSS per ACK
/// (exponential per RTT); in congestion avoidance it grows by ~SMSS²/cwnd per ACK
/// (additive, ~one SMSS per RTT).
unsafe fn cc_on_ack(acked: u32) {
    if acked == 0 {
        return;
    }
    if CONN.cwnd < CONN.ssthresh {
        CONN.cwnd = CONN.cwnd.saturating_add(acked.min(TCP_MSS));
    } else {
        let inc = ((TCP_MSS * TCP_MSS) / CONN.cwnd).max(1);
        CONN.cwnd = CONN.cwnd.saturating_add(inc);
    }
}

/// Congestion control on an RTO timeout (RFC 5681 §3.1): set ssthresh to half the
/// window (floored at 2·SMSS) and collapse cwnd to one segment, restarting slow
/// start.
unsafe fn cc_on_timeout() {
    CONN.ssthresh = (CONN.cwnd / 2).max(2 * TCP_MSS);
    CONN.cwnd = TCP_MSS;
}

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
    // Drive the timer from the SRTT-derived adaptive RTO (Part II.6), defaulting
    // to TCP_RTO_TICKS until the first RTT sample. Record the send tick so a
    // clean (never-retransmitted) ACK can measure this segment's RTT.
    CONN.rt_ticks_left = CONN.rto_ticks;
    CONN.rt_retries = 0;
    CONN.rt_send_tick = TCP_TICK;
    CONN.dup_acks = 0; // a fresh segment starts a new duplicate-ACK run
    CONN.rt_resent = false; // not yet retransmitted, so its ACK can sample RTT
}

/// Fold one RTT measurement `m` (in ticks) into SRTT/RTTVAR and recompute the
/// base RTO (full-os guide Part II.6, RFC 6298 with integer fixed-point):
/// srtt8 = 8*SRTT, rttvar4 = 4*RTTVAR, RTO = SRTT + max(1, 4*RTTVAR), clamped.
unsafe fn tcp_rtt_update(m: u32) {
    let m = m.max(1); // clock granularity floor (G = 1 tick)
    if !CONN.rtt_valid {
        // First measurement: SRTT = R, RTTVAR = R/2.
        CONN.srtt8 = m << 3;
        CONN.rttvar4 = m << 1; // 4 * (R/2) = 2*R
        CONN.rtt_valid = true;
    } else {
        // err = R - SRTT (old); SRTT += err/8; RTTVAR += (|err| - RTTVAR)/4.
        let srtt = (CONN.srtt8 >> 3) as i64;
        let err = m as i64 - srtt;
        CONN.srtt8 = (CONN.srtt8 as i64 + err) as u32; // (7/8)SRTT + (1/8)R
        let abserr = err.unsigned_abs() as i64;
        CONN.rttvar4 = (CONN.rttvar4 as i64 + abserr - (CONN.rttvar4 >> 2) as i64) as u32;
    }
    // RTO = SRTT + K*RTTVAR (K=4); rttvar4 already holds 4*RTTVAR.
    let rto = (CONN.srtt8 >> 3) + CONN.rttvar4.max(1);
    CONN.rto_ticks = rto.clamp(TCP_RTO_MIN, TCP_RTO_MAX);
}

/// Clear the retransmit timer once the peer's cumulative ACK covers the whole
/// outstanding segment. Uses wrapping sequence arithmetic so it is correct
/// across the 32-bit wrap.
unsafe fn tcp_rt_ack(ack: u32, is_pure_ack: bool) {
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
        // Karn's algorithm: only sample RTT from a segment that was sent exactly
        // once. A segment retransmitted by EITHER an RTO timeout (rt_retries > 0)
        // OR fast retransmit (rt_resent) has an ambiguous ACK, so it must not
        // update SRTT.
        if CONN.rt_retries == 0 && !CONN.rt_resent {
            let m = (TCP_TICK.wrapping_sub(CONN.rt_send_tick)) as u32;
            tcp_rtt_update(m);
        }
        // Congestion control: a cumulative ACK of new data grows the window.
        // SYN/FIN-only segments (rt_len == 0) carry no data, so they do not.
        if CONN.rt_len > 0 {
            cc_on_ack(CONN.rt_len as u32);
        }
        CONN.snd_una = CONN.rt_seq.wrapping_add(span);
        CONN.rt_active = false;
        CONN.rt_retries = 0;
        CONN.dup_acks = 0; // new data acked: reset the duplicate run
    } else if is_pure_ack && ack == CONN.snd_una {
        // Duplicate ACK (RFC 5681 §2: a true dup ACK carries NO data and re-acks
        // the same point while the outstanding segment is still missing). The 3rd
        // triggers fast retransmit + fast recovery (§3.2) — re-send immediately
        // rather than waiting for the RTO. A data-carrying segment with the same
        // ack is NOT a dup ACK (is_pure_ack gates this), so a full-duplex peer
        // streaming to us does not spuriously trip fast retransmit.
        CONN.dup_acks += 1;
        if CONN.dup_acks == 3 {
            cc_fast_recovery();
            let len = CONN.rt_len;
            let mut buf = [0u8; RT_DATA_MAX];
            buf[..len].copy_from_slice(&CONN.rt_data[..len]);
            CONN.rt_last_send_ok = tcp_tx(CONN.rt_flags, CONN.rt_seq, CONN.rcv_nxt, &buf[..len]);
            CONN.rt_ticks_left = CONN.rto_ticks; // restart the RTO timer
            CONN.rt_resent = true; // retransmitted: its ACK can no longer sample RTT (Karn)
            serial_write(b"TCP: fast rexmit\n");
        }
    }
}

/// One PIT tick of the retransmit timer: retransmit the oldest unacknowledged
/// segment when its RTO elapses (exponential backoff, capped), and tear the
/// connection down after TCP_MAX_RETRIES. Called once per tick while a
/// connection is active (full-os guide Part II.6, TCP reliability).
pub(crate) unsafe fn tcp_rt_tick() {
    // Advance the free-running RTT clock every tick (before the early return) so
    // an ACK can measure elapsed ticks regardless of timer activity.
    TCP_TICK = TCP_TICK.wrapping_add(1);
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
    // RTO timeout: collapse the congestion window and restart slow start
    // (RFC 5681 §3.1) before retransmitting; a timeout exits fast recovery.
    cc_on_timeout();
    CONN.dup_acks = 0;
    let len = CONN.rt_len;
    let mut buf = [0u8; RT_DATA_MAX];
    buf[..len].copy_from_slice(&CONN.rt_data[..len]);
    CONN.rt_last_send_ok = tcp_tx(CONN.rt_flags, CONN.rt_seq, CONN.rcv_nxt, &buf[..len]);
    CONN.rt_retries += 1;
    // Exponential backoff from the adaptive base RTO, capped at 16x.
    let shift = CONN.rt_retries.min(4);
    CONN.rt_ticks_left = CONN.rto_ticks << shift;
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
        // A true duplicate ACK carries no payload (RFC 5681 §2); pass that so a
        // data-carrying segment from a full-duplex peer is not miscounted.
        tcp_rt_ack(ack, payload.is_empty());
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
    // A new connection starts with no RTT estimate (RTO back to the default).
    CONN.srtt8 = 0;
    CONN.rttvar4 = 0;
    CONN.rto_ticks = TCP_RTO_TICKS;
    CONN.rtt_valid = false;
    CONN.rt_send_tick = 0;
    CONN.cwnd = TCP_IW;
    CONN.ssthresh = TCP_INIT_SSTHRESH;
    CONN.dup_acks = 0;
    CONN.rt_resent = false;
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

/// RTT-estimation self-test (full-os guide Part II.6, RFC 6298 + Karn) on a
/// synthetic established connection. Proves, deterministically: a clean
/// (never-retransmitted) ACK measured at a known tick delta seeds SRTT/RTTVAR
/// and the derived RTO with the exact fixed-point values; a second sample
/// smooths them per the EWMA recurrences AND the new RTO drives the next
/// segment's retransmit timer; and a RETRANSMITTED segment's ACK does NOT update
/// the estimate (Karn). Returns 1 on success.
pub(crate) unsafe fn tcp_rtt_selftest() -> u64 {
    if CONN.state != ST_CLOSED {
        return 0;
    }
    let peer_ip = [10, 0, 2, 99];
    CONN.state = ST_ESTABLISHED;
    CONN.peer_ip = peer_ip;
    CONN.peer_mac = [0x52, 0x55, 0x0a, 0x00, 0x02, 0x63];
    CONN.have_mac = true;
    CONN.local_port = 9090; // must match rto_feed_ack's dst_port
    CONN.remote_port = 50001;
    CONN.snd_nxt = 0x0000_6000;
    CONN.snd_una = 0x0000_6000;
    CONN.rcv_nxt = 0x0000_9000;
    CONN.rx_len = 0;
    CONN.peer_fin = false;
    CONN.rt_active = false;
    CONN.rt_retries = 0;
    CONN.pending_close = false;
    // Start from a clean estimate (a fresh connection).
    CONN.srtt8 = 0;
    CONN.rttvar4 = 0;
    CONN.rto_ticks = TCP_RTO_TICKS;
    CONN.rtt_valid = false;

    let data = b"rtt-test"; // 8 bytes
    let dlen = data.len() as u32;

    // Sample 1: send, let 10 ticks pass (< RTO so no retransmit), then ACK.
    // First measurement R=10 => SRTT=10 (srtt8=80), RTTVAR=5 (rttvar4=20),
    // RTO = SRTT + 4*RTTVAR = 10 + 20 = 30.
    let s1 = CONN.snd_nxt;
    if tcp_send(data) != data.len() {
        conn_reset();
        return 0;
    }
    rt_ticks(10);
    if CONN.rt_retries != 0 {
        conn_reset();
        return 0; // must not have retransmitted (Karn-valid sample)
    }
    rto_feed_ack(&peer_ip, s1.wrapping_add(dlen));
    if CONN.rt_active
        || !CONN.rtt_valid
        || CONN.srtt8 != 80
        || CONN.rttvar4 != 20
        || CONN.rto_ticks != 30
    {
        conn_reset();
        return 0;
    }

    // Sample 2: the next segment must be armed with the ADAPTIVE RTO (30),
    // proving the estimate drives the timer. R=20 =>
    //   err = 20 - 10 = 10; srtt8 = 80 + 10 = 90 (SRTT=11);
    //   rttvar4 = 20 + 10 - (20>>2=5) = 25 (RTTVAR=6.25);
    //   RTO = (90>>3=11) + 25 = 36.
    let s2 = CONN.snd_nxt;
    if tcp_send(data) != data.len() || CONN.rt_ticks_left != 30 {
        conn_reset();
        return 0;
    }
    rt_ticks(20);
    if CONN.rt_retries != 0 {
        conn_reset();
        return 0;
    }
    rto_feed_ack(&peer_ip, s2.wrapping_add(dlen));
    if CONN.srtt8 != 90 || CONN.rttvar4 != 25 || CONN.rto_ticks != 36 {
        conn_reset();
        return 0;
    }

    // Karn: a retransmitted segment's ACK must NOT update the estimate.
    let (srtt_b, rttvar_b, rto_b) = (CONN.srtt8, CONN.rttvar4, CONN.rto_ticks);
    let s3 = CONN.snd_nxt;
    if tcp_send(data) != data.len() {
        conn_reset();
        return 0;
    }
    rt_ticks(CONN.rto_ticks + 1); // force exactly one retransmit
    if CONN.rt_retries != 1 {
        conn_reset();
        return 0;
    }
    rto_feed_ack(&peer_ip, s3.wrapping_add(dlen));
    if CONN.rt_active
        || CONN.srtt8 != srtt_b
        || CONN.rttvar4 != rttvar_b
        || CONN.rto_ticks != rto_b
    {
        conn_reset();
        return 0; // Karn violated: a retransmitted RTT polluted the estimate
    }

    conn_reset();
    serial_write(b"TCP: rtt ok\n");
    1
}

/// Congestion-control self-test (full-os guide Part II.6, RFC 5681) on a
/// synthetic established connection. Proves, deterministically: in slow start a
/// full-MSS ACK grows cwnd by one SMSS (exponential per RTT); once cwnd reaches
/// ssthresh, congestion avoidance grows it by SMSS²/cwnd (additive); and an RTO
/// timeout halves ssthresh (floored at 2·SMSS) and collapses cwnd to one segment.
/// Returns 1 on success.
pub(crate) unsafe fn tcp_cc_selftest() -> u64 {
    if CONN.state != ST_CLOSED {
        return 0;
    }
    let peer_ip = [10, 0, 2, 99];
    CONN.state = ST_ESTABLISHED;
    CONN.peer_ip = peer_ip;
    CONN.peer_mac = [0x52, 0x55, 0x0a, 0x00, 0x02, 0x63];
    CONN.have_mac = true;
    CONN.local_port = 9090; // must match rto_feed_ack's dst_port
    CONN.remote_port = 50001;
    CONN.snd_nxt = 0x0000_7000;
    CONN.snd_una = 0x0000_7000;
    CONN.rcv_nxt = 0x0000_9000;
    CONN.rx_len = 0;
    CONN.peer_fin = false;
    CONN.rt_active = false;
    CONN.rt_retries = 0;
    CONN.pending_close = false;
    CONN.srtt8 = 0;
    CONN.rttvar4 = 0;
    CONN.rto_ticks = TCP_RTO_TICKS;
    CONN.rtt_valid = false;
    CONN.cwnd = TCP_IW; // 512
    CONN.ssthresh = TCP_INIT_SSTHRESH; // 65535 -> begin in slow start

    let data = [0x41u8; TCP_MSS as usize]; // a full-MSS (512-byte) segment
    let dlen = TCP_MSS;

    // Drive one clean send+ACK (RTT < RTO so no retransmit), returning false on
    // any send/timer anomaly. `pre` is asserted to be the cwnd before the ACK.
    // 1) Slow start: each full-MSS ACK adds one SMSS.
    let s1 = CONN.snd_nxt;
    if tcp_send(&data) != TCP_MSS as usize {
        conn_reset();
        return 0;
    }
    rt_ticks(5);
    rto_feed_ack(&peer_ip, s1.wrapping_add(dlen));
    if CONN.rt_active || CONN.cwnd != 1024 {
        conn_reset();
        return 0; // 512 + 512
    }
    let s2 = CONN.snd_nxt;
    if tcp_send(&data) != TCP_MSS as usize {
        conn_reset();
        return 0;
    }
    rt_ticks(5);
    rto_feed_ack(&peer_ip, s2.wrapping_add(dlen));
    if CONN.cwnd != 1536 {
        conn_reset();
        return 0; // 1024 + 512
    }

    // 2) Congestion avoidance: cwnd >= ssthresh -> add SMSS²/cwnd.
    CONN.cwnd = 1024;
    CONN.ssthresh = 1024;
    let s3 = CONN.snd_nxt;
    if tcp_send(&data) != TCP_MSS as usize {
        conn_reset();
        return 0;
    }
    rt_ticks(5);
    rto_feed_ack(&peer_ip, s3.wrapping_add(dlen));
    if CONN.cwnd != 1280 {
        conn_reset();
        return 0; // 1024 + (512*512/1024 = 256)
    }

    // 3) RTO timeout: ssthresh = max(cwnd/2, 2·SMSS); cwnd = 1 SMSS.
    CONN.cwnd = 4096;
    CONN.ssthresh = TCP_INIT_SSTHRESH;
    let s4 = CONN.snd_nxt;
    if tcp_send(&data) != TCP_MSS as usize {
        conn_reset();
        return 0;
    }
    rt_ticks(CONN.rto_ticks + 1); // force exactly one retransmit (a timeout)
    if CONN.rt_retries != 1 || CONN.cwnd != TCP_MSS || CONN.ssthresh != 2048 {
        conn_reset();
        return 0; // 4096/2 = 2048 (> 2*512); cwnd collapses to 512
    }
    let _ = s4;

    conn_reset();
    serial_write(b"TCP: cc ok\n");
    1
}

/// Fast-retransmit self-test (full-os guide Part II.6, RFC 5681 §3.2) on a
/// synthetic established connection. Proves: two duplicate ACKs do NOT
/// retransmit; the THIRD triggers an immediate retransmit (without the RTO
/// elapsing) and enters fast recovery (ssthresh = cwnd/2, cwnd = ssthresh +
/// 3·SMSS). Returns 1 on success.
pub(crate) unsafe fn tcp_fastrexmit_selftest() -> u64 {
    if CONN.state != ST_CLOSED {
        return 0;
    }
    let peer_ip = [10, 0, 2, 99];
    CONN.state = ST_ESTABLISHED;
    CONN.peer_ip = peer_ip;
    CONN.peer_mac = [0x52, 0x55, 0x0a, 0x00, 0x02, 0x63];
    CONN.have_mac = true;
    CONN.local_port = 9090; // must match rto_feed_ack's dst_port
    CONN.remote_port = 50001;
    CONN.snd_nxt = 0x0000_8000;
    CONN.snd_una = 0x0000_8000;
    CONN.rcv_nxt = 0x0000_9000;
    CONN.rx_len = 0;
    CONN.peer_fin = false;
    CONN.rt_active = false;
    CONN.rt_retries = 0;
    CONN.pending_close = false;
    CONN.srtt8 = 0;
    CONN.rttvar4 = 0;
    CONN.rto_ticks = TCP_RTO_TICKS;
    CONN.rtt_valid = false;
    CONN.cwnd = 8192; // a window large enough that ssthresh = cwnd/2 (not the floor)
    CONN.ssthresh = TCP_INIT_SSTHRESH;

    let data = b"fr-test!"; // 8 bytes
    let s = CONN.snd_nxt;
    if tcp_send(data) != data.len() || CONN.dup_acks != 0 {
        conn_reset();
        return 0;
    }
    // A DATA-carrying segment with ack == snd_una must NOT count as a duplicate
    // ACK (RFC 5681 §2): a full-duplex peer streaming to us must not trip fast
    // retransmit. Build a 1-byte-payload ACK at rcv_nxt with ack == s.
    {
        let mut dseg = [0u8; 41];
        let mut hdr = [0u8; 40];
        build_seg(&mut hdr, &peer_ip, 50001, 9090, CONN.rcv_nxt, s, 0x10);
        dseg[..40].copy_from_slice(&hdr);
        dseg[3] = 41; // IP total length: header (40) + 1 payload byte
        dseg[40] = 0x5A; // payload
        tcp_input(&dseg);
        if CONN.dup_acks != 0 {
            conn_reset();
            return 0; // a data segment was wrongly counted as a duplicate ACK
        }
    }
    // Two (pure) duplicate ACKs (ack == snd_una, no payload): no retransmit yet.
    rto_feed_ack(&peer_ip, s);
    rto_feed_ack(&peer_ip, s);
    if CONN.dup_acks != 2 || CONN.rt_retries != 0 {
        conn_reset();
        return 0;
    }
    // The third duplicate ACK triggers fast retransmit + fast recovery.
    rto_feed_ack(&peer_ip, s);
    if CONN.dup_acks != 3 || !CONN.rt_last_send_ok {
        conn_reset();
        return 0;
    }
    // Fast recovery window: ssthresh = max(8192/2, 2·MSS) = 4096; cwnd += 3·MSS.
    if CONN.ssthresh != 4096 || CONN.cwnd != 4096 + 3 * TCP_MSS {
        conn_reset();
        return 0;
    }
    // The retransmit happened via fast retransmit, NOT an RTO timeout.
    if CONN.rt_retries != 0 {
        conn_reset();
        return 0;
    }
    conn_reset();
    serial_write(b"TCP: fast rexmit ok\n");
    1
}

// ---- multi-segment send window (full-os guide Part II.6) ----
//
// A sliding send window over multiple outstanding segments, bounded by
// min(cwnd, peer receive window), with cumulative-ACK retirement — vs the live
// CONN's single-outstanding-segment model. v1: the window state machine + a
// self-test; unifying it with the live CONN's per-segment RTO/retransmit slots
// (so the wire path sends multiple segments) is the larger refactor and is
// carry-forward (the single-segment live path + its RTO/RTT/CC tests are
// unchanged). This proves the windowed-send accounting independently.
const SNDWIN_MAX_SEG: usize = 16;

struct SndWindow {
    una: u32,                       // oldest unacknowledged sequence
    nxt: u32,                       // next sequence to send
    cwnd: u32,                      // congestion window (bytes)
    rwnd: u32,                      // peer's advertised receive window (bytes)
    nseg: usize,                    // number of outstanding segments
    seglen: [u32; SNDWIN_MAX_SEG],  // their lengths, oldest first
}

impl SndWindow {
    /// Bytes currently in flight (sent, unacknowledged).
    fn inflight(&self) -> u32 {
        self.nxt.wrapping_sub(self.una)
    }
    /// The usable window: min(cwnd, rwnd) minus what is already in flight.
    fn usable(&self) -> u32 {
        let win = if self.cwnd < self.rwnd { self.cwnd } else { self.rwnd };
        win.saturating_sub(self.inflight())
    }
    /// Try to send a `len`-byte segment: allowed only if it fits the usable window
    /// and a free segment slot exists. Records it and advances snd_nxt.
    fn send(&mut self, len: u32) -> bool {
        if len == 0 || len > self.usable() || self.nseg >= SNDWIN_MAX_SEG {
            return false;
        }
        self.seglen[self.nseg] = len;
        self.nseg += 1;
        self.nxt = self.nxt.wrapping_add(len);
        true
    }
    /// Retire every segment fully covered by a cumulative ACK, sliding snd_una.
    fn ack(&mut self, ack: u32) {
        while self.nseg > 0 {
            let end = self.una.wrapping_add(self.seglen[0]);
            // Covered when ack - una >= seglen[0] (wrapping, forward only).
            if ack.wrapping_sub(self.una) < self.seglen[0]
                || ack.wrapping_sub(self.una) >= 0x8000_0000
            {
                break;
            }
            self.una = end;
            let mut i = 1;
            while i < self.nseg {
                self.seglen[i - 1] = self.seglen[i];
                i += 1;
            }
            self.nseg -= 1;
        }
    }
}

/// Multi-segment send-window self-test (full-os guide Part II.6): fill the window
/// up to cwnd, confirm a segment exceeding it is refused, retire part of it with a
/// cumulative ACK (sliding the window), and confirm more can then be sent.
/// Returns 1 on success.
pub(crate) unsafe fn tcp_sndwin_selftest() -> u64 {
    let mss = 512u32;
    let mut w = SndWindow {
        una: 1000,
        nxt: 1000,
        cwnd: 2000,
        rwnd: 8000,
        nseg: 0,
        seglen: [0; SNDWIN_MAX_SEG],
    };
    // 1) Three 512-byte segments fit (1536 <= cwnd 2000); the fourth (2048) does not.
    if !w.send(mss) || !w.send(mss) || !w.send(mss) {
        return 0;
    }
    if w.send(mss) {
        return 0; // the 4th must be refused: cwnd-bound (in-flight would be 2048)
    }
    if w.nseg != 3 || w.inflight() != 1536 || w.nxt != 1000 + 1536 {
        return 0;
    }
    // 2) A cumulative ACK for two segments slides the window and frees space.
    w.ack(1000 + 1024); // acks the first two segments
    if w.nseg != 1 || w.una != 1000 + 1024 || w.inflight() != 512 {
        return 0;
    }
    // 3) With 512 in flight and cwnd 2000, two more segments fit; a third overflows.
    if !w.send(mss) || !w.send(mss) {
        return 0; // in-flight 512 -> 1024 -> 1536
    }
    if w.send(mss) {
        return 0; // 2048 > 2000
    }
    if w.nseg != 3 || w.inflight() != 1536 {
        return 0;
    }
    // 4) The receive window also bounds sending: shrink rwnd below the in-flight.
    w.rwnd = 1536;
    if w.send(mss) {
        return 0; // usable = min(cwnd,rwnd) - inflight = 1536 - 1536 = 0
    }
    serial_write(b"TCP: sndwin ok\n");
    1
}
