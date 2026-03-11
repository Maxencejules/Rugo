"""G1 acceptance test: canonical Go userspace bootstrap path."""


def test_go_userspace_bootstrap(qemu_serial_go):
    """Kernel boot must reach Go init, launcher, shell, and syscall-backed service."""
    serial = qemu_serial_go.stdout

    markers = [
        "RUGO: boot ok",
        "GOINIT: start",
        "GOINIT: svcmgr up",
        "GOSVCM: start",
        "TIMESVC: start",
        "TIMESVC: ready",
        "GOSVCM: shell",
        "GOSH: start",
        "GOSH: lookup ok",
        "TIMESVC: req ok",
        "TIMESVC: time ok",
        "GOSH: reply ok",
        "GOINIT: ready",
        "RUGO: halt ok",
    ]

    positions = []
    for marker in markers:
        assert marker in serial, (
            f"Expected '{marker}' in serial output.\n"
            f"Full output:\n{serial}"
        )
        positions.append(serial.index(marker))

    assert positions == sorted(positions), (
        "Expected canonical Go userspace markers in boot order.\n"
        f"Full output:\n{serial}"
    )

    for error_marker in (
        "GOINIT: err",
        "GOSVCM: err",
        "TIMESVC: err",
        "GOSH: err",
    ):
        assert error_marker not in serial, (
            f"Did not expect '{error_marker}' in serial output.\n"
            f"Full output:\n{serial}"
        )
