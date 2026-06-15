// Network device bring-up plus the runtime socket state used by go_test.

use crate::*;

#[cfg(any(feature = "net_test", feature = "go_test"))]
const VIRTIO_NET_HDR_SIZE: usize = 10;

#[cfg(any(feature = "net_test", feature = "go_test"))]
const NET_GUEST_IP: [u8; 4] = [10, 0, 2, 15];

#[cfg(any(feature = "net_test", feature = "go_test"))]
unsafe fn pci_find_virtio_net_device() -> Option<u16> {
    pci_find_virtio_legacy_iobase(0x1000)
}

#[cfg(any(feature = "net_test", feature = "go_test"))]
#[repr(C, align(4096))]
struct NetVqPages([u8; 16384]);

#[cfg(any(feature = "net_test", feature = "go_test"))]
#[repr(C, align(4096))]
struct NetBuf([u8; 4096]);

#[cfg(any(feature = "net_test", feature = "go_test"))]
static mut NET_IOBASE: u16 = 0;
#[cfg(any(feature = "net_test", feature = "go_test"))]
static mut NET_MAC: [u8; 6] = [0; 6];
#[cfg(any(feature = "net_test", feature = "go_test"))]
pub(crate) static mut NET_KV2P_DELTA: u64 = 0;

#[cfg(any(feature = "net_test", feature = "go_test"))]
static mut NET_RXQ_MEM: NetVqPages = NetVqPages([0; 16384]);
#[cfg(any(feature = "net_test", feature = "go_test"))]
static mut NET_RX_BUF: NetBuf = NetBuf([0; 4096]);
#[cfg(any(feature = "net_test", feature = "go_test"))]
static mut NET_RX_DESCS: *mut u8 = core::ptr::null_mut();
#[cfg(any(feature = "net_test", feature = "go_test"))]
static mut NET_RX_AVAIL: *mut u8 = core::ptr::null_mut();
#[cfg(any(feature = "net_test", feature = "go_test"))]
static mut NET_RX_USED: *const u8 = core::ptr::null();
#[cfg(any(feature = "net_test", feature = "go_test"))]
static mut NET_RX_LAST_USED: u16 = 0;
#[cfg(any(feature = "net_test", feature = "go_test"))]
static mut NET_RX_QSIZE: u16 = 0;

#[cfg(any(feature = "net_test", feature = "go_test"))]
static mut NET_TXQ_MEM: NetVqPages = NetVqPages([0; 16384]);
#[cfg(any(feature = "net_test", feature = "go_test"))]
static mut NET_TX_BUF: NetBuf = NetBuf([0; 4096]);
#[cfg(any(feature = "net_test", feature = "go_test"))]
static mut NET_TX_DESCS: *mut u8 = core::ptr::null_mut();
#[cfg(any(feature = "net_test", feature = "go_test"))]
static mut NET_TX_AVAIL: *mut u8 = core::ptr::null_mut();
#[cfg(any(feature = "net_test", feature = "go_test"))]
static mut NET_TX_USED: *const u8 = core::ptr::null();
#[cfg(any(feature = "net_test", feature = "go_test"))]
static mut NET_TX_LAST_USED: u16 = 0;
#[cfg(any(feature = "net_test", feature = "go_test"))]
static mut NET_TX_QSIZE: u16 = 0;

#[cfg(any(feature = "net_test", feature = "go_test"))]
unsafe fn net_kv2p(va: u64) -> u64 {
    va.wrapping_add(NET_KV2P_DELTA)
}

#[cfg(any(feature = "net_test", feature = "go_test"))]
pub(crate) unsafe fn pci_find_virtio_net() -> Option<u16> {
    pci_find_virtio_net_device()
}

#[cfg(any(feature = "net_test", feature = "go_test"))]
pub(crate) unsafe fn virtio_net_init(iobase: u16) -> bool {
    NET_IOBASE = iobase;

    outb(iobase + VIRTIO_DEVICE_STATUS, 0);
    outb(iobase + VIRTIO_DEVICE_STATUS, 1);
    outb(iobase + VIRTIO_DEVICE_STATUS, 1 | 2);
    let _features = inl(iobase + VIRTIO_DEVICE_FEATURES);
    outl(iobase + VIRTIO_GUEST_FEATURES, 0);

    for i in 0..6u16 {
        NET_MAC[i as usize] = inb(iobase + 0x14 + i);
    }

    outw(iobase + VIRTIO_QUEUE_SEL, 0);
    let rxqsz = inw(iobase + VIRTIO_QUEUE_SIZE);
    if rxqsz == 0 {
        outb(iobase + VIRTIO_DEVICE_STATUS, 0x80);
        return false;
    }
    NET_RX_QSIZE = rxqsz;
    core::ptr::write_bytes(NET_RXQ_MEM.0.as_mut_ptr(), 0, NET_RXQ_MEM.0.len());
    let rxbase = NET_RXQ_MEM.0.as_mut_ptr();
    NET_RX_DESCS = rxbase;
    let rx_avail_off = (rxqsz as usize) * 16;
    NET_RX_AVAIL = rxbase.add(rx_avail_off);
    let rx_avail_end = rx_avail_off + 6 + 2 * (rxqsz as usize);
    let rx_used_off = (rx_avail_end + 4095) & !4095;
    NET_RX_USED = rxbase.add(rx_used_off);
    NET_RX_LAST_USED = 0;
    let rxq_phys = net_kv2p(rxbase as u64);
    outl(iobase + VIRTIO_QUEUE_PFN, (rxq_phys >> 12) as u32);

    outw(iobase + VIRTIO_QUEUE_SEL, 1);
    let txqsz = inw(iobase + VIRTIO_QUEUE_SIZE);
    if txqsz == 0 {
        outb(iobase + VIRTIO_DEVICE_STATUS, 0x80);
        return false;
    }
    NET_TX_QSIZE = txqsz;
    core::ptr::write_bytes(NET_TXQ_MEM.0.as_mut_ptr(), 0, NET_TXQ_MEM.0.len());
    let txbase = NET_TXQ_MEM.0.as_mut_ptr();
    NET_TX_DESCS = txbase;
    let tx_avail_off = (txqsz as usize) * 16;
    NET_TX_AVAIL = txbase.add(tx_avail_off);
    let tx_avail_end = tx_avail_off + 6 + 2 * (txqsz as usize);
    let tx_used_off = (tx_avail_end + 4095) & !4095;
    NET_TX_USED = txbase.add(tx_used_off);
    NET_TX_LAST_USED = 0;
    let txq_phys = net_kv2p(txbase as u64);
    outl(iobase + VIRTIO_QUEUE_PFN, (txq_phys >> 12) as u32);

    outb(iobase + VIRTIO_DEVICE_STATUS, 1 | 2 | 4);
    virtio_net_post_rx();
    true
}

