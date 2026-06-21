# Full-OS guide Part II.6 acceptance: IPv6 NDP Duplicate Address Detection (DAD).
#
# Beyond the NDP responder ([test_ndp_v1]), the guest now performs DAD on its OWN
# address (RFC 4862 sec 5.4 / RFC 4861 sec 4.3). At boot netcfg::dad_selftest builds
# a Neighbor Solicitation FROM the unspecified source (::) FOR the guest's tentative
# link-local address, to the solicited-node multicast, with NO Source Link-Layer
# Address option (mandatory when the source is ::), hop limit 255, and a
# pseudo-header checksum that folds to zero -- all validated against known-correct
# values -- then transmits it. No host defends the guest's fe80:: over the slirp
# link, so the address is unique. This is the guest-initiated half of NDP (it already
# DEFENDS others' DAD probes and resolves remote neighbors via NUD).


def test_ndp_duplicate_address_detection(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    find_in_order(out, [
        # The NDP responder (NS -> NA) self-test still passes.
        "NDP: advert ok",
        # The guest's own DAD probe was built wire-correctly and sent.
        "NDP: dad probe ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "GOINIT: err" not in out
