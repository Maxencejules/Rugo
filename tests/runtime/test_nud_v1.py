# Full-OS guide Part II.6 acceptance: IPv6 neighbor cache + NUD (guest-initiated).
#
# Unlike the NDP responder (which answers a host's Neighbor Solicitation), this
# is the guest INITIATING resolution. At boot the kernel: builds its own Neighbor
# Solicitation for a target (recording an INCOMPLETE neighbor-cache entry; a
# lookup misses), verifies the NS is wire-correct (solicited-node multicast dst,
# guest src, SLLA option, checksum folds to zero), then ingests a matching
# Neighbor Advertisement and confirms the lookup resolves to the advertised MAC
# (REACHABLE) -- RFC 4861 Neighbor Unreachability Detection.


def test_ipv6_neighbor_cache_nud(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    find_in_order(out, [
        "NUD: resolve ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
