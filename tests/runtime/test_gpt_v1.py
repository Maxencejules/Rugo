# Full-OS guide Part II.5 acceptance: GPT partition table parse.
#
# A GPT disk is crafted (protective MBR at LBA 0, GPT header "EFI PART" at LBA 1,
# two partition entries at LBA 2). At boot the kernel reads LBA 1, validates the
# signature, walks the entry array, and counts the live entries (non-zero type
# GUID), logging each one's first/last LBA. Complements the MBR parser.

import struct
import uuid

SEC = 512


def _gpt_disk(size_bytes=4 * 1024 * 1024):
    disk = bytearray(size_bytes)
    total_sectors = size_bytes // SEC

    # Protective MBR (LBA 0): one 0xEE partition spanning the disk + 0x55AA.
    mbr = disk
    e = 446
    mbr[e + 4] = 0xEE  # type = GPT protective
    struct.pack_into("<I", mbr, e + 8, 1)  # first LBA
    struct.pack_into("<I", mbr, e + 12, min(total_sectors - 1, 0xFFFFFFFF))
    mbr[510] = 0x55
    mbr[511] = 0xAA

    # GPT header (LBA 1). CRCs left zero (the kernel v1 does not validate them).
    hdr = bytearray(SEC)
    hdr[0:8] = b"EFI PART"
    struct.pack_into("<I", hdr, 8, 0x00010000)   # revision 1.0
    struct.pack_into("<I", hdr, 12, 92)          # header size
    struct.pack_into("<Q", hdr, 24, 1)           # my LBA
    struct.pack_into("<Q", hdr, 32, total_sectors - 1)  # alternate LBA
    struct.pack_into("<Q", hdr, 40, 34)          # first usable
    struct.pack_into("<Q", hdr, 48, total_sectors - 34)  # last usable
    hdr[56:72] = uuid.uuid4().bytes              # disk GUID
    struct.pack_into("<Q", hdr, 72, 2)           # partition entry LBA
    struct.pack_into("<I", hdr, 80, 128)         # number of entries
    struct.pack_into("<I", hdr, 84, 128)         # size of each entry
    disk[1 * SEC:1 * SEC + SEC] = hdr

    # Partition entry array (LBA 2): two live entries.
    def entry(type_guid, part_guid, first, last, name):
        ent = bytearray(128)
        ent[0:16] = type_guid
        ent[16:32] = part_guid
        struct.pack_into("<Q", ent, 32, first)
        struct.pack_into("<Q", ent, 40, last)
        nm = name.encode("utf-16-le")
        ent[56:56 + len(nm)] = nm
        return ent

    LINUX = uuid.UUID("0fc63daf-8483-4772-8e79-3d69d8477de4").bytes
    EFI = uuid.UUID("c12a7328-f81f-11d2-ba4b-00a0c93ec93b").bytes
    arr = bytearray(SEC)
    arr[0:128] = entry(EFI, uuid.uuid4().bytes, 34, 2081, "ESP")
    arr[128:256] = entry(LINUX, uuid.uuid4().bytes, 2082, total_sectors - 34, "root")
    disk[2 * SEC:2 * SEC + SEC] = arr
    return bytes(disk)


def test_gpt_partition_parse(qemu_go_c4_runtime, find_in_order):
    boot, disk_path = qemu_go_c4_runtime

    # Create the disk with the GPT structures (LBA 0..2). boot() then runs
    # _ensure_app_region, which adds the app region at sector 64+ INTO this file,
    # preserving the GPT (matching the FAT-test pattern).
    with open(disk_path, "wb") as f:
        f.write(_gpt_disk())

    out = boot("shutdown\n").stdout

    find_in_order(out, [
        "GPT: part first=0x0000000000000022",   # ESP at LBA 34 (0x22)
        "GPT: part first=0x0000000000000822",   # root at LBA 2082 (0x822)
        "GPT: parsed n=0x0000000000000002",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "GPT: none" not in out
    assert "GPT: bad header" not in out
