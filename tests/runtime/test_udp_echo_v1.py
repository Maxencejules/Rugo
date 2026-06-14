# Full-OS guide Part II.6 acceptance: UDP echo responder.
#
# At boot the kernel runs a UDP echo self-test (netcfg::udp_echo_selftest): it
# synthesizes a UDP datagram to the guest on port 7, runs the real responder
# (build_udp_echo_reply), and validates the reply swaps the endpoints and echoes
# the payload (and the IPv4 header checksum is wire-correct). The responder is
# also wired into the live RX pump, so the guest echoes real UDP on port 7.


def test_udp_echo_responder(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    find_in_order(out, [
        "UDP: echo ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "GOINIT: err" not in out
