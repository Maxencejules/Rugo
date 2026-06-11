"""Core runtime acceptance: the default Go lane exposes control/diagnostic flow."""


def test_service_control_runtime_v1_exercises_diag_snapshot_and_shutdown(qemu_serial_go, find_in_order):
    serial = qemu_serial_go.stdout

    find_in_order(
        serial,
        [
            "SVC: diagsvc declared",
            "SVC: diagsvc starting",
            "GOSVCM: class diagsvc best-effort",
            "DIAGSVC: start",
            "SVC: diagsvc running",
            "DIAGSVC: ready",
            "GOSH: session ready",
            "GOSH: reply ok",
            "DIAGSVC: snapshot",
            "PROC: timesvc s=1 r=0 f=0 x=0",
            "PROC: diagsvc s=1 r=0 f=0 x=0",
            "PROC: shell s=1 r=0 f=0 x=0",
            # The sampled scheduler state (st=) of OTHER tasks is a point-in-
            # time read: under preemptive timing a peer can be ready instead
            # of blocked at snapshot instant, so only diagsvc's own state
            # (always running while it samples itself) is pinned.
            "TASK: timesvc tid=1 parent=0 cls=critical st=",
            "TASK: diagsvc tid=2 parent=0 cls=best-effort st=running",
            "TASK: shell tid=4 parent=0 cls=best-effort st=",
            "GOSH: diag ok",
            "GOSVCM: phase shutdown",
            "GOSVCM: stop diagsvc",
            "SVC: diagsvc stopping",
            "DIAGSVC: stop",
            "SVC: diagsvc stopped",
            "GOSVCM: reap diagsvc stopped res=ordered-stop",
        ],
    )

    assert "GOSVCM: wedge" not in serial, f"Unexpected wedge marker.\nFull output:\n{serial}"
    assert "DIAGSVC: err" not in serial, f"Unexpected diagnostic service error.\nFull output:\n{serial}"
    assert "PROC: timesvc s=1 r=0 f=0 x=0" in serial and "svc=ready res=online" in serial
    assert "PROC: pkgsvc s=1 r=0 f=0 x=0" in serial
    assert "TASK: pkgsvc tid=3 parent=0 cls=best-effort st=" in serial
    assert "GOSVCM: stop pkgsvc" in serial
    assert "PKGSVC: stop" in serial
    assert "SVC: pkgsvc stopped" in serial
    assert "GOSVCM: reap pkgsvc stopped res=ordered-stop" in serial
    assert "TASK: timesvc" in serial and "run=" in serial and "tx=" in serial
    assert "TASK: diagsvc" in serial and "run=" in serial and "rx=" in serial
    assert "TASK: shell" in serial and "y=" in serial and "blk=" in serial
