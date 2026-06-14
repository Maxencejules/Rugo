# Full-OS guide Part II.6 acceptance: TCP fast retransmit (RFC 5681 3 dup ACKs).
#
# At boot the kernel runs a fast-retransmit self-test on a synthetic established
# connection: two duplicate ACKs do NOT retransmit; the third triggers an
# immediate retransmit (without the RTO timer elapsing) and enters fast recovery
# (ssthresh = cwnd/2, cwnd = ssthresh + 3*SMSS). The same logic is wired into the
# live ACK path (a dup ACK does not advance snd_una; the 3rd fast-retransmits).


def test_tcp_fast_retransmit(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    find_in_order(out, [
        "TCP: fast rexmit ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "GOINIT: err" not in out
    # The RTO / RTT / congestion machinery must remain correct alongside it.
    assert "TCP: rto ok" in out
    assert "TCP: cc ok" in out
    assert "TCP: rto giveup" not in out
