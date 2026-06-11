"""Default shell Alpha check: package sync exposes runnable bundled apps."""


def test_default_shell_runtime_exposes_installed_apps_across_reboot(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    first = boot("pkg\napps\nrun base-shell\nshutdown\n").stdout
    find_in_order(
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
    find_in_order(
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
