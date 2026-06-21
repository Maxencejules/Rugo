# Full-OS guide Part II.6 acceptance: IPv6 UDP echo (IPv6 on the wire).
#
# Beyond the IPv4 UDP echo responder, the guest answers IPv6 UDP datagrams on
# port 7. At boot the kernel synthesizes an IPv6/UDP datagram to the guest's
# link-local address, runs the responder, and verifies the reply swaps the
# endpoints, echoes the payload, and carries a correct, non-zero (mandatory for
# IPv6) UDP checksum. The responder is also wired into the live RX pump.


def test_ipv6_udp_echo(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    find_in_order(out, [
        "UDP6: echo ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    # The IPv4 UDP echo + ICMPv6 responders must remain correct alongside it.
    assert "UDP: echo ok" in out
    assert "ICMPV6: echo reply ok" in out
