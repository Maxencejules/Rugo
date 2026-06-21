# Full-OS guide Part II.7 acceptance: MSI-X capability setup.
#
# At boot the kernel walks the PCI capability list of each function, finds the
# first device exposing an MSI-X capability (id 0x11) -- the virtio devices in the
# go-lane fixture do -- reads its table size from the Message Control register,
# sets the MSI-X Enable bit, confirms it reads back enabled, then restores the
# original control. It also PROGRAMS the MSI-X table: it locates the table via the
# cap's Table BIR + offset, maps that BAR, writes a message address/data + mask
# into entry 0, reads it back, and restores the original -- so the device is left
# undisturbed (MSI-X stays disabled; no live interrupt is armed).


def test_msix_capability(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    find_in_order(out, [
        "MSIX: dev=0x",
        "enable ok",
        # The MSI-X table registers latched the written message addr/data + mask.
        "MSIX: table ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "MSIX: none" not in out
    assert "enable fail" not in out
    assert "MSIX: table fail" not in out
