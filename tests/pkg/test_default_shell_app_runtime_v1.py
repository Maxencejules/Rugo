"""Default shell Alpha check: package sync exposes runnable bundled apps."""


def _find_in_order(serial: str, markers: list[str]) -> None:
    pos = -1
    for marker in markers:
        pos = serial.find(marker, pos + 1)
        assert pos != -1, f"Missing '{marker}' in serial output.\nFull output:\n{serial}"


def test_default_shell_runtime_exposes_installed_apps_across_reboot(qemu_go_c4_runtime):
    boot, _disk_path = qemu_go_c4_runtime

    first = boot("pkg\napps\nrun base-shell\nshutdown\n").stdout
    _find_in_order(
        first,
        [
            "PKGSVC: ready",
            "UPD3: metadata ok",
            "GOSH: pkg ok",
            "APP: base-shell installed",
            "APP: net-tools installed",
            "APP: media-suite missing",
            "APP: base-shell ok",
            "GOINIT: ready",
            "RUGO: halt ok",
        ],
    )

    second = boot("pkg\napps\nrun media-suite\nshutdown\n").stdout
    _find_in_order(
        second,
        [
            "UPD3: rotate ok",
            "UPD3: metadata ok",
            "UPD3: apply ok",
            "GOSH: pkg ok",
            "APP: base-shell installed",
            "APP: net-tools installed",
            "APP: media-suite installed",
            "APP: media-suite ok",
            "GOINIT: ready",
            "RUGO: halt ok",
        ],
    )
