# Phase 2 acceptance: the DEFAULT lane preempts user tasks on the PIT timer.
# Live runtime evidence - serial markers from a normal Go-lane boot.


def _find_in_order(serial: str, markers: list[str]) -> None:
    pos = 0
    for marker in markers:
        found = serial.find(marker, pos)
        assert found != -1, f"marker not found in order: {marker}"
        pos = found + len(marker)


def test_default_lane_preempts(qemu_serial_go):
    out = qemu_serial_go.stdout
    _find_in_order(out, [
        "SCHED: preempt on hz=100",
        "GOINIT: start",
        "SCHED: preempt hit",
        "GOINIT: ready",
        "RUGO: halt ok",
    ])
    assert out.count("SCHED: preempt on hz=100") == 1
    assert out.count("SCHED: preempt hit") == 1
    assert "GOINIT: err" not in out
    assert "GOSVCM: err" not in out
