# Full-OS guide Part II.6 acceptance: IPv6 SLAAC (stateless autoconfiguration).
#
# At boot the guest builds a Router Solicitation (verified wire-correct: all-routers
# multicast dst, guest src, SLLA option, hop limit 255, checksum folds to zero),
# then processes a synthetic Router Advertisement carrying a 2001:db8::/64 Prefix
# Information option and derives its global address = the announced /64 prefix +
# the interface EUI-64. This is the guest configuring a routable IPv6 address.


def test_ipv6_slaac(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    find_in_order(out, [
        "SLAAC: global ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
