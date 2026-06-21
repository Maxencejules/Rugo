# Full-OS guide Part IV.9 acceptance: nanosleep (sys_time op 2) over the
# scheduler idle/wait-queue infrastructure.
#
# `sleepprobe` reads CLOCK_MONOTONIC, sleeps ~100 ms, reads it again, and
# verifies >= ~90 ms of monotonic time elapsed (the task was genuinely
# blocked and woken by the PIT - the kernel idles when nothing else is
# runnable, rather than spinning or deadlocking).


def test_nanosleep_blocks(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("probe sleepprobe\nshutdown\n").stdout

    # The sleeper finishes before the machine halts - the idle path keeps the
    # kernel alive for a pending timed wakeup rather than halting (it may even
    # outlast the async shutdown teardown, which is fine).
    find_in_order(out, [
        "SLEEPPROBE: ok",
        "RUGO: halt ok",
    ])
    assert "SLEEPPROBE: FAIL" not in out
    assert "R4: deadlock" not in out
    assert "GOINIT: err" not in out
