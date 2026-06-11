"""M25 runtime-backed acceptance: real boot path consumes the service model."""


def test_userspace_model_v2_boots_manifest_driven_go_runtime(qemu_serial_go, find_in_order):
    serial = qemu_serial_go.stdout

    find_in_order(
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
            "SVC: shell running",
            "SVC: shell ready",
            "GOSH: session ready",
            "GOSH: lookup ok",
            "GOSH: recv deny",
            "GOSH: reg deny",
            "GOSH: spawn deny",
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
            # Stop requests are issued by the supervisor in reverse plan
            # order; these three lines are same-task and stay ordered.
            "GOSVCM: stop pkgsvc",
            "GOSVCM: stop diagsvc",
            "GOSVCM: stop timesvc",
            "GOINIT: result shutdown-clean",
            "GOINIT: ready",
            "RUGO: halt ok",
        ],
    )

    # Each service's shutdown chain is causally ordered (stop request ->
    # supervisor publishes stopping -> child acknowledges and stops ->
    # supervisor reaps -> clean init result). The interleaving BETWEEN
    # sibling services is scheduler-dependent under preemptive timing and
    # is deliberately not asserted.
    find_in_order(
        serial,
        [
            "GOSVCM: stop timesvc",
            "SVC: timesvc stopping",
            "SVC: timesvc stopped",
            "GOSVCM: reap timesvc stopped res=ordered-stop",
            "GOINIT: result shutdown-clean",
        ],
    )
    find_in_order(
        serial,
        [
            "GOSVCM: stop diagsvc",
            "SVC: diagsvc stopping",
            "DIAGSVC: stop",
            "SVC: diagsvc stopped",
            "GOSVCM: reap diagsvc stopped res=ordered-stop",
            "GOINIT: result shutdown-clean",
        ],
    )
    find_in_order(
        serial,
        [
            "GOSVCM: stop pkgsvc",
            "SVC: pkgsvc stopping",
            "PKGSVC: stop",
            "SVC: pkgsvc stopped",
            "GOSVCM: reap pkgsvc stopped res=ordered-stop",
            "GOINIT: result shutdown-clean",
        ],
    )

    assert serial.count("GOSVCM: plan ") == 4
    assert serial.count("GOSVCM: phase ") == 4
    assert serial.count("SVC: shell starting") == 1
    assert serial.count("GOSVCM: class shell best-effort") == 1
    assert serial.count("GOSVCM: class timesvc critical") == 1
    assert serial.count("GOSVCM: restart shell") == 0
    assert serial.count("GOSVCM: reap shell") == 1
    assert serial.count("GOSH: crash") == 0
    assert serial.count("SVC: shell failed") == 0

    assert serial.count("SVC: timesvc starting") == 1
    assert serial.count("SVC: timesvc running") == 1
    assert serial.count("SVC: timesvc ready") == 1
    assert serial.count("SVC: shell running") == 1
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
