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
            "DIAGSVC: start",
            "SVC: diagsvc running",
            "DIAGSVC: ready",
            "GOSH: reply ok",
            "DIAGSVC: snapshot",
            "PROC: timesvc s=1 r=0 f=0 x=0",
            "PROC: diagsvc s=1 r=0 f=0 x=0",
            "PROC: shell s=3 r=2 f=2 x=2",
            "GOSH: diag ok",
            "GOSVCM: stop diagsvc",
            "SVC: diagsvc stopping",
            "DIAGSVC: stop",
            "SVC: diagsvc stopped",
            "GOSVCM: reap diagsvc",
        ],
    )

    assert "GOSVCM: wedge" not in serial, f"Unexpected wedge marker.\nFull output:\n{serial}"
    assert "DIAGSVC: err" not in serial, f"Unexpected diagnostic service error.\nFull output:\n{serial}"
