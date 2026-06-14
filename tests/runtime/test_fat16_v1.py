# Full-OS guide Part II.5 acceptance: FAT16 file read.
#
# A minimal FAT16 volume (one file HELLO.TXT) is written at LBA 2048 of a 4 MiB
# data disk. sys_sysinfo op 6 parses the BPB, scans the root directory, and
# reads the file's first cluster. fatprobe drives it and echoes the contents.

import struct

SEC = 512


def _fat16_volume():
    total = 6144          # 3 MiB volume -> >4085 clusters => genuine FAT16
    spf = 24              # sectors per FAT
    reserved = 1
    nfats = 2
    root_entries = 512
    vol = bytearray(total * SEC)

    # BPB / boot sector
    vol[0:3] = b"\xEB\x3C\x90"
    vol[3:11] = b"RUGO    "
    struct.pack_into("<H", vol, 11, 512)        # bytes per sector
    vol[13] = 1                                  # sectors per cluster
    struct.pack_into("<H", vol, 14, reserved)
    vol[16] = nfats
    struct.pack_into("<H", vol, 17, root_entries)
    struct.pack_into("<H", vol, 19, total)      # total sectors (16-bit)
    vol[21] = 0xF8                               # media descriptor
    struct.pack_into("<H", vol, 22, spf)
    struct.pack_into("<H", vol, 24, 32)         # sectors per track
    struct.pack_into("<H", vol, 26, 2)          # heads
    vol[36] = 0x80                              # drive number
    vol[38] = 0x29                              # extended boot signature
    struct.pack_into("<I", vol, 39, 0x12345678)
    vol[43:54] = b"RUGOFAT16  "
    vol[54:62] = b"FAT16   "
    vol[510] = 0x55
    vol[511] = 0xAA

    # FAT1 + FAT2: media byte, EOC for cluster 1, EOC for the file's cluster 2.
    fat = bytearray(spf * SEC)
    struct.pack_into("<H", fat, 0, 0xFFF8)
    struct.pack_into("<H", fat, 2, 0xFFFF)
    struct.pack_into("<H", fat, 4, 0xFFFF)
    fat1_off = reserved * SEC
    fat2_off = (reserved + spf) * SEC
    vol[fat1_off:fat1_off + len(fat)] = fat
    vol[fat2_off:fat2_off + len(fat)] = fat

    # Root directory: one 8.3 entry for HELLO.TXT at cluster 2.
    content = b"fat16-file-content"
    root_off = (reserved + nfats * spf) * SEC
    ent = bytearray(32)
    ent[0:11] = b"HELLO   TXT"
    ent[11] = 0x20                              # archive
    struct.pack_into("<H", ent, 26, 2)         # first cluster low
    struct.pack_into("<I", ent, 28, len(content))
    vol[root_off:root_off + 32] = ent

    # Data area: cluster 2.
    root_sectors = (root_entries * 32 + SEC - 1) // SEC
    data_off = (reserved + nfats * spf + root_sectors) * SEC
    vol[data_off:data_off + len(content)] = content
    return bytes(vol)


def test_fat16_file_read(qemu_go_c4_runtime, find_in_order):
    boot, disk_path = qemu_go_c4_runtime

    disk = bytearray(4 * 1024 * 1024)          # 4 MiB
    vol = _fat16_volume()
    disk[2048 * SEC:2048 * SEC + len(vol)] = vol
    with open(disk_path, "wb") as f:
        f.write(disk)

    out = boot("probe fatprobe\nshutdown\n").stdout

    find_in_order(out, [
        "fat16-file-content",
        "FATPROBE: ok",
        "RUGO: halt ok",
    ])
    assert "FATPROBE: FAIL" not in out


def test_fat16_mount_namespace(qemu_go_c4_runtime, find_in_order):
    """The FAT volume is reachable through the namespace at /mnt via plain open."""
    boot, disk_path = qemu_go_c4_runtime

    disk = bytearray(4 * 1024 * 1024)
    vol = _fat16_volume()
    disk[2048 * SEC:2048 * SEC + len(vol)] = vol
    with open(disk_path, "wb") as f:
        f.write(disk)

    # fscat is a generic shell builtin: open(path)+read. /mnt routing makes the
    # FAT file appear in the namespace, so no FAT-specific userspace is needed.
    out = boot("fscat /mnt/HELLO.TXT\nshutdown\n").stdout

    find_in_order(out, [
        "fat16-file-content",
        "FSH: cat ok",
        "RUGO: halt ok",
    ])
    assert "FSH: err" not in out
