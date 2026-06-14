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

/// Build an ARP *reply* for a received ARP *request* Ethernet frame. `req` is
/// the full frame; if it is an ARP request (opcode 1) asking for the guest IP,
/// writes a 42-byte reply into `out` and returns its length, else None. Shared
/// by the live responder (`arp_input`) and the self-test (`arp_selftest`).
unsafe fn build_arp_reply(req: &[u8], out: &mut [u8]) -> Option<usize> {
    if req.len() < 42 || out.len() < 42 {
        return None;
    }
    if u16::from_be_bytes([req[12], req[13]]) != 0x0806 {
        return None;
    }
    let arp = &req[14..];
    // opcode 1 (request), target protocol address == GUEST_IP.
    if u16::from_be_bytes([arp[6], arp[7]]) != 1 || arp[24..28] != GUEST_IP {
        return None;
    }
    let mac = net::net_mac();
    let sender_mac = [arp[8], arp[9], arp[10], arp[11], arp[12], arp[13]];
    let sender_ip = [arp[14], arp[15], arp[16], arp[17]];
    // Ethernet: dst = requester, src = us, ethertype ARP.
    out[0..6].copy_from_slice(&sender_mac);
    out[6..12].copy_from_slice(&mac);
    out[12] = 0x08;
    out[13] = 0x06;
    // ARP reply.
    let a = &mut out[14..42];
    a[0] = 0x00;
    a[1] = 0x01; // HTYPE ethernet
    a[2] = 0x08;
    a[3] = 0x00; // PTYPE IPv4
    a[4] = 6;
    a[5] = 4;
    a[6] = 0x00;
    a[7] = 0x02; // opcode reply
    a[8..14].copy_from_slice(&mac); // sender MAC = us
    a[14..18].copy_from_slice(&GUEST_IP); // sender IP = guest
    a[18..24].copy_from_slice(&sender_mac); // target MAC = requester
    a[24..28].copy_from_slice(&sender_ip); // target IP = requester
    Some(42)
}

/// Live RX responder: answer ARP "who-has GUEST_IP" so the guest is reachable
/// (full-os guide Part II.6). Called from the RX pump for ARP request frames.
pub(crate) unsafe fn arp_input(frame: &[u8]) {
    let mut out = [0u8; 42];
    if let Some(len) = build_arp_reply(frame, &mut out) {
        let _ = net::wire_send(&out[..len]);
        serial_write(b"ARP: reply sent\n");
    }
}

