# Full-OS guide Part II.7 acceptance: PCI device enumeration (driver-model
# discovery step). At boot the kernel scans PCI bus 0 and logs every present
# function's vendor/device/class; multi-function devices are walked too.


def test_pci_device_enumeration(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    find_in_order(out, [
        "PCI: enumerate bus0",
        # virtio-blk (1AF4:1001), mass-storage class 0x01
        "vendor=0x0000000000001AF4 device=0x0000000000001001 class=0x0000000000000100",
        "PCI: devices=0x",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    # virtio-net (1AF4:1000) is also discovered.
    assert "vendor=0x0000000000001AF4 device=0x0000000000001000" in out
    # The multi-function ICH9 LPC bridge at dev 0x1F is walked past func 0.
    assert "dev=0x000000000000001F func=0x0000000000000002" in out
    # All seven q35 bus-0 functions enumerated.
    assert "PCI: devices=0x0000000000000007" in out
    # Driver registry matched the virtio functions and emitted ATTACH.
    assert "ATTACH: virtio-blk-pci" in out
    assert "ATTACH: virtio-net-pci" in out
