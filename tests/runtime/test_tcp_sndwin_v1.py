# Full-OS guide Part II.6 acceptance: TCP multi-segment send window.
#
# At boot the kernel runs a sliding send-window self-test: it fills the window
# with multiple outstanding segments up to cwnd, confirms a segment exceeding the
# window is refused, retires part of it with a cumulative ACK (sliding snd_una and
# freeing space), confirms more can then be sent, and confirms the peer receive
# window also bounds sending -- the windowed-send accounting beyond the live
# single-outstanding-segment path.


def test_tcp_multi_segment_send_window(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    find_in_order(out, [
        "TCP: sndwin ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    # The single-segment RTO/RTT/CC machinery stays correct alongside it.
    assert "TCP: rto ok" in out
    assert "TCP: cc ok" in out
