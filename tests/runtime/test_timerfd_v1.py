# Full-OS guide Part IV.9 acceptance: timerfd (sys_time op 3 + TimerFd fd).
#
# `timerfdprobe` creates a 50 ms one-shot timerfd, verifies an immediate
# read is not ready (0), sleeps 60 ms, then reads the 8-byte expiration
# count (1). Builds on the scheduler idle/wait infrastructure (nanosleep).


def test_timerfd_oneshot(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("probe timerfdprobe\nshutdown\n").stdout

    find_in_order(out, [
        "TIMERFDPROBE: ok",
        "RUGO: halt ok",
    ])
    assert "TIMERFDPROBE: FAIL" not in out
    assert "GOINIT: err" not in out


def test_timerfd_periodic(qemu_go_c4_runtime):
    # Full-OS guide Part IV.9: sys_time op 4 creates a PERIODIC timerfd that
    # re-arms every interval; a read returns the number of expirations since the
    # last read and advances to the next future deadline (a one-shot disarms). The
    # boot self-test exercises the exact re-arm logic the TimerFd read path uses.
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    assert "TIMERFD: periodic ok" in out
    assert "TIMERFD: periodic fail" not in out
