# Full-OS guide Part II.6 acceptance: IPv6 / ICMPv6 echo responder.
#
# At boot the kernel runs an ICMPv6 self-test (netcfg::icmpv6_selftest): it
# synthesizes an ICMPv6 echo request (type 128) to the guest's link-local
# address, runs the real responder (build_icmpv6_echo_reply), and validates the
# reply is type 129 with a wire-correct pseudo-header checksum and the echoed
# payload. The same responder is wired into the live RX pump (ethertype 0x86DD),
# so the guest answers real ping6 to its link-local address.


def test_icmpv6_echo_responder(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    find_in_order(out, [
        "ICMPV6: echo reply ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "GOINIT: err" not in out
