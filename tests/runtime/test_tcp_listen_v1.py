# Full-OS guide Part II.6 acceptance: TCP passive open (listener).
#
# At boot the kernel runs a listener self-test (tcp::tcp_listen_selftest): it
# binds a listener, feeds a synthesized SYN (expecting SYN_RCVD + a SYN|ACK),
# then feeds the client's ACK (expecting ESTABLISHED). The same ST_LISTEN /
# ST_SYN_RCVD arms are wired into the live tcp_input, so the guest can accept
# an inbound connection. The self-test resets the connection afterwards so the
# outbound client path (tcp_connect) is unaffected.


def test_tcp_passive_open(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    find_in_order(out, [
        "TCP: syn-rcvd",
        "TCP: established",
        "TCP: listen ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "GOINIT: err" not in out