#[cfg(any(feature = "net_test", feature = "go_test"))]
unsafe fn virtio_net_post_rx() {
    let buf_phys = net_kv2p(NET_RX_BUF.0.as_ptr() as u64);
    let d0 = NET_RX_DESCS;
    core::ptr::write(d0.add(0) as *mut u64, buf_phys);
    core::ptr::write(d0.add(8) as *mut u32, NET_RX_BUF.0.len() as u32);
    core::ptr::write(d0.add(12) as *mut u16, VRING_DESC_F_WRITE);
    core::ptr::write(d0.add(14) as *mut u16, 0);

    let avail = NET_RX_AVAIL;
    let avail_idx = core::ptr::read_volatile((avail as *const u16).add(1));
    let qsz = NET_RX_QSIZE as usize;
    let ring_slot = (avail as *mut u16).add(2 + (avail_idx as usize % qsz));
    core::ptr::write_volatile(ring_slot, 0u16);
    core::arch::asm!("mfence", options(nostack));
    core::ptr::write_volatile((avail as *mut u16).add(1), avail_idx.wrapping_add(1));

    outw(NET_IOBASE + VIRTIO_QUEUE_NOTIFY, 0);
}

#[cfg(any(feature = "net_test", feature = "go_test"))]
pub(crate) unsafe fn virtio_net_recv(buf: &mut [u8]) -> usize {
    let used = NET_RX_USED;
    let used_idx = core::ptr::read_volatile((used as *const u16).add(1));
    if used_idx == NET_RX_LAST_USED {
        return 0;
    }
    NET_RX_LAST_USED = NET_RX_LAST_USED.wrapping_add(1);

    let qsz = NET_RX_QSIZE as usize;
    let entry_idx = (used_idx.wrapping_sub(1) as usize) % qsz;
    let entry_ptr = (used as *const u8).add(4 + entry_idx * 8);
    let total_len = core::ptr::read(entry_ptr.add(4) as *const u32) as usize;

    let _ = inb(NET_IOBASE + VIRTIO_ISR_STATUS);

    if total_len <= VIRTIO_NET_HDR_SIZE {
        virtio_net_post_rx();
        return 0;
    }
    let frame_len = total_len - VIRTIO_NET_HDR_SIZE;
    let copy_len = if frame_len > buf.len() { buf.len() } else { frame_len };
    core::ptr::copy_nonoverlapping(
        NET_RX_BUF.0.as_ptr().add(VIRTIO_NET_HDR_SIZE),
        buf.as_mut_ptr(),
        copy_len,
    );

    virtio_net_post_rx();
    copy_len
}

#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub(crate) unsafe fn net_mac() -> [u8; 6] {
    NET_MAC
}

#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub(crate) unsafe fn wire_send(frame: &[u8]) -> bool {
    virtio_net_send(frame)
}

// ---- Package fetch over TCP (full-os guide Part V.11, package manager). The
// network-download core of a package manager: connect out to a repo host, receive
// a framed package, and content-verify it (magic + checksum). Driven by the same
// start/poll pattern DHCP uses — pkg_fetch_start sends the SYN, the PIT-tick RX
// pump (net_rx_pump + tcp_rt_tick) advances the connection and accumulates the
// reply, and pkg_fetch_poll drains the wire receive buffer + verifies once the
// whole package has arrived. Package wire format: "RUGOPKG1" (8) | le32 payload
// length | payload | le32 checksum (sum of payload bytes, wrapping).
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
const PKG_MAX: usize = 1024;
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static mut PKG_BUF: [u8; PKG_MAX] = [0; PKG_MAX];
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static mut PKG_LEN: usize = 0;
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static mut PKG_FETCHING: bool = false;
// A package fetch is "armed" by a request record on disk (read at boot); the
// PIT-tick driver then starts + drives it. (Only the package-fetch test writes
// that record, so ordinary boots never attempt a fetch.)
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static mut PKG_ARMED: bool = false;
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static mut PKG_STARTED: bool = false;
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static mut PKG_REQ_PORT: u16 = 0;
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static mut PKG_TICKS: u32 = 0; // ticks since the fetch started (give-up bound)

/// Arm a package fetch for `port` (called at boot when the request record is
/// present). The PIT-tick driver (pkg_fetch_tick) then starts + completes it.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub(crate) unsafe fn pkg_fetch_arm(port: u16) {
    PKG_REQ_PORT = port;
    PKG_ARMED = true;
    PKG_STARTED = false;
}

/// Whether a package fetch is armed/in-flight (keeps the PIT pump running for it).
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub(crate) unsafe fn pkg_fetch_armed() -> bool {
    PKG_ARMED
}

/// PIT-tick driver for an armed package fetch (called from the timer handler after
/// the RX pump). Once the NIC is up it starts the fetch, then drains + verifies
/// each tick until the package arrives (pkg_fetch_poll prints the marker).
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub(crate) unsafe fn pkg_fetch_tick() {
    if !PKG_ARMED {
        return;
    }
    if !PKG_STARTED {
        if !R4_NET_NIC_READY {
            return;
        }
        PKG_STARTED = true;
        PKG_TICKS = 0;
        if pkg_fetch_start(PKG_REQ_PORT) != 0 {
            PKG_ARMED = false; // could not start; do not retry
            serial_write(b"PKG: fetch start FAIL\n");
        }
        return;
    }
    if PKG_FETCHING {
        // Patience: the gateway ARP reply + the package can take many ticks to be
        // processed (net_rx_pump drains a bounded number of frames per tick behind
        // the boot's other traffic). The wire TCP's own RTO retransmits the SYN,
        // so no connection retry is needed here; just give up after a long bound.
        PKG_TICKS = PKG_TICKS.wrapping_add(1);
        if PKG_TICKS > 2000 {
            PKG_FETCHING = false;
            PKG_ARMED = false;
            crate::tcp::tcp_close();
            serial_write(b"PKG: fetch timeout FAIL\n");
            return;
        }
        let _ = pkg_fetch_poll();
        if !PKG_FETCHING {
            PKG_ARMED = false; // completed (success or failure); stop driving
        }
    }
}

/// Begin a package fetch: connect to the slirp gateway (10.0.2.2) on `port`
/// (slirp forwards guest->10.0.2.2:port to the host repo server). Returns 0 on a
/// started fetch, or u64::MAX. The PIT-tick pump then drives the handshake.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn pkg_fetch_start(port: u16) -> u64 {
    const ERR: u64 = 0xFFFF_FFFF_FFFF_FFFF;
    if port == 0 {
        return ERR;
    }
    if crate::tcp::tcp_state() != crate::tcp::ST_CLOSED {
        crate::tcp::tcp_close();
    }
    PKG_LEN = 0;
    PKG_FETCHING = false;
    if !crate::tcp::tcp_connect([10, 0, 2, 2], port) {
        return ERR;
    }
    PKG_FETCHING = true;
    0
}

