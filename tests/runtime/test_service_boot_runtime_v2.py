"""M25 runtime-backed acceptance: real boot path consumes the service model."""


def _find_in_order(serial: str, markers: list[str]) -> None:
    pos = -1
    for marker in markers:
        pos = serial.find(marker, pos + 1)
        assert pos != -1, f"Missing '{marker}' in serial output.\nFull output:\n{serial}"


def test_userspace_model_v2_boots_manifest_driven_go_runtime(qemu_serial_go):
    serial = qemu_serial_go.stdout

    _find_in_order(
        serial,
        [
            "RUGO: boot ok",
            "GOINIT: start",
            "GOINIT: bootstrap",
            "GOINIT: svcmgr up",
            "GOSVCM: start",
            "SVC: timesvc declared",
            "GOSVCM: plan timesvc role=time",
            "SVC: diagsvc declared",
            "GOSVCM: plan diagsvc role=diag",
            "SVC: pkgsvc declared",
            "GOSVCM: plan pkgsvc role=pkg phase=base need=optional",
            "SVC: shell declared",
            "GOSVCM: plan shell role=shell",
            "GOSVCM: phase core",
            "SVC: timesvc starting",
            "GOSVCM: class timesvc critical",
            "TIMESVC: start",
            "SVC: timesvc running",
            "TIMESVC: ready",
            "SVC: timesvc ready",
            "GOSVCM: phase base",
            "DIAGSVC: ready",
            "SVC: diagsvc ready",
            "PKGSVC: ready",
            "SVC: pkgsvc ready",
            "GOINIT: operational",
            "GOSVCM: phase session",
            "GOSVCM: shell",
            "SVC: shell starting",
            "GOSVCM: class shell best-effort",
            "GOSH: start",
            "GOSH: recycle",
            "SVC: shell failed",
            "GOSVCM: reap shell failed",
            "GOSVCM: restart shell",
            "SVC: shell starting",
            "GOSVCM: class shell best-effort",
            "GOSH: start",
            "GOSH: recycle",
            "SVC: shell failed",
            "GOSVCM: reap shell failed",
            "GOSVCM: restart shell",
            "SVC: shell starting",
            "GOSVCM: class shell best-effort",
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
            "GOSH: diag ok",
            "NETC4: reply ok",
            "ISOC5: domain ok",
            "ISOC5: quota ok",
            "ISOC5: observe ok",
            "SOAKC5: mixed ok",
            "SVC: shell stopping",
            "SVC: shell stopped",
            "ISOC5: cleanup ok",
            "GOSVCM: reap shell stopped res=session-done",
            "GOSVCM: phase shutdown",
            "GOSVCM: stop pkgsvc",
            "SVC: pkgsvc stopping",
            "GOSVCM: stop diagsvc",
            "SVC: diagsvc stopping",
            "GOSVCM: stop timesvc",
            "SVC: timesvc stopping",
            "SVC: timesvc stopped",
            "DIAGSVC: stop",
            "SVC: diagsvc stopped",
            "GOSVCM: reap timesvc stopped res=ordered-stop",
            "GOSVCM: reap diagsvc stopped res=ordered-stop",
            "GOSVCM: reap pkgsvc stopped res=ordered-stop",
            "GOINIT: result shutdown-clean",
            "GOINIT: ready",
            "RUGO: halt ok",
        ],
    )

    assert serial.count("GOSVCM: plan ") == 4
    assert serial.count("GOSVCM: phase ") == 4
    assert serial.count("SVC: shell starting") == 3
    assert serial.count("GOSVCM: class shell best-effort") == 3
    assert serial.count("GOSVCM: class timesvc critical") == 1
    assert serial.count("GOSVCM: restart shell") == 2
    assert serial.count("GOSVCM: reap shell") == 3
    assert serial.count("GOSH: recycle") == 2
    assert serial.count("SVC: shell failed") == 2

    assert serial.count("SVC: timesvc starting") == 1
    assert serial.count("SVC: timesvc running") == 1
    assert serial.count("SVC: timesvc ready") == 1
    assert serial.count("SVC: shell running") == 3
    assert serial.count("SVC: shell ready") == 1
    assert serial.count("SVC: shell stopped") == 1
    assert serial.count("SVC: timesvc stopped") == 1
    assert serial.count("GOSVCM: stop pkgsvc") == 1
    assert serial.count("GOSVCM: stop diagsvc") == 1
    assert serial.count("GOSVCM: stop timesvc") == 1
    assert serial.count("GOSVCM: phase shutdown") == 1
    assert serial.count("SVC: diagsvc stopped") == 1
    assert serial.count("SVC: pkgsvc stopped") == 1
    assert "GOINIT: result shutdown-clean" in serial

    for error_marker in (
        "GOINIT: err",
        "GOSVCM: err",
        "TIMESVC: err",
        "GOSH: err",
        "R4: mgr",
    ):
        assert error_marker not in serial, (
            f"Did not expect '{error_marker}' in serial output.\n"
            f"Full output:\n{serial}"
        )
