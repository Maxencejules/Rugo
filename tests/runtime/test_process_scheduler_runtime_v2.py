"""M16 runtime-backed acceptance: the default Go lane reaps and restarts services."""


def _find_in_order(serial: str, markers: list[str]) -> None:
    pos = -1
    for marker in markers:
        pos = serial.find(marker, pos + 1)
        assert pos != -1, f"Missing '{marker}' in serial output.\nFull output:\n{serial}"


def test_process_scheduler_runtime_v2_waits_reaps_and_restarts(qemu_serial_go):
    serial = qemu_serial_go.stdout

    _find_in_order(
        serial,
        [
            "SVC: shell starting",
            "GOSH: start",
            "SVC: shell running",
            "GOSH: recycle",
            "SVC: shell failed",
            "GOSVCM: reap shell failed res=runtime-failed",
            "GOSVCM: restart shell",
            "SVC: shell starting",
            "GOSH: start",
            "SVC: shell running",
            "GOSH: recycle",
            "SVC: shell failed",
            "GOSVCM: reap shell failed res=runtime-failed",
            "GOSVCM: restart shell",
            "SVC: shell starting",
            "GOSH: start",
            "SVC: shell running",
            "GOSH: lookup ok",
            "GOSH: recv deny",
            "GOSH: reg deny",
            "GOSH: spawn deny",
            "SVC: shell ready",
            "TIMESVC: req ok",
            "TIMESVC: time ok",
            "GOSH: reply ok",
            "SOAKC5: mixed ok",
            "SVC: shell stopping",
            "SVC: shell stopped",
            "ISOC5: cleanup ok",
            "GOSVCM: reap shell stopped res=session-done",
            "GOSVCM: phase shutdown",
            "GOSVCM: stop pkgsvc",
            "GOSVCM: stop diagsvc",
            "GOSVCM: stop timesvc",
            "SVC: timesvc stopping",
            "SVC: timesvc stopped",
            "GOSVCM: reap timesvc stopped res=ordered-stop",
            "GOINIT: result shutdown-clean",
            "GOINIT: ready",
        ],
    )

    assert serial.count("SVC: shell starting") == 3
    assert serial.count("SVC: shell running") == 3
    assert serial.count("GOSH: recycle") == 2
    assert serial.count("SVC: shell ready") == 1
    assert serial.count("SVC: shell failed") == 2
    assert serial.count("GOSVCM: restart shell") == 2
    assert serial.count("GOSVCM: reap shell") == 3
    assert serial.count("GOSVCM: stop pkgsvc") == 1
    assert serial.count("GOSVCM: phase shutdown") == 1
    assert serial.count("SVC: timesvc stopped") == 1
    assert serial.count("GOSVCM: stop diagsvc") == 1
    assert serial.count("GOSVCM: stop timesvc") == 1
    assert "GOINIT: result shutdown-clean" in serial
    assert "R4: deadlock" not in serial, f"Unexpected deadlock marker.\nFull output:\n{serial}"
    assert "GOSVCM: err" not in serial, f"Unexpected service-manager error.\nFull output:\n{serial}"
    assert "R4: mgr" not in serial, f"Unexpected manager debug marker.\nFull output:\n{serial}"
