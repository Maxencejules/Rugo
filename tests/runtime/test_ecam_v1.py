# Full-OS guide Part II.7 acceptance: PCIe ECAM (memory-mapped config space).
#
# At boot the kernel reads the q35 MCH PCIEXBAR to find the ECAM base, then reads
# PCI config space for the host bridge (0:0:0) and the LPC bridge (0:0x1F:0)
# through the memory-mapped ECAM window and confirms each agrees with the legacy
# 0xCF8/0xCFC I/O path. Proves config access works through both mechanisms.


def test_ecam_config_access(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    find_in_order(out, [
        "ECAM: base=0x",
        "ECAM: selftest ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "ECAM: selftest fail" not in out
    assert "ECAM: disabled" not in out
