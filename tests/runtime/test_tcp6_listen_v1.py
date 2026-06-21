# Full-OS guide Part II.6 acceptance: IPv6 TCP passive open (IPv6 on the wire).
#
# A minimal IPv6 TCP handshake responder: at boot the kernel binds an IPv6
# listener, is fed a bare SYN (-> SYN_RCVD + a wire-correct SYN|ACK whose checksum
# folds to zero), then the client's ACK (-> ESTABLISHED). This completes wire IPv6
# transport for TCP alongside the IPv6 UDP echo.


def test_ipv6_tcp_passive_open(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    find_in_order(out, [
        "TCP6: syn-rcvd",
        "TCP6: established",
        "TCP6: listen ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    # The IPv4 TCP listener must remain correct alongside it.
    assert "TCP: listen ok" in out
