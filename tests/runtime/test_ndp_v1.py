# Full-OS guide Part II.6 acceptance: IPv6 Neighbor Discovery (NDP).
#
# At boot the kernel runs an NDP self-test (netcfg::ndp_selftest): it synthesizes
# a Neighbor Solicitation (ICMPv6 type 135) for the guest's link-local address,
# addressed to the solicited-node multicast as a real host's NDP would be, runs
# the real responder (build_neighbor_advert), and validates the reply is a
# Neighbor Advertisement (type 136) with the Solicited+Override flags, the
# guest's address as target, a Target Link-Layer Address option carrying the
# guest MAC, and a wire-correct pseudo-header checksum. The same responder is
# wired into the live RX pump (ethertype 0x86DD, ICMPv6 type 135), so a host
# doing Neighbor Discovery can actually resolve the guest's IPv6 to its MAC.


def test_ndp_neighbor_advertisement_responder(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    find_in_order(out, [
        "NDP: advert ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "GOINIT: err" not in out
