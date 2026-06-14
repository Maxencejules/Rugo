# Full-OS guide Part II.6 acceptance: ICMP echo responder.
#
# At boot the kernel runs an ICMP self-test (netcfg::icmp_selftest): it
# synthesizes an echo request, runs the real responder (build_icmp_echo_reply),
# and validates the reply's IP/ICMP checksums and the echoed ident/seq/payload.
# The same responder is wired into the live RX pump (net_rx_pump, proto 1), so
# the guest also answers real inbound pings.


def test_icmp_echo_responder(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    find_in_order(out, [
        "ICMP: echo reply ok seq=0x0000000000000001",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "GOINIT: err" not in out
