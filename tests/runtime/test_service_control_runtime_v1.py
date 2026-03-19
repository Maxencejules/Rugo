"""Core runtime acceptance: the default Go lane exposes control/diagnostic flow."""


def _find_in_order(serial: str, markers: list[str]) -> None:
    pos = -1
    for marker in markers:
        pos = serial.find(marker, pos + 1)
        assert pos != -1, f"Missing '{marker}' in serial output.\nFull output:\n{serial}"


def test_service_control_runtime_v1_exercises_diag_snapshot_and_shutdown(qemu_serial_go):
    serial = qemu_serial_go.stdout

    _find_in_order(
        serial,
        [
            "SVC: diagsvc declared",
            "SVC: diagsvc starting",
            "GOSVCM: class diagsvc best-effort",
            "DIAGSVC: start",
            "SVC: diagsvc running",
            "DIAGSVC: ready",
            "GOSH: reply ok",
            "DIAGSVC: snapshot",
            "PROC: timesvc s=1 r=0 f=0 x=0",
            "PROC: diagsvc s=1 r=0 f=0 x=0",
            "PROC: shell s=3 r=2 f=2 x=2",
            "TASK: timesvc tid=1 parent=0 cls=critical st=blocked",
            "TASK: diagsvc tid=2 parent=0 cls=best-effort st=running",
            "TASK: shell tid=4 parent=0 cls=best-effort st=blocked",
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
    assert "TASK: pkgsvc tid=3 parent=0 cls=best-effort st=blocked" in serial
    assert "GOSVCM: stop pkgsvc" in serial
    assert "PKGSVC: stop" in serial
    assert "SVC: pkgsvc stopped" in serial
    assert "GOSVCM: reap pkgsvc stopped res=ordered-stop" in serial
    assert "TASK: timesvc" in serial and "run=" in serial and "tx=" in serial
    assert "TASK: diagsvc" in serial and "run=" in serial and "rx=" in serial
    assert "TASK: shell" in serial and "y=" in serial and "blk=" in serial
