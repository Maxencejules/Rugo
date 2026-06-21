# Full-OS guide Part II.6 acceptance: TCP RTT estimation (SRTT/RTTVAR) + Karn.
#
# At boot the kernel runs a TCP RTT self-test (tcp::tcp_rtt_selftest) on a
# synthetic established connection. It takes two clean (never-retransmitted) RTT
# measurements at known PIT-tick deltas and asserts the exact RFC 6298 integer
# fixed-point evolution of SRTT/RTTVAR and the derived RTO, that the adaptive RTO
# then drives the next segment's retransmit timer, and that a RETRANSMITTED
# segment's ACK does NOT update the estimate (Karn's algorithm). QEMU's user-mode
# network is loss-free and has near-zero latency, so adaptive-RTO behaviour cannot
# be observed on the live wire; the self-test exercises it deterministically.
#
# The same estimator is wired into the live path: tcp_rt_arm drives the timer
# from the SRTT-derived RTO, a clean cumulative ACK folds the measured RTT into
# SRTT/RTTVAR, and a retransmitted segment is excluded (Karn).


def test_tcp_rtt_estimation_and_karn(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    find_in_order(out, [
        "TCP: rtt ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "GOINIT: err" not in out
    # The RTO machinery must remain correct alongside the estimator.
    assert "TCP: rto ok" in out
    assert "TCP: rto giveup" not in out
