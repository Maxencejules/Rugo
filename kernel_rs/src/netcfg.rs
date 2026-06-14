// DHCP + DNS clients (gap-analysis item 6 remainder): one outstanding
// UDP query over the VirtIO NIC, driven by the same RX pump as wire
// TCP. DHCP DISCOVER broadcasts to the network's DHCP server (QEMU's
// slirp answers offline); DNS sends an A query to the gateway (the
// acceptance test runs the resolver on the host side of slirp).

#![allow(dead_code)]

use crate::net;
use crate::{serial_write, serial_write_hex};

const GUEST_IP: [u8; 4] = [10, 0, 2, 15];
const GATEWAY_IP: [u8; 4] = [10, 0, 2, 2];
const RESOLVER_IP: [u8; 4] = [10, 0, 2, 3];

const Q_IDLE: u8 = 0;
const Q_DHCP: u8 = 1;
const Q_DNS_ARP: u8 = 2;
const Q_DNS: u8 = 3;
const Q_DONE: u8 = 4;
const Q_DHCP_REQ: u8 = 5; // awaiting ACK after sending REQUEST (full DORA)

const DHCP_XID: u32 = 0x5247_4F31; // "RGO1"
const DNS_TXID: u16 = 0x5255; // "RU"
const DHCP_CLIENT_PORT: u16 = 68;
const DNS_CLIENT_PORT: u16 = 0xD035;

struct NetQuery {
    state: u8,
    result: u64,
    server_ip: [u8; 4],
    server_mac: [u8; 6],
    have_mac: bool,
    server_port: u16,
    qname: [u8; 64],
    qname_len: usize,
}

static mut QUERY: NetQuery = NetQuery {
    state: Q_IDLE,
    result: 0,
    server_ip: [0; 4],
    server_mac: [0; 6],
    have_mac: false,
    server_port: 0,
    qname: [0; 64],
    qname_len: 0,
};

