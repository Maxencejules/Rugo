# Phase 1 acceptance: boot-verified dynamic memory foundation (PMM + heap +
# demand paging). Live runtime evidence per SOURCE_MAP.md - serial markers only.


def test_pmm_boot_marker_kernel_lane(qemu_serial, find_in_order):
    out = qemu_serial.stdout
    find_in_order(out, [
        "RUGO: boot ok",
        "MM: paging=on",
        "MM: pmm ok frames=0x",
        "RUGO: halt ok",
    ])
    assert "MM: pmm none" not in out


def test_pmm_boot_marker_go_lane(qemu_serial_go, find_in_order):
    out = qemu_serial_go.stdout
    find_in_order(out, [
        "RUGO: boot ok",
        "MM: pmm ok frames=0x",
        "GOINIT: start",
        "GOINIT: ready",
        "RUGO: halt ok",
    ])
    assert "MM: pmm none" not in out


def test_heap_boot_marker_kernel_lane(qemu_serial, find_in_order):
    out = qemu_serial.stdout
    find_in_order(out, [
        "MM: pmm ok frames=0x",
        "MM: heap ok size=0x0000000000400000",
        "MM: heap selftest ok",
        "RUGO: halt ok",
    ])
    assert "MM: heap none" not in out
    assert "MM: heap selftest err" not in out


def test_heap_boot_marker_go_lane(qemu_serial_go, find_in_order):
    out = qemu_serial_go.stdout
    find_in_order(out, [
        "MM: heap ok size=0x0000000000400000",
        "MM: heap selftest ok",
        "GOINIT: ready",
    ])


def test_demand_paging_go_lane(qemu_serial_go, find_in_order):
    out = qemu_serial_go.stdout
    find_in_order(out, [
        "GOINIT: bootstrap",
        "MM: demand map va=0x",
        "GOINIT: mem demand ok",
        "GOINIT: svcmgr up",
        "GOINIT: ready",
        "RUGO: halt ok",
    ])
    assert out.count("MM: demand map va=0x") == 16
    assert "GOINIT: mem demand err" not in out
    assert "USERPF:" not in out