/// Poll a package fetch: drain newly received bytes, and once the whole framed
/// package has arrived, verify its magic + checksum. Returns u64::MAX while
/// pending, the payload length on success, or 0 on failure (the kernel also
/// prints a one-shot marker on completion).
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn pkg_fetch_poll() -> u64 {
    const ERR: u64 = 0xFFFF_FFFF_FFFF_FFFF;
    if !PKG_FETCHING {
        return 0;
    }
    if PKG_LEN < PKG_MAX {
        let n = crate::tcp::tcp_recv(&mut PKG_BUF[PKG_LEN..]);
        PKG_LEN += n;
    }
    if PKG_LEN >= 12 && &PKG_BUF[0..8] == b"RUGOPKG1" {
        let plen = u32::from_le_bytes([PKG_BUF[8], PKG_BUF[9], PKG_BUF[10], PKG_BUF[11]]) as usize;
        let total = 12usize.wrapping_add(plen).wrapping_add(4);
        if total > PKG_MAX {
            PKG_FETCHING = false;
            crate::tcp::tcp_close();
            serial_write(b"PKG: fetch too-big FAIL\n");
            return 0;
        }
        if PKG_LEN >= total {
            let mut sum = 0u32;
            let mut i = 0usize;
            while i < plen {
                sum = sum.wrapping_add(PKG_BUF[12 + i] as u32);
                i += 1;
            }
            let want = u32::from_le_bytes([
                PKG_BUF[12 + plen],
                PKG_BUF[13 + plen],
                PKG_BUF[14 + plen],
                PKG_BUF[15 + plen],
            ]);
            PKG_FETCHING = false;
            crate::tcp::tcp_close();
            if sum == want {
                serial_write(b"PKG: fetched len=0x");
                serial_write_hex(plen as u64);
                serial_write(b" ok\n");
                return plen as u64;
            }
            serial_write(b"PKG: fetch checksum FAIL\n");
            return 0;
        }
    }
    // The connection ended (RST / RTO give-up) before the package fully arrived.
    if crate::tcp::tcp_state() == crate::tcp::ST_CLOSED {
        PKG_FETCHING = false;
        serial_write(b"PKG: fetch closed-early FAIL\n");
        return 0;
    }
    ERR // pending
}

/// PIT-tick RX pump for the default lane: answer ARP for the guest IP,
/// hand ARP replies and IPv4/TCP packets to the TCP machine, drop the
/// rest. Runs in interrupt context (single core, IF=0 in kernel).
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub(crate) unsafe fn net_rx_pump() {
    if !R4_NET_NIC_READY {
        return;
    }
    let mut budget = 8;
    let mut buf = [0u8; 1514];
    while budget > 0 {
        budget -= 1;
        let n = virtio_net_recv(&mut buf);
        if n == 0 {
            break;
        }
        if n < 14 {
            continue;
        }
        let ethertype = u16::from_be_bytes([buf[12], buf[13]]);
        if ethertype == 0x0806 && n >= 42 {
            let arp = &buf[14..];
            let opcode = u16::from_be_bytes([arp[6], arp[7]]);
            if opcode == 2 {
                let sender_ip = [arp[14], arp[15], arp[16], arp[17]];
                crate::tcp::on_arp_reply(&sender_ip, &arp[8..14]);
                let mut mac = [0u8; 6];
                mac.copy_from_slice(&arp[8..14]);
                crate::netcfg::on_arp_reply(sender_ip, &mac);
            } else if opcode == 1 {
                // Answer "who-has GUEST_IP" so the guest is reachable.
                crate::netcfg::arp_input(&buf[..n]);
            }
            // ARP requests for the guest IP are answered by the slirp
            // gateway path only when needed; the UDP-echo lane keeps its
            // own responder.
            continue;
        }
        if ethertype == 0x0800 && n >= 34 {
            let ip = &buf[14..n];
            if ip[9] == 6 {
                crate::tcp::tcp_input(ip);
            } else if ip[9] == 17 {
                crate::netcfg::udp_input(ip);
                crate::netcfg::udp_echo_input(&buf[..n]);
            } else if ip[9] == 1 {
                crate::netcfg::icmp_input(&buf[..n]);
            }
        } else if ethertype == 0x86DD && n >= 54 {
            // IPv6: answer ICMPv6 echo requests (ping6) + UDP echo (port 7) ->
            // reachable host over IPv6. Each responder checks its own next-header.
            crate::netcfg::icmpv6_input(&buf[..n]);
            crate::netcfg::udp6_echo_input(&buf[..n]);
        }
    }
}

#[cfg(any(feature = "net_test", feature = "go_test"))]
unsafe fn virtio_net_send(frame: &[u8]) -> bool {
    let total_len = VIRTIO_NET_HDR_SIZE + frame.len();
    if total_len > NET_TX_BUF.0.len() {
        return false;
    }

    core::ptr::write_bytes(NET_TX_BUF.0.as_mut_ptr(), 0, VIRTIO_NET_HDR_SIZE);
    core::ptr::copy_nonoverlapping(
        frame.as_ptr(),
        NET_TX_BUF.0.as_mut_ptr().add(VIRTIO_NET_HDR_SIZE),
        frame.len(),
    );

    let buf_phys = net_kv2p(NET_TX_BUF.0.as_ptr() as u64);
    let qsz = NET_TX_QSIZE as usize;

    let d0 = NET_TX_DESCS;
    core::ptr::write(d0.add(0) as *mut u64, buf_phys);
    core::ptr::write(d0.add(8) as *mut u32, total_len as u32);
    core::ptr::write(d0.add(12) as *mut u16, 0);
    core::ptr::write(d0.add(14) as *mut u16, 0);

    let avail = NET_TX_AVAIL;
    let avail_idx = core::ptr::read_volatile((avail as *const u16).add(1));
    let ring_slot = (avail as *mut u16).add(2 + (avail_idx as usize % qsz));
    core::ptr::write_volatile(ring_slot, 0u16);
    core::arch::asm!("mfence", options(nostack));
    core::ptr::write_volatile((avail as *mut u16).add(1), avail_idx.wrapping_add(1));

    outw(NET_IOBASE + VIRTIO_QUEUE_NOTIFY, 1);

    let used = NET_TX_USED;
    let mut timeout: u32 = 10_000_000;
    loop {
        let idx = core::ptr::read_volatile((used as *const u16).add(1));
        if idx != NET_TX_LAST_USED {
            break;
        }
        core::arch::asm!("pause", options(nomem, nostack));
        timeout -= 1;
        if timeout == 0 {
            return false;
        }
    }
    NET_TX_LAST_USED = NET_TX_LAST_USED.wrapping_add(1);
    let _ = inb(NET_IOBASE + VIRTIO_ISR_STATUS);
    true
}

