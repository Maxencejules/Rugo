# Full-OS guide Part II.6 acceptance: full DHCP DORA.
#
# The kernel now completes Discover -> Offer -> Request -> Ack against
# QEMU's built-in DHCP server (no host network needed). The REQUEST
# confirms the lease (option 50 = offered IP, option 54 = server id).


def test_dhcp_full_dora(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("dhcpcheck\nshutdown\n").stdout

    find_in_order(out, [
        "DHCP: offer ip=0x000000000A00020F",
        "DHCP: request sent",
        "DHCP: ack ip=0x000000000A00020F",
        "NETD: dhcp ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "NETD: dhcp err" not in out
    assert "GOINIT: err" not in out
