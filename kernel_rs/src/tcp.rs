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

const RX_RING: usize = 1024;
const GUEST_IP: [u8; 4] = [10, 0, 2, 15];

struct TcpConn {
    state: u8,
    peer_ip: [u8; 4],
    peer_mac: [u8; 6],
    have_mac: bool,
    local_port: u16,
    remote_port: u16,
    snd_nxt: u32,
    rcv_nxt: u32,
    peer_fin: bool,
    rx_len: usize,
    rx: [u8; RX_RING],
}

static mut CONN: TcpConn = TcpConn {
    state: ST_CLOSED,
    peer_ip: [0; 4],
    peer_mac: [0; 6],
    have_mac: false,
    local_port: 0,
    remote_port: 0,
    snd_nxt: 0,
    rcv_nxt: 0,
    peer_fin: false,
    rx_len: 0,
    rx: [0; RX_RING],
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
    CONN.rcv_nxt = 0;
    CONN.rx_len = 0;
    CONN.peer_fin = false;
    if CONN.have_mac {
        CONN.state = ST_SYN_SENT;
        tcp_tx(0x02, CONN.snd_nxt, 0, &[]); // SYN
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
        tcp_tx(0x02, CONN.snd_nxt, 0, &[]);
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
        CONN.state = ST_CLOSED;
        serial_write(b"TCP: rst\n");
        return;
    }

    match CONN.state {
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
    if !tcp_tx(0x18, CONN.snd_nxt, CONN.rcv_nxt, data) {
        return 0;
    }
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
            tcp_tx(0x11, CONN.snd_nxt, CONN.rcv_nxt, &[]); // FIN|ACK
            CONN.snd_nxt = CONN.snd_nxt.wrapping_add(1);
            CONN.state = ST_FIN_WAIT;
        }
        ST_CLOSED => {}
        _ => {
            CONN.state = ST_CLOSED;
        }
    }
}