#[cfg(any(feature = "net_test", feature = "go_test"))]
pub(crate) unsafe fn sys_net_send(buf: u64, len: u64) -> u64 {
    if len == 0 || len > 1514 {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    let mut kbuf = [0u8; 1514];
    let n = len as usize;
    if copyin_user(&mut kbuf[..n], buf, n).is_err() {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    if virtio_net_send(&kbuf[..n]) {
        len
    } else {
        0xFFFF_FFFF_FFFF_FFFF
    }
}

#[cfg(any(feature = "net_test", feature = "go_test"))]
pub(crate) unsafe fn sys_net_recv(buf: u64, cap: u64) -> u64 {
    if cap == 0 || cap > 1514 {
        return 0;
    }
    let cap_n = cap as usize;
    if !user_range_ok(buf, cap_n) || !user_pages_ok(buf, cap_n, USER_PERM_WRITE) {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    let mut kbuf = [0u8; 1514];
    let n = virtio_net_recv(&mut kbuf[..cap_n]);
    if n > 0 && copyout_user(buf, &kbuf[..n], n).is_err() {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    n as u64
}

#[cfg(feature = "net_test")]
pub(crate) unsafe fn net_handle_frame(frame: &[u8]) -> bool {
    if frame.len() < 14 {
        return false;
    }
    let ethertype = u16::from_be_bytes([frame[12], frame[13]]);
    match ethertype {
        runtime::networking::ETHERTYPE_ARP => {
            net_handle_arp(frame);
            false
        }
        runtime::networking::ETHERTYPE_IPV4 => net_handle_ipv4(frame),
        _ => false,
    }
}

#[cfg(feature = "net_test")]
unsafe fn net_handle_arp(frame: &[u8]) {
    if frame.len() < 42 {
        return;
    }
    let arp = &frame[14..];
    let opcode = u16::from_be_bytes([arp[6], arp[7]]);
    if opcode != 1 {
        return;
    }
    if arp[24] != NET_GUEST_IP[0]
        || arp[25] != NET_GUEST_IP[1]
        || arp[26] != NET_GUEST_IP[2]
        || arp[27] != NET_GUEST_IP[3]
    {
        return;
    }

    let mut reply = [0u8; 42];
    reply[0..6].copy_from_slice(&frame[6..12]);
    reply[6..12].copy_from_slice(&NET_MAC);
    reply[12] = 0x08;
    reply[13] = 0x06;
    reply[14] = 0x00;
    reply[15] = 0x01;
    reply[16] = 0x08;
    reply[17] = 0x00;
    reply[18] = 6;
    reply[19] = 4;
    reply[20] = 0x00;
    reply[21] = 0x02;
    reply[22..28].copy_from_slice(&NET_MAC);
    reply[28..32].copy_from_slice(&NET_GUEST_IP);
    reply[32..38].copy_from_slice(&arp[8..14]);
    reply[38..42].copy_from_slice(&arp[14..18]);

    virtio_net_send(&reply);
}

#[cfg(feature = "net_test")]
unsafe fn net_handle_ipv4(frame: &[u8]) -> bool {
    let ip = &frame[14..];
    let ip_hdr_len = ((ip[0] & 0x0F) as usize) * 4;
    if ip_hdr_len < 20 {
        return false;
    }
    if frame.len() < 14 + ip_hdr_len + 8 {
        return false;
    }
    if ip[9] != runtime::networking::IPPROTO_UDP {
        return false;
    }
    if ip[16] != NET_GUEST_IP[0]
        || ip[17] != NET_GUEST_IP[1]
        || ip[18] != NET_GUEST_IP[2]
        || ip[19] != NET_GUEST_IP[3]
    {
        return false;
    }

    let udp = &ip[ip_hdr_len..];
    let src_port = u16::from_be_bytes([udp[0], udp[1]]);
    let dst_port = u16::from_be_bytes([udp[2], udp[3]]);
    if !runtime::networking::is_udp_echo_port(dst_port) {
        return false;
    }

    let total = frame.len();
    if total > 1514 {
        return false;
    }
    let mut reply = [0u8; 1514];
    reply[..total].copy_from_slice(&frame[..total]);

    reply[0..6].copy_from_slice(&frame[6..12]);
    reply[6..12].copy_from_slice(&NET_MAC);

    let rip = &mut reply[14..];
    let orig_src = [ip[12], ip[13], ip[14], ip[15]];
    rip[12..16].copy_from_slice(&ip[16..20]);
    rip[16..20].copy_from_slice(&orig_src);

    rip[10] = 0;
    rip[11] = 0;
    let cksum = net_ip_checksum(&rip[..ip_hdr_len]);
    rip[10] = (cksum >> 8) as u8;
    rip[11] = (cksum & 0xFF) as u8;

    let rudp = &mut rip[ip_hdr_len..];
    rudp[0] = (dst_port >> 8) as u8;
    rudp[1] = (dst_port & 0xFF) as u8;
    rudp[2] = (src_port >> 8) as u8;
    rudp[3] = (src_port & 0xFF) as u8;
    rudp[6] = 0;
    rudp[7] = 0;

    serial_write(b"NET: udp echo\n");
    virtio_net_send(&reply[..total]);
    true
}

#[cfg(feature = "net_test")]
fn net_ip_checksum(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut i = 0;
    while i + 1 < data.len() {
        sum += u16::from_be_bytes([data[i], data[i + 1]]) as u32;
        i += 2;
    }
    if i < data.len() {
        sum += (data[i] as u32) << 8;
    }
    while sum > 0xFFFF {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}

#[cfg(feature = "go_test")]
const R4_NET_AF_INET: u64 = 2;
#[cfg(feature = "go_test")]
const R4_NET_AF_INET6: u64 = 10;
#[cfg(feature = "go_test")]
const R4_NET_SOCK_STREAM: u64 = 1;
#[cfg(feature = "go_test")]
const R4_NET_IF_MAX: usize = 2;
#[cfg(feature = "go_test")]
const R4_NET_ROUTE_MAX: usize = 8;
#[cfg(feature = "go_test")]
pub(crate) const R4_NET_SOCKET_MAX: usize = 16;
#[cfg(feature = "go_test")]
const R4_NET_RX_MAX: usize = 256;

#[cfg(feature = "go_test")]
#[derive(Clone, Copy)]
struct R4NetInterface {
    active: bool,
    has_ipv4: bool,
    ipv4: [u8; 4],
    ipv4_prefix: u8,
    has_ipv6: bool,
    ipv6: [u8; 16],
    ipv6_prefix: u8,
}

#[cfg(feature = "go_test")]
impl R4NetInterface {
    const EMPTY: Self = Self {
        active: false,
        has_ipv4: false,
        ipv4: [0; 4],
        ipv4_prefix: 0,
        has_ipv6: false,
        ipv6: [0; 16],
        ipv6_prefix: 0,
    };
}

#[cfg(feature = "go_test")]
#[derive(Clone, Copy)]
struct R4NetRoute {
    active: bool,
    family: u8,
    prefix_len: u8,
    if_index: u8,
    dest: [u8; 16],
}

#[cfg(feature = "go_test")]
impl R4NetRoute {
    const EMPTY: Self = Self {
        active: false,
        family: 0,
        prefix_len: 0,
        if_index: 0,
        dest: [0; 16],
    };
}

#[cfg(feature = "go_test")]
#[derive(Clone, Copy)]
struct R4Socket {
    active: bool,
    owner_tid: usize,
    domain: u8,
    kind: u8,
    state: u8,
    if_index: u8,
    backlog: u8,
    peer: i16,
    pending_accept: i16,
    local_port: u16,
    remote_port: u16,
    local_addr: [u8; 16],
    remote_addr: [u8; 16],
    rx_len: usize,
    rx_buf: [u8; R4_NET_RX_MAX],
}

#[cfg(feature = "go_test")]
impl R4Socket {
    const EMPTY: Self = Self {
        active: false,
        owner_tid: 0,
        domain: 0,
        kind: 0,
        state: 0,
        if_index: 0,
        backlog: 0,
        peer: -1,
        pending_accept: -1,
        local_port: 0,
        remote_port: 0,
        local_addr: [0; 16],
        remote_addr: [0; 16],
        rx_len: 0,
        rx_buf: [0; R4_NET_RX_MAX],
    };
}

#[cfg(feature = "go_test")]
static mut R4_NET_INTERFACES: [R4NetInterface; R4_NET_IF_MAX] =
    [R4NetInterface::EMPTY; R4_NET_IF_MAX];
#[cfg(feature = "go_test")]
static mut R4_NET_ROUTES: [R4NetRoute; R4_NET_ROUTE_MAX] =
    [R4NetRoute::EMPTY; R4_NET_ROUTE_MAX];
#[cfg(feature = "go_test")]
static mut R4_SOCKETS: [R4Socket; R4_NET_SOCKET_MAX] =
    [R4Socket::EMPTY; R4_NET_SOCKET_MAX];
#[cfg(feature = "go_test")]
pub(crate) static mut R4_NET_NIC_READY: bool = false;

#[cfg(feature = "go_test")]
pub(crate) unsafe fn r4_net_reset(has_nic: bool) {
    R4_NET_NIC_READY = has_nic;
    for idx in 0..R4_NET_IF_MAX {
        R4_NET_INTERFACES[idx] = R4NetInterface::EMPTY;
    }
    for idx in 0..R4_NET_ROUTE_MAX {
        R4_NET_ROUTES[idx] = R4NetRoute::EMPTY;
    }
    for idx in 0..R4_NET_SOCKET_MAX {
        R4_SOCKETS[idx] = R4Socket::EMPTY;
    }
    R4_NET_INTERFACES[0].active = true;
    R4_NET_INTERFACES[0].has_ipv4 = true;
    R4_NET_INTERFACES[0].ipv4 = [127, 0, 0, 1];
    R4_NET_INTERFACES[0].ipv4_prefix = 8;
    R4_NET_INTERFACES[0].has_ipv6 = true;
    R4_NET_INTERFACES[0].ipv6 = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
    R4_NET_INTERFACES[0].ipv6_prefix = 128;
    if has_nic {
        R4_NET_INTERFACES[1].active = true;
    }
}

#[cfg(feature = "go_test")]
fn r4_net_family_ok(family: u8) -> bool {
    family as u64 == R4_NET_AF_INET || family as u64 == R4_NET_AF_INET6
}

#[cfg(feature = "go_test")]
fn r4_net_prefix_ok(family: u8, prefix: u8) -> bool {
    if family as u64 == R4_NET_AF_INET {
        prefix <= 32
    } else if family as u64 == R4_NET_AF_INET6 {
        prefix <= 128
    } else {
        false
    }
}

#[cfg(feature = "go_test")]
fn r4_net_prefix_match(family: u8, left: &[u8; 16], right: &[u8; 16], prefix: u8) -> bool {
    let total_bits = if family as u64 == R4_NET_AF_INET { 32 } else { 128 };
    if prefix > total_bits {
        return false;
    }
    let compare_len = if family as u64 == R4_NET_AF_INET { 4 } else { 16 };
    let mut bits_left = prefix as usize;
    for idx in 0..compare_len {
        if bits_left == 0 {
            return true;
        }
        let bit_count = if bits_left >= 8 { 8 } else { bits_left };
        let mask = (!0u8) << (8 - bit_count);
        if (left[idx] & mask) != (right[idx] & mask) {
            return false;
        }
        bits_left -= bit_count;
    }
    true
}

#[cfg(feature = "go_test")]
unsafe fn r4_net_copy_record(ptr: u64, len: u64, out: &mut [u8]) -> bool {
    if len < out.len() as u64 {
        return false;
    }
    copyin_user(out, ptr, out.len()).is_ok()
}

#[cfg(feature = "go_test")]
unsafe fn r4_net_copy_sockaddr(ptr: u64, len: u64) -> Option<(u8, u16, [u8; 16])> {
    let mut raw = [0u8; 32];
    if !r4_net_copy_record(ptr, len, &mut raw) {
        return None;
    }
    let family = u64::from_le_bytes(raw[0..8].try_into().ok()?) as u8;
    let port = u64::from_le_bytes(raw[8..16].try_into().ok()?) as u16;
    if !r4_net_family_ok(family) || port == 0 {
        return None;
    }
    let mut addr = [0u8; 16];
    addr.copy_from_slice(&raw[16..32]);
    Some((family, port, addr))
}

#[cfg(feature = "go_test")]
unsafe fn r4_net_copy_cfg(ptr: u64, len: u64) -> Option<(u8, u8, [u8; 16])> {
    let mut raw = [0u8; 32];
    if !r4_net_copy_record(ptr, len, &mut raw) {
        return None;
    }
    let family = u64::from_le_bytes(raw[0..8].try_into().ok()?) as u8;
    let prefix_len = u64::from_le_bytes(raw[8..16].try_into().ok()?) as u8;
    if !r4_net_family_ok(family) || !r4_net_prefix_ok(family, prefix_len) {
        return None;
    }
    let mut addr = [0u8; 16];
    addr.copy_from_slice(&raw[16..32]);
    Some((family, prefix_len, addr))
}

#[cfg(feature = "go_test")]
unsafe fn r4_net_interface_addr(family: u8, if_index: usize) -> Option<[u8; 16]> {
    if if_index >= R4_NET_IF_MAX || !R4_NET_INTERFACES[if_index].active {
        return None;
    }
    if family as u64 == R4_NET_AF_INET {
        if !R4_NET_INTERFACES[if_index].has_ipv4 {
            return None;
        }
        let mut out = [0u8; 16];
        out[0..4].copy_from_slice(&R4_NET_INTERFACES[if_index].ipv4);
        return Some(out);
    }
    if !R4_NET_INTERFACES[if_index].has_ipv6 {
        return None;
    }
    Some(R4_NET_INTERFACES[if_index].ipv6)
}

#[cfg(feature = "go_test")]
unsafe fn r4_net_addr_on_interface(family: u8, if_index: usize, addr: &[u8; 16]) -> bool {
    match r4_net_interface_addr(family, if_index) {
        Some(bound) => bound == *addr,
        None => false,
    }
}

#[cfg(feature = "go_test")]
unsafe fn r4_net_find_route(family: u8, dest: &[u8; 16]) -> Option<usize> {
    let mut best: Option<usize> = None;
    let mut best_prefix = 0u8;
    for idx in 0..R4_NET_ROUTE_MAX {
        if !R4_NET_ROUTES[idx].active || R4_NET_ROUTES[idx].family != family {
            continue;
        }
        if !r4_net_prefix_match(
            family,
            &R4_NET_ROUTES[idx].dest,
            dest,
            R4_NET_ROUTES[idx].prefix_len,
        ) {
            continue;
        }
        if best.is_none() || R4_NET_ROUTES[idx].prefix_len >= best_prefix {
            best = Some(idx);
            best_prefix = R4_NET_ROUTES[idx].prefix_len;
        }
    }
    best
}

/// Routing-table self-test (full-os guide Part II.6): install overlapping routes
/// and confirm `r4_net_find_route` selects the LONGEST-prefix match for several
/// destinations. Saves/restores the live route table so it leaves no residue.
/// Returns 1 on success.
#[cfg(feature = "go_test")]
pub(crate) unsafe fn route_selftest() -> u64 {
    let saved = R4_NET_ROUTES;
    let mut i = 0;
    while i < R4_NET_ROUTE_MAX {
        R4_NET_ROUTES[i] = R4NetRoute::EMPTY;
        i += 1;
    }
    let af = R4_NET_AF_INET as u8;
    let route = |prefix: u8, d: [u8; 4]| -> R4NetRoute {
        let mut dest = [0u8; 16];
        dest[0] = d[0];
        dest[1] = d[1];
        dest[2] = d[2];
        dest[3] = d[3];
        R4NetRoute { active: true, family: af, prefix_len: prefix, if_index: 0, dest }
    };
    // Deliberately out of prefix order, so success depends on longest-match, not
    // table position: 10.0.2.0/24 (most specific), 10.0.0.0/8, 0.0.0.0/0.
    R4_NET_ROUTES[0] = route(0, [0, 0, 0, 0]);
    R4_NET_ROUTES[1] = route(8, [10, 0, 0, 0]);
    R4_NET_ROUTES[2] = route(24, [10, 0, 2, 0]);

    let prefix_for = |d: [u8; 4]| -> i32 {
        let mut dest = [0u8; 16];
        dest[0] = d[0];
        dest[1] = d[1];
        dest[2] = d[2];
        dest[3] = d[3];
        unsafe {
            match r4_net_find_route(af, &dest) {
                Some(idx) => R4_NET_ROUTES[idx].prefix_len as i32,
                None => -1,
            }
        }
    };
    let ok = prefix_for([10, 0, 2, 5]) == 24   // matches /24 over /8 and default
        && prefix_for([10, 5, 5, 5]) == 8       // matches /8 over default
        && prefix_for([8, 8, 8, 8]) == 0;       // only the default matches

    R4_NET_ROUTES = saved;
    if ok {
        serial_write(b"ROUTE: selftest ok\n");
        1
    } else {
        serial_write(b"ROUTE: selftest fail\n");
        0
    }
}

#[cfg(feature = "go_test")]
unsafe fn r4_net_alloc_socket() -> Option<usize> {
    for idx in 0..R4_NET_SOCKET_MAX {
        if !R4_SOCKETS[idx].active {
            R4_SOCKETS[idx] = R4Socket::EMPTY;
            R4_SOCKETS[idx].active = true;
            return Some(idx);
        }
    }
    None
}

#[cfg(feature = "go_test")]
#[inline(always)]
unsafe fn r4_socket_owner_ok(socket_id: usize) -> bool {
    socket_id < R4_NET_SOCKET_MAX
        && R4_SOCKETS[socket_id].active
        && R4_SOCKETS[socket_id].owner_tid == R4_CURRENT
}

#[cfg(feature = "go_test")]
unsafe fn r4_release_socket(socket_id: usize) {
    if socket_id >= R4_NET_SOCKET_MAX || !R4_SOCKETS[socket_id].active {
        return;
    }

    let owner_tid = R4_SOCKETS[socket_id].owner_tid;
    let peer = R4_SOCKETS[socket_id].peer;
    let pending_accept = R4_SOCKETS[socket_id].pending_accept;

    R4_SOCKETS[socket_id] = R4Socket::EMPTY;

    if owner_tid < R4_NUM_TASKS && R4_TASKS[owner_tid].socket_count != 0 {
        R4_TASKS[owner_tid].socket_count -= 1;
    }

    if peer >= 0 {
        let pid = peer as usize;
        if pid < R4_NET_SOCKET_MAX
            && R4_SOCKETS[pid].active
            && R4_SOCKETS[pid].peer == socket_id as i16
        {
            R4_SOCKETS[pid].peer = -1;
        }
    }

    if pending_accept >= 0 {
        let acc = pending_accept as usize;
        if acc < R4_NET_SOCKET_MAX && R4_SOCKETS[acc].active {
            r4_release_socket(acc);
        }
    }
}

#[cfg(feature = "go_test")]
pub(crate) unsafe fn r4_release_owned_sockets(owner_tid: usize) {
    for socket_id in 0..R4_NET_SOCKET_MAX {
        if R4_SOCKETS[socket_id].active && R4_SOCKETS[socket_id].owner_tid == owner_tid {
            r4_release_socket(socket_id);
        }
    }
    if owner_tid < R4_NUM_TASKS {
        R4_TASKS[owner_tid].socket_count = 0;
    }
}

#[cfg(feature = "go_test")]
unsafe fn r4_net_port_in_use(family: u8, addr: &[u8; 16], port: u16) -> bool {
    for idx in 0..R4_NET_SOCKET_MAX {
        if !R4_SOCKETS[idx].active {
            continue;
        }
        if R4_SOCKETS[idx].domain == family
            && R4_SOCKETS[idx].local_port == port
            && R4_SOCKETS[idx].local_addr == *addr
            && R4_SOCKETS[idx].state >= 2
        {
            return true;
        }
    }
    false
}

#[cfg(feature = "go_test")]
unsafe fn r4_net_find_listener(
    family: u8,
    addr: &[u8; 16],
    port: u16,
    if_index: u8,
) -> Option<usize> {
    for idx in 0..R4_NET_SOCKET_MAX {
        if !R4_SOCKETS[idx].active {
            continue;
        }
        if R4_SOCKETS[idx].domain != family
            || R4_SOCKETS[idx].kind as u64 != R4_NET_SOCK_STREAM
            || R4_SOCKETS[idx].state != 3
        {
            continue;
        }
        if R4_SOCKETS[idx].local_port == port
            && R4_SOCKETS[idx].local_addr == *addr
            && R4_SOCKETS[idx].if_index == if_index
        {
            return Some(idx);
        }
    }
    None
}

#[cfg(feature = "go_test")]
pub(crate) unsafe fn sys_socket_open_r4(domain: u64, kind: u64) -> u64 {
    let dom = domain as u8;
    let typ = kind as u8;
    if !r4_net_family_ok(dom) || kind != R4_NET_SOCK_STREAM {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    if !r4_current_has_cap(R4_TASK_CAP_NETWORK) {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    if !runtime::isolation::under_quota(
        R4_TASKS[R4_CURRENT].socket_count,
        R4_TASKS[R4_CURRENT].socket_limit as usize,
    ) {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    match r4_net_alloc_socket() {
        Some(idx) => {
            R4_SOCKETS[idx].owner_tid = R4_CURRENT;
            R4_SOCKETS[idx].domain = dom;
            R4_SOCKETS[idx].kind = typ;
            R4_SOCKETS[idx].state = 1;
            R4_TASKS[R4_CURRENT].socket_count += 1;
            idx as u64
        }
        None => 0xFFFF_FFFF_FFFF_FFFF,
    }
}

#[cfg(feature = "go_test")]
pub(crate) unsafe fn sys_socket_bind_r4(socket_id: u64, addr_ptr: u64, addr_len: u64) -> u64 {
    let sid = socket_id as usize;
    if sid >= R4_NET_SOCKET_MAX || !R4_SOCKETS[sid].active {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    if !r4_socket_owner_ok(sid) {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    let (family, port, addr) = match r4_net_copy_sockaddr(addr_ptr, addr_len) {
        Some(v) => v,
        None => return 0xFFFF_FFFF_FFFF_FFFF,
    };
    if R4_SOCKETS[sid].domain != family || !r4_net_family_ok(family) {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    let mut found_if = None;
    for if_index in 0..R4_NET_IF_MAX {
        if r4_net_addr_on_interface(family, if_index, &addr) {
            found_if = Some(if_index as u8);
            break;
        }
    }
    let if_index = match found_if {
        Some(idx) => idx,
        None => return 0xFFFF_FFFF_FFFF_FFFF,
    };
    if r4_net_port_in_use(family, &addr, port) {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    R4_SOCKETS[sid].if_index = if_index;
    R4_SOCKETS[sid].local_addr = addr;
    R4_SOCKETS[sid].local_port = port;
    R4_SOCKETS[sid].state = 2;
    0
}

#[cfg(feature = "go_test")]
pub(crate) unsafe fn sys_socket_listen_r4(socket_id: u64, backlog: u64) -> u64 {
    let sid = socket_id as usize;
    if sid >= R4_NET_SOCKET_MAX || !R4_SOCKETS[sid].active {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    if !r4_socket_owner_ok(sid) {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    if R4_SOCKETS[sid].kind as u64 != R4_NET_SOCK_STREAM || R4_SOCKETS[sid].state != 2 {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    R4_SOCKETS[sid].backlog = if backlog == 0 { 1 } else { backlog as u8 };
    R4_SOCKETS[sid].pending_accept = -1;
    R4_SOCKETS[sid].state = 3;
    0
}

#[cfg(feature = "go_test")]
pub(crate) unsafe fn sys_socket_connect_r4(socket_id: u64, addr_ptr: u64, addr_len: u64) -> u64 {
    let sid = socket_id as usize;
    if sid >= R4_NET_SOCKET_MAX || !R4_SOCKETS[sid].active {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    if !r4_socket_owner_ok(sid) {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    let (family, port, addr) = match r4_net_copy_sockaddr(addr_ptr, addr_len) {
        Some(v) => v,
        None => return 0xFFFF_FFFF_FFFF_FFFF,
    };
    if R4_SOCKETS[sid].domain != family || R4_SOCKETS[sid].kind as u64 != R4_NET_SOCK_STREAM {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    // AF_INET stream connects go to the wire TCP machine (gap item 6);
    // the loopback rendezvous below stays the AF_INET6 test surface.
    #[cfg(not(feature = "compat_real_test"))]
    {
        if family as u64 == R4_NET_AF_INET {
            if !R4_NET_NIC_READY {
                return 0xFFFF_FFFF_FFFF_FFFF;
            }
            let dst = [addr[0], addr[1], addr[2], addr[3]];
            if !crate::tcp::tcp_connect(dst, port) {
                return 0xFFFF_FFFF_FFFF_FFFF;
            }
            R4_SOCKETS[sid].state = 8;
            R4_SOCKETS[sid].remote_port = port;
            R4_SOCKETS[sid].remote_addr[..4].copy_from_slice(&dst);
            net_rx_pump();
            return 0;
        }
    }
    let route_idx = match r4_net_find_route(family, &addr) {
        Some(idx) => idx,
        None => return 0xFFFF_FFFF_FFFF_FFFF,
    };
    let if_index = R4_NET_ROUTES[route_idx].if_index;
    let listener = match r4_net_find_listener(family, &addr, port, if_index) {
        Some(idx) => idx,
        None => return 0xFFFF_FFFF_FFFF_FFFF,
    };
    if R4_SOCKETS[listener].pending_accept >= 0 {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    let listener_owner = R4_SOCKETS[listener].owner_tid;
    if listener_owner >= R4_NUM_TASKS
        || !runtime::isolation::under_quota(
            R4_TASKS[listener_owner].socket_count,
            R4_TASKS[listener_owner].socket_limit as usize,
        )
    {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    let accepted = match r4_net_alloc_socket() {
        Some(idx) => idx,
        None => return 0xFFFF_FFFF_FFFF_FFFF,
    };
    let local_addr = match r4_net_interface_addr(family, if_index as usize) {
        Some(addr) => addr,
        None => return 0xFFFF_FFFF_FFFF_FFFF,
    };
    let local_port = if R4_SOCKETS[sid].local_port == 0 {
        40000u16.wrapping_add(sid as u16)
    } else {
        R4_SOCKETS[sid].local_port
    };
    R4_SOCKETS[sid].if_index = if_index;
    R4_SOCKETS[sid].local_addr = local_addr;
    R4_SOCKETS[sid].local_port = local_port;
    R4_SOCKETS[sid].remote_addr = addr;
    R4_SOCKETS[sid].remote_port = port;
    R4_SOCKETS[sid].peer = accepted as i16;
    R4_SOCKETS[sid].state = 4;

    R4_SOCKETS[accepted].owner_tid = listener_owner;
    R4_SOCKETS[accepted].domain = family;
    R4_SOCKETS[accepted].kind = R4_NET_SOCK_STREAM as u8;
    R4_SOCKETS[accepted].state = 4;
    R4_SOCKETS[accepted].if_index = if_index;
    R4_SOCKETS[accepted].local_addr = addr;
    R4_SOCKETS[accepted].local_port = port;
    R4_SOCKETS[accepted].remote_addr = local_addr;
    R4_SOCKETS[accepted].remote_port = local_port;
    R4_SOCKETS[accepted].peer = sid as i16;
    R4_TASKS[listener_owner].socket_count += 1;

    R4_SOCKETS[listener].pending_accept = accepted as i16;
    0
}

#[cfg(feature = "go_test")]
pub(crate) unsafe fn sys_socket_accept_r4(
    socket_id: u64,
    addr_ptr: u64,
    addr_len_ptr: u64,
) -> u64 {
    let sid = socket_id as usize;
    if sid >= R4_NET_SOCKET_MAX || !R4_SOCKETS[sid].active || R4_SOCKETS[sid].state != 3 {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    if !r4_socket_owner_ok(sid) {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    let accepted = R4_SOCKETS[sid].pending_accept;
    if accepted < 0 {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    let acc = accepted as usize;
    let mut raw = [0u8; 32];
    raw[0..8].copy_from_slice(&(R4_SOCKETS[acc].domain as u64).to_le_bytes());
    raw[8..16].copy_from_slice(&(R4_SOCKETS[acc].remote_port as u64).to_le_bytes());
    raw[16..32].copy_from_slice(&R4_SOCKETS[acc].remote_addr);
    if copyout_user(addr_ptr, &raw, raw.len()).is_err() {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    if copyout_user(addr_len_ptr, &(32u64).to_le_bytes(), 8).is_err() {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    R4_SOCKETS[sid].pending_accept = -1;
    acc as u64
}

#[cfg(feature = "go_test")]
pub(crate) unsafe fn sys_socket_send_r4(socket_id: u64, buf: u64, len: u64) -> u64 {
    let sid = socket_id as usize;
    if sid >= R4_NET_SOCKET_MAX || !R4_SOCKETS[sid].active {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    #[cfg(not(feature = "compat_real_test"))]
    {
        if R4_SOCKETS[sid].state == 8 {
            if !r4_socket_owner_ok(sid) {
                return 0xFFFF_FFFF_FFFF_FFFF;
            }
            net_rx_pump();
            let n = len as usize;
            if n == 0 || n > 512 {
                return 0xFFFF_FFFF_FFFF_FFFF;
            }
            let mut kbuf = [0u8; 512];
            if copyin_user(&mut kbuf[..n], buf, n).is_err() {
                return 0xFFFF_FFFF_FFFF_FFFF;
            }
            let sent = crate::tcp::tcp_send(&kbuf[..n]);
            if sent == 0 {
                return 0xFFFF_FFFF_FFFF_FFFF;
            }
            return sent as u64;
        }
    }
    if R4_SOCKETS[sid].state != 4 {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    if !r4_socket_owner_ok(sid) {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    let peer = R4_SOCKETS[sid].peer;
    if peer < 0 {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    let pid = peer as usize;
    if pid >= R4_NET_SOCKET_MAX || !R4_SOCKETS[pid].active {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    let n = len as usize;
    if n == 0 || n > R4_NET_RX_MAX || R4_SOCKETS[pid].rx_len != 0 {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    if copyin_user(&mut R4_SOCKETS[pid].rx_buf[..n], buf, n).is_err() {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    R4_SOCKETS[pid].rx_len = n;
    len
}

#[cfg(feature = "go_test")]
pub(crate) unsafe fn sys_socket_recv_r4(socket_id: u64, buf: u64, len: u64) -> u64 {
    let sid = socket_id as usize;
    if sid >= R4_NET_SOCKET_MAX || !R4_SOCKETS[sid].active {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    #[cfg(not(feature = "compat_real_test"))]
    {
        if R4_SOCKETS[sid].state == 8 {
            if !r4_socket_owner_ok(sid) {
                return 0xFFFF_FFFF_FFFF_FFFF;
            }
            net_rx_pump();
            let cap = (len as usize).min(512);
            let mut kbuf = [0u8; 512];
            let n = crate::tcp::tcp_recv(&mut kbuf[..cap]);
            if n == 0 {
                return 0xFFFF_FFFF_FFFF_FFFF;
            }
            if copyout_user(buf, &kbuf[..n], n).is_err() {
                return 0xFFFF_FFFF_FFFF_FFFF;
            }
            return n as u64;
        }
    }
    if R4_SOCKETS[sid].state != 4 {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    if !r4_socket_owner_ok(sid) {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    let n = R4_SOCKETS[sid].rx_len;
    if n == 0 || len < n as u64 {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    if copyout_user(buf, &R4_SOCKETS[sid].rx_buf[..n], n).is_err() {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    R4_SOCKETS[sid].rx_len = 0;
    n as u64
}

#[cfg(feature = "go_test")]
pub(crate) unsafe fn sys_socket_close_r4(socket_id: u64) -> u64 {
    let sid = socket_id as usize;
    if sid >= R4_NET_SOCKET_MAX || !R4_SOCKETS[sid].active {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    if !r4_socket_owner_ok(sid) {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    #[cfg(not(feature = "compat_real_test"))]
    {
        if R4_SOCKETS[sid].state == 8 {
            crate::tcp::tcp_close();
        }
    }
    r4_release_socket(sid);
    0
}

#[cfg(feature = "go_test")]
pub(crate) unsafe fn sys_net_if_config_r4(if_index: u64, cfg_ptr: u64, cfg_len: u64) -> u64 {
    if !r4_current_has_cap(R4_TASK_CAP_NETWORK) {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    let idx = if_index as usize;
    if idx >= R4_NET_IF_MAX || !R4_NET_INTERFACES[idx].active {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    let (family, prefix_len, addr) = match r4_net_copy_cfg(cfg_ptr, cfg_len) {
        Some(v) => v,
        None => return 0xFFFF_FFFF_FFFF_FFFF,
    };
    if family as u64 == R4_NET_AF_INET {
        R4_NET_INTERFACES[idx].has_ipv4 = true;
        R4_NET_INTERFACES[idx].ipv4.copy_from_slice(&addr[0..4]);
        R4_NET_INTERFACES[idx].ipv4_prefix = prefix_len;
        return 0;
    }
    R4_NET_INTERFACES[idx].has_ipv6 = true;
    R4_NET_INTERFACES[idx].ipv6 = addr;
    R4_NET_INTERFACES[idx].ipv6_prefix = prefix_len;
    0
}

#[cfg(feature = "go_test")]
pub(crate) unsafe fn sys_net_route_add_r4(if_index: u64, route_ptr: u64, route_len: u64) -> u64 {
    if !r4_current_has_cap(R4_TASK_CAP_NETWORK) {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    let idx = if_index as usize;
    if idx >= R4_NET_IF_MAX || !R4_NET_INTERFACES[idx].active {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    let (family, prefix_len, dest) = match r4_net_copy_cfg(route_ptr, route_len) {
        Some(v) => v,
        None => return 0xFFFF_FFFF_FFFF_FFFF,
    };
    if r4_net_interface_addr(family, idx).is_none() {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    for slot in 0..R4_NET_ROUTE_MAX {
        if R4_NET_ROUTES[slot].active
            && R4_NET_ROUTES[slot].family == family
            && R4_NET_ROUTES[slot].prefix_len == prefix_len
            && R4_NET_ROUTES[slot].if_index == idx as u8
            && R4_NET_ROUTES[slot].dest == dest
        {
            return 0;
        }
    }
    for slot in 0..R4_NET_ROUTE_MAX {
        if !R4_NET_ROUTES[slot].active {
            R4_NET_ROUTES[slot].active = true;
            R4_NET_ROUTES[slot].family = family;
            R4_NET_ROUTES[slot].prefix_len = prefix_len;
            R4_NET_ROUTES[slot].if_index = idx as u8;
            R4_NET_ROUTES[slot].dest = dest;
            return 0;
        }
    }
    0xFFFF_FFFF_FFFF_FFFF
}

#[cfg(feature = "go_test")]
pub(crate) unsafe fn r4_c4_runtime_init() {
    storage::r4_storage_boot_probe();

    let hhdm_resp_ptr = core::ptr::read_volatile(core::ptr::addr_of!(HHDM_REQUEST.response));
    let kaddr_resp_ptr = core::ptr::read_volatile(core::ptr::addr_of!(KADDR_REQUEST.response));
    let kphys = (*kaddr_resp_ptr).physical_base;
    let kvirt = (*kaddr_resp_ptr).virtual_base;
    let _hhdm = (*hhdm_resp_ptr).offset;
    NET_KV2P_DELTA = kphys.wrapping_sub(kvirt);

    let mut nic_ready = false;
    if let Some(iobase) = pci_find_virtio_net() {
        if virtio_net_init(iobase) {
            nic_ready = true;
            serial_write(b"NETC4: nic ready\n");
        }
    }
    r4_net_reset(nic_ready);
}
