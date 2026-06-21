# Full-OS guide Part II.6 acceptance: routing table longest-prefix-match.
#
# At boot the kernel runs a routing self-test: it installs overlapping IPv4 routes
# (0.0.0.0/0, 10.0.0.0/8, 10.0.2.0/24) deliberately out of prefix order, then
# confirms r4_net_find_route selects the LONGEST-prefix match for each
# destination (10.0.2.5 -> /24, 10.5.5.5 -> /8, 8.8.8.8 -> default). The live
# route table is saved and restored so the test leaves no residue.


def test_routing_longest_prefix_match(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    find_in_order(out, [
        "ROUTE: selftest ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "ROUTE: selftest fail" not in out
