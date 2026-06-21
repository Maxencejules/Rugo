# Full-OS guide Part II.6 acceptance: TCP congestion control (slow start + cwnd).
#
# At boot the kernel runs a congestion-control self-test (tcp::tcp_cc_selftest)
# on a synthetic established connection. It asserts the exact RFC 5681 evolution
# of the congestion window: in slow start a full-MSS ACK adds one SMSS
# (512->1024->1536); once cwnd reaches ssthresh, congestion avoidance adds
# SMSS^2/cwnd (1024->1280); and an RTO timeout halves ssthresh (floored at
# 2*SMSS) and collapses cwnd to one segment (4096->512, ssthresh=2048).
# QEMU's loss-free, near-zero-latency user net cannot exercise these transitions
# on the live wire, so the self-test drives the ACK/timeout events deterministically.
#
# The same state machine is wired into the live path: a cumulative ACK of new
# data calls cc_on_ack, and an RTO timeout (tcp_rt_tick) calls cc_on_timeout.


def test_tcp_congestion_control(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    find_in_order(out, [
        "TCP: cc ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "GOINIT: err" not in out
    # The retransmit + RTT machinery must remain correct alongside cwnd.
    assert "TCP: rto ok" in out
    assert "TCP: rtt ok" in out
    assert "TCP: rto giveup" not in out
