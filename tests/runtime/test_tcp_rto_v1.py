# Full-OS guide Part II.6 acceptance: TCP retransmission / RTO.
#
# At boot the kernel runs a TCP RTO self-test (tcp::tcp_rto_selftest) on a
# synthetic established connection: it sends a data segment (arming the single
# retransmit slot), drives the RTO tick-countdown with NO ACK and confirms the
# segment is retransmitted exactly once, then feeds a cumulative ACK and confirms
# the retransmit timer clears (snd_una advances) and no further retransmission
# occurs. QEMU's user-mode network is loss-free, so the timeout path cannot be
# observed on the live wire; the self-test exercises it deterministically.
#
# The same retransmit machinery is wired into the live path: tcp_send/connect/
# close arm the timer, an inbound cumulative ACK clears it, and the PIT tick
# drives tcp_rt_tick with exponential backoff and give-up after TCP_MAX_RETRIES.


def test_tcp_rto_retransmits_then_clears_on_ack(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    find_in_order(out, [
        "TCP: rto ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "GOINIT: err" not in out
    # The give-up path must NOT fire in the self-test (the ACK arrives first).
    assert "TCP: rto giveup" not in out
