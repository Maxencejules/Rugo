# Phase 3 acceptance: the task population is no longer fixed at build time.
# Go init spawns 8 throwaway workers before any service starts: with init
# that is 9 concurrent tasks, beyond the historical 6-slot static limit.
# Live runtime evidence - serial markers from a normal Go-lane boot.


def test_dynamic_task_table_lifts_static_limit(qemu_serial_go, find_in_order):
    out = qemu_serial_go.stdout
    find_in_order(out, [
        "GOINIT: bootstrap",
        "GOINIT: spawn stress ok n=8",
        "GOINIT: svcmgr up",
        "GOINIT: ready",
        "SCHED: tasks high=0x0000000000000009",
        "RUGO: halt ok",
    ])
    assert out.count("GOINIT: spawn stress ok n=8") == 1
    assert "GOINIT: spawn stress err" not in out
    assert "GOINIT: err" not in out
    # Slot reuse keeps the four services on their historical tids.
    assert "TASK: timesvc tid=1" in out
    assert "TASK: shell tid=4" in out
