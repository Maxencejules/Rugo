# Full-OS guide Part II.6 acceptance: ARP responder.
#
# At boot the kernel runs an ARP self-test (netcfg::arp_selftest): it
# synthesizes a "who-has GUEST_IP" request, runs the real responder
# (build_arp_reply), and validates the reply (opcode 2, sender = our MAC/IP,
# target = the requester). The same responder is wired into the live RX pump
# (net_rx_pump, ARP opcode 1), so the guest answers real ARP requests and is a
# reachable host.


def test_arp_responder(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    find_in_order(out, [
        "ARP: reply ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "GOINIT: err" not in out
