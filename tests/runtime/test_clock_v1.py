# Full-OS guide Part IV.9 acceptance: clock_gettime (sys_time id 53).
#
# `timeprobe` reads CLOCK_MONOTONIC twice across a busy interval (the PIT
# preempts the loop and ticks the clock) and proves it advanced, then reads
# CLOCK_REALTIME and proves it is a plausible recent Unix timestamp from the
# CMOS RTC.


def test_clock_monotonic_and_realtime(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("probe timeprobe\nshutdown\n").stdout

    find_in_order(out, [
        "TIMEPROBE: monotonic ok",
        "TIMEPROBE: realtime ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "TIMEPROBE: monotonic FAIL" not in out
    assert "TIMEPROBE: realtime FAIL" not in out
    assert "GOINIT: err" not in out


def test_clock_extended_ids(qemu_go_c4_runtime):
    # Full-OS guide Part IV.9: the kernel also serves CLOCK_MONOTONIC_RAW (id 2,
    # raw TSC ns -- finer than the 10 ms PIT and unaffected by adjustment) and
    # CLOCK_BOOTTIME (id 3). The boot self-test reads MONOTONIC_RAW twice across a
    # busy interval and proves it advanced even while the PIT-tick MONOTONIC clock
    # is frozen (IF=0), proves BOOTTIME >= MONOTONIC, and proves an unknown clock
    # id returns the error sentinel.
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    assert "CLOCK: ext-ids ok" in out
    assert "CLOCK: ext-ids fail" not in out