pub(crate) unsafe fn query_active() -> bool {
    QUERY.state != Q_IDLE && QUERY.state != Q_DONE
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

/// Compose and transmit one UDP datagram (UDP checksum 0 = unused,
/// which IPv4 permits).
unsafe fn udp_tx(
    dst_mac: &[u8; 6],
    src_ip: &[u8; 4],
    dst_ip: &[u8; 4],
    sport: u16,
    dport: u16,
    payload: &[u8],
) -> bool {
    let udp_len = 8 + payload.len();
    let ip_len = 20 + udp_len;
    let total = 14 + ip_len;
    if total > 1514 {
        return false;
    }
    let mut f = [0u8; 1514];
    f[0..6].copy_from_slice(dst_mac);
    f[6..12].copy_from_slice(&net::net_mac());
    f[12] = 0x08;
    f[13] = 0x00;
    let ip = &mut f[14..];
    ip[0] = 0x45;
    ip[2] = (ip_len >> 8) as u8;
    ip[3] = (ip_len & 0xFF) as u8;
    ip[5] = 0x02;
    ip[8] = 64;
    ip[9] = 17; // proto UDP
    ip[12..16].copy_from_slice(src_ip);
    ip[16..20].copy_from_slice(dst_ip);
    let mut s = 0u32;
    csum_words(&mut s, &ip[..20]);
    let c = csum_fold(s);
    ip[10] = (c >> 8) as u8;
    ip[11] = (c & 0xFF) as u8;
    let u = &mut ip[20..];
    u[0..2].copy_from_slice(&sport.to_be_bytes());
    u[2..4].copy_from_slice(&dport.to_be_bytes());
    u[4..6].copy_from_slice(&(udp_len as u16).to_be_bytes());
    u[8..8 + payload.len()].copy_from_slice(payload);
    net::wire_send(&f[..total])
}

/// Start a DHCP DISCOVER. Result (op 3) is the offered IPv4 address.
pub(crate) unsafe fn start_dhcp() -> u64 {
    if query_active() {
        return u64::MAX;
    }
    QUERY.state = Q_DHCP;
    QUERY.result = 0;
    let mut p = [0u8; 300];
    p[0] = 1; // BOOTREQUEST
    p[1] = 1; // ethernet
    p[2] = 6; // hlen
    p[4..8].copy_from_slice(&DHCP_XID.to_be_bytes());
    p[10] = 0x80; // broadcast flag
    p[28..34].copy_from_slice(&net::net_mac());
    p[236..240].copy_from_slice(&[0x63, 0x82, 0x53, 0x63]); // cookie
    p[240] = 53; // message type
    p[241] = 1;
    p[242] = 1; // DISCOVER
    p[243] = 255; // end
    let ok = udp_tx(
        &[0xFF; 6],
        &[0, 0, 0, 0],
        &[255, 255, 255, 255],
        DHCP_CLIENT_PORT,
        67,
        &p,
    );
    if !ok {
        QUERY.state = Q_IDLE;
        return u64::MAX;
    }
    0
}

/// Start a DNS A query. Port 53 goes to the slirp resolver; any other
/// port goes to the gateway (the host side, where tests run a server).
pub(crate) unsafe fn start_dns(name: &[u8], port: u16) -> u64 {
    if query_active() || name.is_empty() || name.len() > 63 {
        return u64::MAX;
    }
    QUERY.qname[..name.len()].copy_from_slice(name);
    QUERY.qname_len = name.len();
    QUERY.server_ip = if port == 53 { RESOLVER_IP } else { GATEWAY_IP };
    QUERY.server_port = port;
    QUERY.result = 0;
    if QUERY.have_mac {
        QUERY.state = Q_DNS;
        send_dns_query();
    } else {
        QUERY.state = Q_DNS_ARP;
        arp_request(&QUERY.server_ip.clone());
    }
    0
}

unsafe fn arp_request(target: &[u8; 4]) {
    let mut f = [0u8; 42];
    f[0..6].copy_from_slice(&[0xFF; 6]);
    f[6..12].copy_from_slice(&net::net_mac());
    f[12] = 0x08;
    f[13] = 0x06;
    f[15] = 0x01;
    f[16] = 0x08;
    f[18] = 6;
    f[19] = 4;
    f[21] = 0x01;
    f[22..28].copy_from_slice(&net::net_mac());
    f[28..32].copy_from_slice(&GUEST_IP);
    f[38..42].copy_from_slice(target);
    let _ = net::wire_send(&f);
}

pub(crate) unsafe fn on_arp_reply(sender_ip: [u8; 4], mac: &[u8; 6]) {
    if QUERY.state == Q_DNS_ARP && sender_ip == QUERY.server_ip {
        QUERY.server_mac = *mac;
        QUERY.have_mac = true;
        QUERY.state = Q_DNS;
        send_dns_query();
    }
}

unsafe fn send_dns_query() {
    // header + QNAME (single label set split on '.') + QTYPE/QCLASS
    let mut p = [0u8; 96];
    p[0..2].copy_from_slice(&DNS_TXID.to_be_bytes());
    p[2] = 0x01; // RD
    p[5] = 1; // QDCOUNT
    let mut w = 12usize;
    let name = &QUERY.qname[..QUERY.qname_len];
    let mut start = 0usize;
    let mut i = 0usize;
    while i <= name.len() {
        if i == name.len() || name[i] == b'.' {
            let label = &name[start..i];
            if label.is_empty() || label.len() > 63 {
                QUERY.state = Q_IDLE;
                return;
            }
            p[w] = label.len() as u8;
            w += 1;
            p[w..w + label.len()].copy_from_slice(label);
            w += label.len();
            start = i + 1;
        }
        i += 1;
    }
    p[w] = 0;
    w += 1;
    p[w + 1] = 1; // QTYPE A
    p[w + 3] = 1; // QCLASS IN
    w += 4;
    let server_ip = QUERY.server_ip;
    let server_mac = QUERY.server_mac;
    let port = QUERY.server_port;
    let _ = udp_tx(&server_mac, &GUEST_IP, &server_ip, DNS_CLIENT_PORT, port, &p[..w]);
}

/// Called from the RX pump for IPv4/UDP frames (ip = IPv4 header on).
pub(crate) unsafe fn udp_input(ip: &[u8]) {
    if ip.len() < 28 {
        return;
    }
    let ihl = ((ip[0] & 0x0F) as usize) * 4;
    let udp = &ip[ihl..];
    if udp.len() < 8 {
        return;
    }
    let dport = u16::from_be_bytes([udp[2], udp[3]]);
    let payload = &udp[8..];
    match QUERY.state {
        Q_DHCP if dport == DHCP_CLIENT_PORT => {
            // BOOTREPLY with our xid: take yiaddr.
            if payload.len() >= 240
                && payload[0] == 2
                && payload[4..8] == DHCP_XID.to_be_bytes()
            {
                let yiaddr = [payload[16], payload[17], payload[18], payload[19]];
                let ip4 = u32::from_be_bytes(yiaddr);
                QUERY.result = ip4 as u64;
                serial_write(b"DHCP: offer ip=0x");
                serial_write_hex(ip4 as u64);
                serial_write(b"\n");
                // Full DORA: confirm the lease with a REQUEST (option 50 =
                // offered IP, option 54 = server id = the slirp gateway).
                let mut q = [0u8; 300];
                q[0] = 1;
                q[1] = 1;
                q[2] = 6;
                q[4..8].copy_from_slice(&DHCP_XID.to_be_bytes());
                q[10] = 0x80;
                q[28..34].copy_from_slice(&net::net_mac());
                q[236..240].copy_from_slice(&[0x63, 0x82, 0x53, 0x63]);
                q[240] = 53;
                q[241] = 1;
                q[242] = 3; // REQUEST
                q[243] = 50;
                q[244] = 4;
                q[245..249].copy_from_slice(&yiaddr);
                q[249] = 54;
                q[250] = 4;
                q[251..255].copy_from_slice(&GATEWAY_IP);
                q[255] = 255;
                let _ = udp_tx(
                    &[0xFF; 6],
                    &[0, 0, 0, 0],
                    &[255, 255, 255, 255],
                    DHCP_CLIENT_PORT,
                    67,
                    &q,
                );
                QUERY.state = Q_DHCP_REQ;
                serial_write(b"DHCP: request sent\n");
            }
        }
        Q_DHCP_REQ if dport == DHCP_CLIENT_PORT => {
            // BOOTREPLY ACK with our xid: the lease is confirmed.
            if payload.len() >= 240
                && payload[0] == 2
                && payload[4..8] == DHCP_XID.to_be_bytes()
            {
                let ip4 = u32::from_be_bytes([
                    payload[16], payload[17], payload[18], payload[19],
                ]);
                QUERY.result = ip4 as u64;
                QUERY.state = Q_DONE;
                serial_write(b"DHCP: ack ip=0x");
                serial_write_hex(ip4 as u64);
                serial_write(b"\n");
            }
        }
        Q_DNS if dport == DNS_CLIENT_PORT => {
            if payload.len() < 12 || payload[0..2] != DNS_TXID.to_be_bytes() {
                return;
            }
            let ancount = u16::from_be_bytes([payload[6], payload[7]]);
            if ancount == 0 {
                return;
            }
            // Skip the question: labels then 4 bytes of type/class.
            let mut o = 12usize;
            while o < payload.len() && payload[o] != 0 {
                o += payload[o] as usize + 1;
            }
            o += 5; // NUL + qtype + qclass
            // Answer: name (compression pointer or labels), then
            // type(2) class(2) ttl(4) rdlen(2) rdata.
            if o >= payload.len() {
                return;
            }
            if payload[o] & 0xC0 == 0xC0 {
                o += 2;
            } else {
                while o < payload.len() && payload[o] != 0 {
                    o += payload[o] as usize + 1;
                }
                o += 1;
            }
            if o + 10 + 4 > payload.len() {
                return;
            }
            let rtype = u16::from_be_bytes([payload[o], payload[o + 1]]);
            let rdlen = u16::from_be_bytes([payload[o + 8], payload[o + 9]]);
            if rtype != 1 || rdlen != 4 {
                return;
            }
            let a = u32::from_be_bytes([
                payload[o + 10], payload[o + 11], payload[o + 12], payload[o + 13],
            ]);
            QUERY.result = a as u64;
            QUERY.state = Q_DONE;
            serial_write(b"DNS: a=0x");
            serial_write_hex(a as u64);
            serial_write(b"\n");
        }
        _ => {}
    }
}

const ICMP_IDENT: u16 = 0x5247; // "RG"

/// Build an ICMP echo *reply* for a received echo-*request* Ethernet frame.
/// `req` is the full frame (dst MAC = us, ethertype IPv4, proto ICMP, type 8,
/// dest IP = the guest). Writes the reply into `out` and returns its length, or
/// `None` if `req` is not a well-formed echo request addressed to us. Shared by
/// the live RX responder (`icmp_input`) and the self-test (`icmp_selftest`).
unsafe fn build_icmp_echo_reply(req: &[u8], out: &mut [u8]) -> Option<usize> {
    if req.len() < 14 + 20 + 8 {
        return None;
    }
    if u16::from_be_bytes([req[12], req[13]]) != 0x0800 {
        return None;
    }
    let ip = &req[14..];
    let ihl = ((ip[0] & 0x0F) as usize) * 4;
    if ihl < 20 || ip[9] != 1 {
        return None;
    }
    if ip[16..20] != GUEST_IP {
        return None;
    }
    let icmp_off = 14 + ihl;
    if req.len() < icmp_off + 8 || req[icmp_off] != 8 {
        return None;
    }
    let total = req.len();
    if total > out.len() {
        return None;
    }
    out[..total].copy_from_slice(&req[..total]);
    // Ethernet: dst = original sender, src = our MAC.
    out[0..6].copy_from_slice(&req[6..12]);
    out[6..12].copy_from_slice(&net::net_mac());
    // IPv4: swap src/dst, reset TTL, recompute the header checksum.
    let src = [ip[12], ip[13], ip[14], ip[15]];
    {
        let oip = &mut out[14..14 + ihl];
        oip[12..16].copy_from_slice(&GUEST_IP);
        oip[16..20].copy_from_slice(&src);
        oip[8] = 64;
        oip[10] = 0;
        oip[11] = 0;
        let mut s = 0u32;
        csum_words(&mut s, oip);
        let ck = csum_fold(s);
        oip[10] = (ck >> 8) as u8;
        oip[11] = (ck & 0xFF) as u8;
    }
    // ICMP: type 8 -> 0 (echo reply); recompute the ICMP checksum.
    {
        let oic = &mut out[icmp_off..total];
        oic[0] = 0;
        oic[2] = 0;
        oic[3] = 0;
        let mut s = 0u32;
        csum_words(&mut s, oic);
        let ck = csum_fold(s);
        oic[2] = (ck >> 8) as u8;
        oic[3] = (ck & 0xFF) as u8;
    }
    Some(total)
}

/// Live RX responder: reply to inbound pings so the guest is a pingable host
/// (full-os guide Part II.6). Called from the RX pump for IPv4/ICMP frames.
pub(crate) unsafe fn icmp_input(frame: &[u8]) {
    let mut out = [0u8; 1514];
    if let Some(len) = build_icmp_echo_reply(frame, &mut out) {
        let _ = net::wire_send(&out[..len]);
        serial_write(b"ICMP: echo reply sent\n");
    }
}

/// Self-test (op 4): synthesize an echo request to ourselves, run the real
/// responder, and verify the reply is a checksum-correct echo reply that
/// echoes the ident/seq/payload. Deterministic — no external responder needed.
/// Returns 1 on success, 0 on failure.
pub(crate) unsafe fn icmp_selftest() -> u64 {
    const PAYLOAD: &[u8; 8] = b"rugoping";
    let mut req = [0u8; 14 + 20 + 8 + 8];
    // Ethernet: from a fake gateway to us.
    req[0..6].copy_from_slice(&net::net_mac());
    req[6..12].copy_from_slice(&[0x52, 0x55, 0x0a, 0x00, 0x02, 0x02]);
    req[12] = 0x08;
    req[13] = 0x00;
    // IPv4 header.
    {
        let ip = &mut req[14..34];
        ip[0] = 0x45;
        let tot = (20 + 8 + 8) as u16;
        ip[2] = (tot >> 8) as u8;
        ip[3] = (tot & 0xFF) as u8;
        ip[8] = 64;
        ip[9] = 1;
        ip[12..16].copy_from_slice(&GATEWAY_IP);
        ip[16..20].copy_from_slice(&GUEST_IP);
        let mut s = 0u32;
        csum_words(&mut s, ip);
        let ck = csum_fold(s);
        ip[10] = (ck >> 8) as u8;
        ip[11] = (ck & 0xFF) as u8;
    }
    // ICMP echo request (type 8), ident/seq + payload.
    {
        let ic = &mut req[34..50];
        ic[0] = 8;
        ic[4] = (ICMP_IDENT >> 8) as u8;
        ic[5] = (ICMP_IDENT & 0xFF) as u8;
        ic[6] = 0x00;
        ic[7] = 0x01;
        ic[8..16].copy_from_slice(PAYLOAD);
        let mut s = 0u32;
        csum_words(&mut s, ic);
        let ck = csum_fold(s);
        ic[2] = (ck >> 8) as u8;
        ic[3] = (ck & 0xFF) as u8;
    }
    let mut out = [0u8; 1514];
    let len = match build_icmp_echo_reply(&req, &mut out) {
        Some(l) => l,
        None => return 0,
    };
    let oic = &out[34..len];
    // type 0, ident/seq/payload preserved.
    if oic[0] != 0
        || oic[4] != req[38]
        || oic[5] != req[39]
        || oic[6] != req[40]
        || oic[7] != req[41]
        || oic[8..16] != *PAYLOAD
    {
        return 0;
    }
    // Reply checksums must fold to zero (wire-correct).
    let mut v = 0u32;
    csum_words(&mut v, oic);
    if csum_fold(v) != 0 {
        return 0;
    }
    let mut v2 = 0u32;
    csum_words(&mut v2, &out[14..34]);
    if csum_fold(v2) != 0 {
        return 0;
    }
    serial_write(b"ICMP: echo reply ok seq=0x");
    serial_write_hex(1);
    serial_write(b"\n");
    1
}

/// Poll (op 3): -1 while pending; the result once, then idle.
pub(crate) unsafe fn poll_result() -> u64 {
    net::net_rx_pump();
    match QUERY.state {
        Q_DONE => {
            QUERY.state = Q_IDLE;
            QUERY.result
        }
        Q_IDLE => u64::MAX,
        _ => u64::MAX,
    }
}
