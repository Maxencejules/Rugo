# Full-OS guide Part II.5 acceptance: MBR partition-table parsing.
#
# A crafted MBR (two primary partitions) is written to LBA 0 of the data disk.
# sys_sysinfo op 5 reads LBA 0, validates the 0x55AA signature, and logs each
# non-empty partition entry. partprobe drives it; the test asserts the logged
# type/lba/sectors match the crafted table.

import struct


def test_mbr_partition_parse(qemu_go_c4_runtime, find_in_order):
    boot, disk_path = qemu_go_c4_runtime

    # Pre-create the data disk with an MBR at LBA 0. _ensure_app_region (run
    # inside boot()) sees the file exists and only writes the app region at
    # sector 64+, preserving sector 0.
    mbr = bytearray(1024 * 1024)
    e0 = 446
    mbr[e0 + 4] = 0x83                                   # Linux
    mbr[e0 + 8:e0 + 16] = struct.pack("<II", 2048, 1000)
    e1 = 446 + 16
    mbr[e1 + 4] = 0x0C                                   # FAT32 LBA
    mbr[e1 + 8:e1 + 16] = struct.pack("<II", 4096, 2000)
    mbr[510] = 0x55
    mbr[511] = 0xAA
    with open(disk_path, "wb") as f:
        f.write(mbr)

    out = boot("probe partprobe\nshutdown\n").stdout

    find_in_order(out, [
        "PART: 0000000000000000 type=0x0000000000000083 lba=0x0000000000000800 sectors=0x00000000000003E8",
        "PART: 0000000000000001 type=0x000000000000000C lba=0x0000000000001000 sectors=0x00000000000007D0",
        "PARTPROBE: ok",
        "RUGO: halt ok",
    ])
    assert "PARTPROBE: FAIL" not in out