/// Self-test (op 5): synthesize an ARP request for the guest IP, run the real
/// responder, and verify the reply (opcode 2, sender = our MAC/IP, target =
/// requester). Deterministic. Returns 1 on success, 0 on failure.
pub(crate) unsafe fn arp_selftest() -> u64 {
    let mut req = [0u8; 42];
    let fake_mac = [0x52, 0x55, 0x0a, 0x00, 0x02, 0x02];
    let fake_ip = GATEWAY_IP;
    req[0..6].copy_from_slice(&[0xFF; 6]); // broadcast
    req[6..12].copy_from_slice(&fake_mac);
    req[12] = 0x08;
    req[13] = 0x06;
    {
        let a = &mut req[14..42];
        a[1] = 0x01;
        a[2] = 0x08;
        a[4] = 6;
        a[5] = 4;
        a[7] = 0x01; // request
        a[8..14].copy_from_slice(&fake_mac);
        a[14..18].copy_from_slice(&fake_ip);
        a[24..28].copy_from_slice(&GUEST_IP); // who-has guest
    }
    let mut out = [0u8; 42];
    let len = match build_arp_reply(&req, &mut out) {
        Some(l) => l,
        None => return 0,
    };
    let mac = net::net_mac();
    let a = &out[14..len];
    // opcode 2, sender = our MAC/IP, target = the requester.
    if a[6] != 0 || a[7] != 2 || a[8..14] != mac || a[14..18] != GUEST_IP
        || a[18..24] != fake_mac || a[24..28] != fake_ip
    {
        return 0;
    }
    // Ethernet dst = requester, src = us.
    if out[0..6] != fake_mac || out[6..12] != mac {
        return 0;
    }
    serial_write(b"ARP: reply ok\n");
    1
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

/// The guest's IPv6 link-local address (fe80::/64 + EUI-64 from the NIC MAC).
unsafe fn guest_ip6() -> [u8; 16] {
    let mac = net::net_mac();
    let mut a = [0u8; 16];
    a[0] = 0xfe;
    a[1] = 0x80;
    a[8] = mac[0] ^ 0x02; // flip the universal/local bit
    a[9] = mac[1];
    a[10] = mac[2];
    a[11] = 0xff;
    a[12] = 0xfe;
    a[13] = mac[3];
    a[14] = mac[4];
    a[15] = mac[5];
    a
}

/// ICMPv6 checksum over the IPv6 pseudo-header + message (`icmp6` has its
/// checksum field already zeroed). `src`/`dst` are the reply's addresses.
fn icmpv6_checksum(src: &[u8; 16], dst: &[u8; 16], icmp6: &[u8]) -> u16 {
    let mut s = 0u32;
    csum_words(&mut s, src);
    csum_words(&mut s, dst);
    s += icmp6.len() as u32; // upper-layer length
    s += 58; // next header = ICMPv6
    csum_words(&mut s, icmp6);
    csum_fold(s)
}

/// Build an ICMPv6 echo *reply* (type 129) for a received echo *request*
/// (type 128) Ethernet frame addressed to the guest's link-local address.
/// Returns the reply length in `out`, or None. Shared by the live responder
/// and the self-test (full-os guide Part II.6, IPv6).
unsafe fn build_icmpv6_echo_reply(req: &[u8], out: &mut [u8]) -> Option<usize> {
    if req.len() < 14 + 40 + 8 {
        return None;
    }
    if u16::from_be_bytes([req[12], req[13]]) != 0x86DD {
        return None;
    }
    let ip6 = &req[14..];
    if ip6[6] != 58 {
        return None; // next header must be ICMPv6
    }
    let g6 = guest_ip6();
    if ip6[24..40] != g6 {
        return None; // not addressed to us
    }
    let icmp_off = 14 + 40;
    if req[icmp_off] != 128 {
        return None; // not an echo request
    }
    let total = req.len();
    if total > out.len() {
        return None;
    }
    out[..total].copy_from_slice(&req[..total]);
    // Ethernet: dst = requester, src = us.
    out[0..6].copy_from_slice(&req[6..12]);
    out[6..12].copy_from_slice(&net::net_mac());
    let orig_src: [u8; 16] = ip6[8..24].try_into().ok()?;
    {
        let oip = &mut out[14..14 + 40];
        oip[7] = 255; // hop limit
        oip[8..24].copy_from_slice(&g6); // src = guest
        oip[24..40].copy_from_slice(&orig_src); // dst = requester
    }
    let icmp_len = total - icmp_off;
    {
        let oic = &mut out[icmp_off..total];
        oic[0] = 129; // echo reply
        oic[2] = 0;
        oic[3] = 0;
        let ck = icmpv6_checksum(&g6, &orig_src, oic);
        oic[2] = (ck >> 8) as u8;
        oic[3] = (ck & 0xFF) as u8;
    }
    Some(total)
}

/// Build an ICMPv6 Neighbor Advertisement (type 136) replying to a received
/// Neighbor Solicitation (type 135) whose target is the guest's link-local
/// address — so a host running IPv6 Neighbor Discovery can resolve the guest's
/// MAC (full-os guide Part II.6, IPv6 NDP). The NA carries the Solicited +
/// Override flags, the guest's address as the target, and a Target Link-Layer
/// Address option holding the guest MAC. Returns the reply length in `out`.
unsafe fn build_neighbor_advert(req: &[u8], out: &mut [u8]) -> Option<usize> {
    // Ethernet(14) + IPv6(40) + NS body: type/code/csum(4) + reserved(4) +
    // target(16) = 24. Options (e.g. the source link-layer address) may follow.
    if req.len() < 14 + 40 + 24 {
        return None;
    }
    if u16::from_be_bytes([req[12], req[13]]) != 0x86DD {
        return None;
    }
    let ip6 = &req[14..];
    if ip6[6] != 58 {
        return None; // next header must be ICMPv6
    }
    let icmp_off = 14 + 40;
    if req[icmp_off] != 135 {
        return None; // not a Neighbor Solicitation
    }
    let g6 = guest_ip6();
    // NS target address (the address being resolved) is at offset 8 in the body.
    let target: [u8; 16] = req[icmp_off + 8..icmp_off + 24].try_into().ok()?;
    if target != g6 {
        return None; // soliciting some other node, not us
    }
    let orig_src: [u8; 16] = ip6[8..24].try_into().ok()?;
    let orig_src_mac: [u8; 6] = req[6..12].try_into().ok()?;
    // RFC 4861 §7.2.4: an NS sourced from the unspecified address (::) is a
    // Duplicate Address Detection probe. The advertisement then MUST go to the
    // all-nodes multicast (ff02::1, MAC 33:33:00:00:00:01) with the Solicited
    // flag CLEARED — a unicast NA to :: would be a malformed, undeliverable
    // packet (RFC 4291 forbids :: as a destination). Otherwise (a normal
    // resolution NS) the NA is unicast back to the soliciting host with
    // Solicited+Override set.
    let dad = orig_src == [0u8; 16];
    let all_nodes: [u8; 16] = [0xff, 0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x01];
    let (na_dst, na_dst_mac, flags): ([u8; 16], [u8; 6], u8) = if dad {
        (all_nodes, [0x33, 0x33, 0x00, 0x00, 0x00, 0x01], 0x20) // Override only
    } else {
        (orig_src, orig_src_mac, 0x60) // Solicited + Override
    };
    // NA ICMPv6 message = header(4) + flags(4) + target(16) + TLLA option(8) = 32.
    let total = 14 + 40 + 32;
    if total > out.len() {
        return None;
    }
    for b in out[..total].iter_mut() {
        *b = 0;
    }
    // Ethernet: dst = the soliciting host (or all-nodes mcast for DAD), src = us.
    out[0..6].copy_from_slice(&na_dst_mac);
    out[6..12].copy_from_slice(&net::net_mac());
    out[12] = 0x86;
    out[13] = 0xDD;
    {
        let ip = &mut out[14..14 + 40];
        ip[0] = 0x60; // version 6
        let plen = (32u16).to_be_bytes(); // ICMPv6 payload length
        ip[4] = plen[0];
        ip[5] = plen[1];
        ip[6] = 58; // next header ICMPv6
        ip[7] = 255; // hop limit (NDP requires 255)
        ip[8..24].copy_from_slice(&g6); // src = guest
        ip[24..40].copy_from_slice(&na_dst); // dst = soliciting host / all-nodes
    }
    {
        let na = &mut out[icmp_off..total];
        na[0] = 136; // Neighbor Advertisement
        na[1] = 0; // code
        // Flag byte 4: R(0x80) Router, S(0x40) Solicited, O(0x20) Override.
        na[4] = flags;
        na[8..24].copy_from_slice(&g6); // target = guest's address
        na[24] = 2; // option type 2 = Target Link-Layer Address
        na[25] = 1; // length in 8-byte units (1 = 8 bytes)
        na[26..32].copy_from_slice(&net::net_mac());
        // Checksum over the IPv6 pseudo-header + message (field already zeroed).
        let ck = icmpv6_checksum(&g6, &na_dst, na);
        na[2] = (ck >> 8) as u8;
        na[3] = (ck & 0xFF) as u8;
    }
    Some(total)
}

/// Live RX responder: answer ICMPv6 echo requests (ping6) and Neighbor
/// Solicitations (NDP) so the guest is a reachable, resolvable IPv6 host.
/// Called from the RX pump for ethertype 0x86DD frames.
pub(crate) unsafe fn icmpv6_input(frame: &[u8]) {
    if frame.len() < 14 + 40 + 1 {
        return;
    }
    let mut out = [0u8; 1514];
    match frame[14 + 40] {
        128 => {
            if let Some(len) = build_icmpv6_echo_reply(frame, &mut out) {
                let _ = net::wire_send(&out[..len]);
                serial_write(b"ICMPV6: echo reply sent\n");
            }
        }
        135 => {
            if let Some(len) = build_neighbor_advert(frame, &mut out) {
                let _ = net::wire_send(&out[..len]);
                serial_write(b"NDP: advert sent\n");
            }
        }
        _ => {}
    }
}

/// Self-test (op 7): synthesize an ICMPv6 echo request to the guest's
/// link-local address, run the responder, and verify the reply is type 129
/// with a wire-correct checksum and echoed payload. Returns 1 on success.
pub(crate) unsafe fn icmpv6_selftest() -> u64 {
    const PAYLOAD: &[u8; 8] = b"rugo-v6!";
    let g6 = guest_ip6();
    let src6: [u8; 16] = [
        0xfe, 0x80, 0, 0, 0, 0, 0, 0, 0x02, 0x55, 0x0a, 0xff, 0xfe, 0x00, 0x02, 0x02,
    ];
    let mut req = [0u8; 14 + 40 + 16];
    req[0..6].copy_from_slice(&net::net_mac());
    req[6..12].copy_from_slice(&[0x52, 0x55, 0x0a, 0x00, 0x02, 0x02]);
    req[12] = 0x86;
    req[13] = 0xDD;
    {
        let ip6 = &mut req[14..54];
        ip6[0] = 0x60; // version 6
        let plen = (16u16).to_be_bytes(); // ICMPv6 payload length
        ip6[4] = plen[0];
        ip6[5] = plen[1];
        ip6[6] = 58; // next header ICMPv6
        ip6[7] = 255; // hop limit
        ip6[8..24].copy_from_slice(&src6);
        ip6[24..40].copy_from_slice(&g6);
    }
    {
        let ic = &mut req[54..70];
        ic[0] = 128; // echo request
        ic[4] = 0x52;
        ic[5] = 0x47; // ident
        ic[6] = 0x00;
        ic[7] = 0x01; // seq
        ic[8..16].copy_from_slice(PAYLOAD);
        let ck = icmpv6_checksum(&src6, &g6, ic);
        ic[2] = (ck >> 8) as u8;
        ic[3] = (ck & 0xFF) as u8;
    }
    let mut out = [0u8; 1514];
    let len = match build_icmpv6_echo_reply(&req, &mut out) {
        Some(l) => l,
        None => return 0,
    };
    let oic = &out[54..len];
    if oic[0] != 129 || oic[8..16] != *PAYLOAD {
        return 0;
    }
    // Verify the reply's ICMPv6 checksum folds to zero over the pseudo-header.
    let reply_src: [u8; 16] = out[22..38].try_into().unwrap();
    let reply_dst: [u8; 16] = out[38..54].try_into().unwrap();
    let mut s = 0u32;
    csum_words(&mut s, &reply_src);
    csum_words(&mut s, &reply_dst);
    s += oic.len() as u32;
    s += 58;
    csum_words(&mut s, oic);
    if csum_fold(s) != 0 {
        return 0;
    }
    serial_write(b"ICMPV6: echo reply ok\n");
    1
}

/// Self-test (op 9): synthesize a Neighbor Solicitation (type 135) for the
/// guest's link-local address — addressed to the solicited-node multicast, as a
/// real host's NDP would be — run the responder, and verify the reply is a
/// type-136 Neighbor Advertisement with the Solicited+Override flags, the
/// guest's address as target, a Target Link-Layer option carrying the guest
/// MAC, and a wire-correct checksum. Returns 1 on success (full-os Part II.6).
pub(crate) unsafe fn ndp_selftest() -> u64 {
    let g6 = guest_ip6();
    let mac = net::net_mac();
    let host6: [u8; 16] = [
        0xfe, 0x80, 0, 0, 0, 0, 0, 0, 0x02, 0x55, 0x0a, 0xff, 0xfe, 0x00, 0x02, 0x02,
    ];
    let host_mac = [0x52u8, 0x55, 0x0a, 0x00, 0x02, 0x02];
    // Solicited-node multicast of the target: ff02::1:ffXX:XXXX (low 24 bits).
    let mut snm = [0u8; 16];
    snm[0] = 0xff;
    snm[1] = 0x02;
    snm[11] = 0x01;
    snm[12] = 0xff;
    snm[13] = g6[13];
    snm[14] = g6[14];
    snm[15] = g6[15];
    // NS frame = eth(14) + ip6(40) + body(type/code/csum 4 + reserved 4 +
    // target 16 + SLLA option 8) = 86.
    let mut req = [0u8; 14 + 40 + 32];
    // Ethernet: dst = solicited-node multicast MAC 33:33:ff:XX:XX:XX, src = host.
    req[0] = 0x33;
    req[1] = 0x33;
    req[2] = 0xff;
    req[3] = g6[13];
    req[4] = g6[14];
    req[5] = g6[15];
    req[6..12].copy_from_slice(&host_mac);
    req[12] = 0x86;
    req[13] = 0xDD;
    {
        let ip6 = &mut req[14..54];
        ip6[0] = 0x60; // version 6
        let plen = (32u16).to_be_bytes();
        ip6[4] = plen[0];
        ip6[5] = plen[1];
        ip6[6] = 58; // next header ICMPv6
        ip6[7] = 255; // hop limit
        ip6[8..24].copy_from_slice(&host6);
        ip6[24..40].copy_from_slice(&snm);
    }
    {
        let ns = &mut req[54..86];
        ns[0] = 135; // Neighbor Solicitation
        // reserved bytes 4..8 stay zero
        ns[8..24].copy_from_slice(&g6); // target = guest
        ns[24] = 1; // Source Link-Layer Address option
        ns[25] = 1;
        ns[26..32].copy_from_slice(&host_mac);
        let ck = icmpv6_checksum(&host6, &snm, ns);
        ns[2] = (ck >> 8) as u8;
        ns[3] = (ck & 0xFF) as u8;
    }
    let mut out = [0u8; 1514];
    let len = match build_neighbor_advert(&req, &mut out) {
        Some(l) => l,
        None => return 0,
    };
    let na = &out[54..len];
    // ICMPv6 NA fields.
    if na[0] != 136 || na[1] != 0 {
        return 0; // type = Neighbor Advertisement, code = 0
    }
    if na[4] & 0x60 != 0x60 {
        return 0; // Solicited + Override set for a unicast resolution reply
    }
    if na[8..24] != g6 {
        return 0; // NA target must be the guest's address
    }
    if na[24] != 2 || na[25] != 1 || na[26..32] != mac {
        return 0; // TLLA option must carry the guest MAC
    }
    // Ethernet + IPv6 header must be wire-correct and deliverable. Check the
    // on-wire bytes against KNOWN-correct values (not values read back from the
    // builder's own output), so a regression in any of these fields is caught.
    if out[0..6] != host_mac || out[6..12] != mac || out[12] != 0x86 || out[13] != 0xDD {
        return 0; // eth dst = soliciting host, src = guest, ethertype IPv6
    }
    if out[21] != 255 {
        return 0; // hop limit MUST be 255 (RFC 4861 §7.1.2)
    }
    if u16::from_be_bytes([out[18], out[19]]) != 32 {
        return 0; // IPv6 payload length = NA message length
    }
    if out[22..38] != g6 || out[38..54] != host6 {
        return 0; // IPv6 src = guest, dst = soliciting host
    }
    // The checksum must fold to zero computed from the known-correct addresses
    // (g6, host6) and the on-wire payload length — not values read back from the
    // builder — so a swapped/garbled src/dst cannot pass.
    let mut s = 0u32;
    csum_words(&mut s, &g6);
    csum_words(&mut s, &host6);
    s += u16::from_be_bytes([out[18], out[19]]) as u32;
    s += 58;
    csum_words(&mut s, na);
    if csum_fold(s) != 0 {
        return 0;
    }
    // DAD sub-case (RFC 4861 §7.2.4): flip the NS source to the unspecified
    // address (::) and confirm the NA is now answered to the all-nodes multicast
    // (ff02::1, MAC 33:33:00:00:00:01) with Solicited cleared (Override kept) —
    // never unicast to ::.
    req[14 + 8..14 + 24].copy_from_slice(&[0u8; 16]); // IPv6 source = ::
    let mut dout = [0u8; 1514];
    let dlen = match build_neighbor_advert(&req, &mut dout) {
        Some(l) => l,
        None => return 0,
    };
    let all_nodes: [u8; 16] = [0xff, 0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x01];
    if dout[0..6] != [0x33u8, 0x33, 0x00, 0x00, 0x00, 0x01] {
        return 0; // eth dst = all-nodes multicast MAC
    }
    if dout[38..54] != all_nodes {
        return 0; // IPv6 dst = ff02::1, never ::
    }
    let dna = &dout[54..dlen];
    if dna[4] & 0x40 != 0 || dna[4] & 0x20 == 0 {
        return 0; // Solicited MUST be clear, Override kept
    }
    let mut ds = 0u32;
    csum_words(&mut ds, &g6);
    csum_words(&mut ds, &all_nodes);
    ds += u16::from_be_bytes([dout[18], dout[19]]) as u32;
    ds += 58;
    csum_words(&mut ds, dna);
    if csum_fold(ds) != 0 {
        return 0;
    }
    serial_write(b"NDP: advert ok\n");
    1
}

// IPv6 neighbor cache / NUD (full-os guide Part II.6): unlike the responder
// above (which answers a host's Neighbor Solicitation), this is the guest
// INITIATING resolution — sending its own NS for a target it wants to reach and
// caching the MAC learned from the returning Neighbor Advertisement (RFC 4861
// §7.3, Neighbor Unreachability Detection states).
const NEIGH_MAX: usize = 8;
const NUD_INCOMPLETE: u8 = 1;
const NUD_REACHABLE: u8 = 2;

#[derive(Clone, Copy)]
struct Neighbor {
    ip6: [u8; 16],
    mac: [u8; 6],
    state: u8, // 0 = free, NUD_INCOMPLETE, NUD_REACHABLE
}

static mut NEIGH_CACHE: [Neighbor; NEIGH_MAX] =
    [Neighbor { ip6: [0; 16], mac: [0; 6], state: 0 }; NEIGH_MAX];

/// Find the cache slot for `target`, or None.
unsafe fn nud_find(target: &[u8; 16]) -> Option<usize> {
    let mut i = 0;
    while i < NEIGH_MAX {
        if NEIGH_CACHE[i].state != 0 && NEIGH_CACHE[i].ip6 == *target {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Look up a resolved neighbor's MAC (REACHABLE only).
unsafe fn nud_lookup(target: &[u8; 16]) -> Option<[u8; 6]> {
    match nud_find(target) {
        Some(i) if NEIGH_CACHE[i].state == NUD_REACHABLE => Some(NEIGH_CACHE[i].mac),
        _ => None,
    }
}

/// Build a guest-originated Neighbor Solicitation to resolve `target`: dst = the
/// target's solicited-node multicast (ff02::1:ffXX:XXXX, MAC 33:33:ff:XX:XX:XX),
/// src = the guest, with the guest's Source Link-Layer Address option. Records an
/// INCOMPLETE cache entry. Returns the frame length in `out`.
unsafe fn build_neighbor_solicit(target: &[u8; 16], out: &mut [u8]) -> Option<usize> {
    let total = 14 + 40 + 32;
    if out.len() < total {
        return None;
    }
    let g6 = guest_ip6();
    let mac = net::net_mac();
    // Solicited-node multicast of the target.
    let mut snm = [0u8; 16];
    snm[0] = 0xff;
    snm[1] = 0x02;
    snm[11] = 0x01;
    snm[12] = 0xff;
    snm[13] = target[13];
    snm[14] = target[14];
    snm[15] = target[15];
    for b in out[..total].iter_mut() {
        *b = 0;
    }
    // Ethernet: dst = solicited-node multicast MAC, src = guest.
    out[0] = 0x33;
    out[1] = 0x33;
    out[2] = 0xff;
    out[3] = target[13];
    out[4] = target[14];
    out[5] = target[15];
    out[6..12].copy_from_slice(&mac);
    out[12] = 0x86;
    out[13] = 0xDD;
    {
        let ip6 = &mut out[14..54];
        ip6[0] = 0x60;
        let plen = (32u16).to_be_bytes();
        ip6[4] = plen[0];
        ip6[5] = plen[1];
        ip6[6] = 58; // ICMPv6
        ip6[7] = 255; // hop limit (RFC 4861)
        ip6[8..24].copy_from_slice(&g6);
        ip6[24..40].copy_from_slice(&snm);
    }
    {
        let ns = &mut out[54..total];
        ns[0] = 135; // Neighbor Solicitation
        ns[8..24].copy_from_slice(target); // target being resolved
        ns[24] = 1; // Source Link-Layer Address option
        ns[25] = 1;
        ns[26..32].copy_from_slice(&mac);
        let ck = icmpv6_checksum(&g6, &snm, ns);
        ns[2] = (ck >> 8) as u8;
        ns[3] = (ck & 0xFF) as u8;
    }
    // Record/refresh an INCOMPLETE entry awaiting the advertisement.
    let slot = nud_find(target).or_else(|| {
        let mut i = 0;
        while i < NEIGH_MAX {
            if NEIGH_CACHE[i].state == 0 {
                return Some(i);
            }
            i += 1;
        }
        None
    });
    if let Some(i) = slot {
        NEIGH_CACHE[i].ip6 = *target;
        NEIGH_CACHE[i].state = NUD_INCOMPLETE;
    }
    Some(total)
}

/// Ingest a received Neighbor Advertisement (type 136): learn the advertiser's
/// MAC from its Target Link-Layer Address option and mark the neighbor REACHABLE.
/// Returns true if the cache was updated.
unsafe fn nud_ingest_advert(frame: &[u8]) -> bool {
    if frame.len() < 14 + 40 + 32 {
        return false;
    }
    if u16::from_be_bytes([frame[12], frame[13]]) != 0x86DD {
        return false;
    }
    let icmp_off = 14 + 40;
    if frame[icmp_off] != 136 {
        return false; // not a Neighbor Advertisement
    }
    let target: [u8; 16] = match frame[icmp_off + 8..icmp_off + 24].try_into() {
        Ok(t) => t,
        Err(_) => return false,
    };
    // Target Link-Layer Address option (type 2, len 1) carries the MAC.
    if frame[icmp_off + 24] != 2 || frame[icmp_off + 25] != 1 {
        return false;
    }
    let mut mac = [0u8; 6];
    mac.copy_from_slice(&frame[icmp_off + 26..icmp_off + 32]);
    let slot = nud_find(&target).or_else(|| {
        let mut i = 0;
        while i < NEIGH_MAX {
            if NEIGH_CACHE[i].state == 0 {
                return Some(i);
            }
            i += 1;
        }
        None
    });
    match slot {
        Some(i) => {
            NEIGH_CACHE[i].ip6 = target;
            NEIGH_CACHE[i].mac = mac;
            NEIGH_CACHE[i].state = NUD_REACHABLE;
            true
        }
        None => false,
    }
}

/// Neighbor-cache / NUD self-test (op 14): the guest builds an NS to resolve a
/// host (INCOMPLETE, lookup misses), then a matching NA is ingested and the
/// lookup resolves to the advertised MAC (REACHABLE). Verifies the NS is
/// wire-correct (solicited-node multicast dst, guest src, SLLA option, checksum
/// folds to zero). Returns 1 on success (full-os guide Part II.6).
pub(crate) unsafe fn nud_selftest() -> u64 {
    // Clear the cache for a deterministic run.
    let mut i = 0;
    while i < NEIGH_MAX {
        NEIGH_CACHE[i].state = 0;
        i += 1;
    }
    let g6 = guest_ip6();
    let target: [u8; 16] = [
        0xfe, 0x80, 0, 0, 0, 0, 0, 0, 0x02, 0x55, 0x0a, 0xff, 0xfe, 0x00, 0x02, 0x09,
    ];
    let target_mac = [0x52u8, 0x55, 0x0a, 0x00, 0x02, 0x09];

    // 1) The guest sends an NS; the entry is INCOMPLETE so a lookup misses.
    let mut ns = [0u8; 14 + 40 + 32];
    let nlen = match build_neighbor_solicit(&target, &mut ns) {
        Some(l) => l,
        None => return 0,
    };
    if nud_lookup(&target).is_some() {
        return 0; // must be unresolved before the advertisement
    }
    // NS wire correctness: solicited-node multicast eth dst, guest src, type 135,
    // target field, SLLA = guest MAC, and a checksum that folds to zero.
    let mac = net::net_mac();
    if ns[0] != 0x33 || ns[1] != 0x33 || ns[2] != 0xff || ns[6..12] != mac {
        return 0;
    }
    if ns[54] != 135 || ns[54 + 8..54 + 24] != target || ns[54 + 24] != 1 || ns[54 + 26..54 + 32] != mac {
        return 0;
    }
    if ns[21] != 255 {
        return 0; // hop limit MUST be 255
    }
    {
        let mut snm = [0u8; 16];
        snm[0] = 0xff;
        snm[1] = 0x02;
        snm[11] = 0x01;
        snm[12] = 0xff;
        snm[13] = target[13];
        snm[14] = target[14];
        snm[15] = target[15];
        let mut s = 0u32;
        csum_words(&mut s, &g6);
        csum_words(&mut s, &snm);
        s += 32;
        s += 58;
        csum_words(&mut s, &ns[54..nlen]);
        if csum_fold(s) != 0 {
            return 0;
        }
    }

    // 2) Synthesize the target's Neighbor Advertisement and ingest it.
    let mut na = [0u8; 14 + 40 + 32];
    na[0..6].copy_from_slice(&mac); // to the guest
    na[6..12].copy_from_slice(&target_mac);
    na[12] = 0x86;
    na[13] = 0xDD;
    na[14] = 0x60;
    na[19] = 32; // payload length
    na[20] = 58; // ICMPv6
    na[21] = 255;
    na[22..38].copy_from_slice(&target); // src = target
    na[38..54].copy_from_slice(&g6); // dst = guest
    na[54] = 136; // Neighbor Advertisement
    na[54 + 4] = 0x60; // Solicited + Override
    na[54 + 8..54 + 24].copy_from_slice(&target);
    na[54 + 24] = 2; // Target Link-Layer Address option
    na[54 + 25] = 1;
    na[54 + 26..54 + 32].copy_from_slice(&target_mac);
    if !nud_ingest_advert(&na) {
        return 0;
    }

    // 3) The lookup now resolves to the advertised MAC (REACHABLE).
    match nud_lookup(&target) {
        Some(m) if m == target_mac => {
            serial_write(b"NUD: resolve ok\n");
            1
        }
        _ => 0,
    }
}

const UDP_ECHO_PORT: u16 = 7;

/// Build a UDP echo reply for a received Ethernet frame: IPv4/UDP to the guest
/// on the echo port (7). Swaps MACs/IPs/ports, copies the payload, recomputes
/// the IPv4 header checksum (UDP checksum left 0 = unused, which IPv4 permits).
/// Returns the reply length in `out`, or None (full-os guide Part II.6).
unsafe fn build_udp_echo_reply(req: &[u8], out: &mut [u8]) -> Option<usize> {
    if req.len() < 14 + 20 + 8 {
        return None;
    }
    if u16::from_be_bytes([req[12], req[13]]) != 0x0800 {
        return None;
    }
    let ip = &req[14..];
    let ihl = ((ip[0] & 0x0F) as usize) * 4;
    if ihl < 20 || ip[9] != 17 {
        return None;
    }
    if ip[16..20] != GUEST_IP {
        return None;
    }
    let udp_off = 14 + ihl;
    if req.len() < udp_off + 8 {
        return None;
    }
    let dport = u16::from_be_bytes([req[udp_off + 2], req[udp_off + 3]]);
    if dport != UDP_ECHO_PORT {
        return None;
    }
    let total = req.len();
    if total > out.len() {
        return None;
    }
    out[..total].copy_from_slice(&req[..total]);
    out[0..6].copy_from_slice(&req[6..12]);
    out[6..12].copy_from_slice(&net::net_mac());
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
    {
        let oudp = &mut out[udp_off..total];
        let sp = [oudp[0], oudp[1]];
        let dp = [oudp[2], oudp[3]];
        oudp[0] = dp[0]; // swap src/dst ports
        oudp[1] = dp[1];
        oudp[2] = sp[0];
        oudp[3] = sp[1];
        oudp[6] = 0; // UDP checksum unused
        oudp[7] = 0;
    }
    Some(total)
}

/// Live RX responder: echo UDP datagrams sent to the guest on port 7.
pub(crate) unsafe fn udp_echo_input(frame: &[u8]) {
    let mut out = [0u8; 1514];
    if let Some(len) = build_udp_echo_reply(frame, &mut out) {
        let _ = net::wire_send(&out[..len]);
        serial_write(b"UDP: echo sent\n");
    }
}

/// Self-test (op 8): synthesize a UDP datagram to port 7, run the echo
/// responder, and verify the reply swaps the endpoints and echoes the payload.
pub(crate) unsafe fn udp_echo_selftest() -> u64 {
    const PAYLOAD: &[u8; 8] = b"udp-echo";
    let mut req = [0u8; 14 + 20 + 8 + 8];
    req[0..6].copy_from_slice(&net::net_mac());
    req[6..12].copy_from_slice(&[0x52, 0x55, 0x0a, 0x00, 0x02, 0x02]);
    req[12] = 0x08;
    req[13] = 0x00;
    {
        let ip = &mut req[14..34];
        ip[0] = 0x45;
        let tot = (20 + 8 + 8) as u16;
        ip[2] = (tot >> 8) as u8;
        ip[3] = (tot & 0xFF) as u8;
        ip[8] = 64;
        ip[9] = 17;
        ip[12..16].copy_from_slice(&GATEWAY_IP);
        ip[16..20].copy_from_slice(&GUEST_IP);
        let mut s = 0u32;
        csum_words(&mut s, ip);
        let ck = csum_fold(s);
        ip[10] = (ck >> 8) as u8;
        ip[11] = (ck & 0xFF) as u8;
    }
    {
        let udp = &mut req[34..50];
        udp[0] = 0xC3;
        udp[1] = 0x50; // src port 50000
        udp[2] = 0x00;
        udp[3] = 0x07; // dst port 7
        udp[4] = 0x00;
        udp[5] = 0x10; // length 16
        udp[8..16].copy_from_slice(PAYLOAD);
    }
    let mut out = [0u8; 1514];
    let len = match build_udp_echo_reply(&req, &mut out) {
        Some(l) => l,
        None => return 0,
    };
    let oudp = &out[34..len];
    // ports swapped (src now 7) and payload echoed.
    if oudp[0] != 0x00 || oudp[1] != 0x07 || oudp[8..16] != *PAYLOAD {
        return 0;
    }
    // IPv4 header checksum of the reply folds to zero.
    let mut v = 0u32;
    csum_words(&mut v, &out[14..34]);
    if csum_fold(v) != 0 {
        return 0;
    }
    serial_write(b"UDP: echo ok\n");
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
