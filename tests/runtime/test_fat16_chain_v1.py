# Full-OS guide Part II.5 acceptance: FAT16 multi-cluster (FAT-chain) read.
#
# A FAT16 volume holds BIG.TXT, a 600-byte file spanning TWO clusters (cluster 2
# -> cluster 3 -> EOC) with content byte i == i & 0xFF. sys_sysinfo op 12 walks
# the FAT chain and verifies the pattern across the cluster boundary (the byte at
# offset 512 lives in the second cluster, reachable only by following FAT[2]->3),
# proving the reader follows the chain rather than stopping at the first cluster.

import struct

SEC = 512


def _fat16_volume_bigfile():
    total = 6144          # 3 MiB volume -> >4085 clusters => genuine FAT16
    spf = 24
    reserved = 1
    nfats = 2
    root_entries = 512
    vol = bytearray(total * SEC)

    vol[0:3] = b"\xEB\x3C\x90"
    vol[3:11] = b"RUGO    "
    struct.pack_into("<H", vol, 11, 512)        # bytes per sector
    vol[13] = 1                                  # sectors per cluster
    struct.pack_into("<H", vol, 14, reserved)
    vol[16] = nfats
    struct.pack_into("<H", vol, 17, root_entries)
    struct.pack_into("<H", vol, 19, total)
    vol[21] = 0xF8
    struct.pack_into("<H", vol, 22, spf)
    struct.pack_into("<H", vol, 24, 32)
    struct.pack_into("<H", vol, 26, 2)
    vol[36] = 0x80
    vol[38] = 0x29
    struct.pack_into("<I", vol, 39, 0x12345678)
    vol[43:54] = b"RUGOFAT16  "
    vol[54:62] = b"FAT16   "
    vol[510] = 0x55
    vol[511] = 0xAA

    # 600-byte file: 512 bytes in cluster 2, the remaining 88 in cluster 3.
    content = bytes((i & 0xFF) for i in range(600))

    # FAT: media byte + reserved, then BIG.TXT's chain 2 -> 3 -> EOC, and BAD.TXT's
    # broken chain: cluster 4's FAT entry is the BAD-CLUSTER marker 0xFFF7 (not a
    # valid next cluster), so a correct reader must stop at cluster 4 rather than
    # follow 0xFFF7 to a wild data_lba (the full-os Part II.5 bad-cluster guard).
    fat = bytearray(spf * SEC)
    struct.pack_into("<H", fat, 0, 0xFFF8)   # cluster 0 (media)
    struct.pack_into("<H", fat, 2, 0xFFFF)   # cluster 1 (reserved)
    struct.pack_into("<H", fat, 4, 3)        # cluster 2 -> next is cluster 3
    struct.pack_into("<H", fat, 6, 0xFFFF)   # cluster 3 -> EOC
    struct.pack_into("<H", fat, 8, 0xFFF7)   # cluster 4 -> BAD-CLUSTER marker (corrupt)
    fat1_off = reserved * SEC
    fat2_off = (reserved + spf) * SEC
    vol[fat1_off:fat1_off + len(fat)] = fat
    vol[fat2_off:fat2_off + len(fat)] = fat

    # Root directory: BIG.TXT at first cluster 2 (size 600), then BAD.TXT at first
    # cluster 4 (size 600 -- it claims a second cluster, but FAT[4] is bad).
    root_off = (reserved + nfats * spf) * SEC
    ent = bytearray(32)
    ent[0:11] = b"BIG     TXT"
    ent[11] = 0x20
    struct.pack_into("<H", ent, 26, 2)
    struct.pack_into("<I", ent, 28, len(content))
    vol[root_off:root_off + 32] = ent
    ent2 = bytearray(32)
    ent2[0:11] = b"BAD     TXT"
    ent2[11] = 0x20
    struct.pack_into("<H", ent2, 26, 4)         # first cluster 4
    struct.pack_into("<I", ent2, 28, 600)       # size 600 (would need a 2nd cluster)
    vol[root_off + 32:root_off + 64] = ent2

    # Data area: cluster 2 = bytes[0:512], cluster 3 = bytes[512:600], cluster 4 =
    # BAD.TXT's first (and only readable) cluster.
    root_sectors = (root_entries * 32 + SEC - 1) // SEC
    data_off = (reserved + nfats * spf + root_sectors) * SEC
    vol[data_off:data_off + 512] = content[0:512]              # cluster 2
    vol[data_off + 512:data_off + 512 + 88] = content[512:600]  # cluster 3
    vol[data_off + 1024:data_off + 1024 + 512] = bytes([0xBB]) * 512  # cluster 4 (BAD.TXT)
    return bytes(vol)


def test_fat16_chain_read(qemu_go_c4_runtime, find_in_order):
    boot, disk_path = qemu_go_c4_runtime

    disk = bytearray(4 * 1024 * 1024)
    vol = _fat16_volume_bigfile()
    disk[2048 * SEC:2048 * SEC + len(vol)] = vol
    with open(disk_path, "wb") as f:
        f.write(disk)

    out = boot("probe fatbigprobe\nshutdown\n").stdout

    find_in_order(out, [
        "FATBIG: chain read ok size=0x0000000000000258",  # 600 bytes
        # The reader stops at BAD.TXT's bad-cluster FAT entry (0xFFF7), returning
        # only its first cluster (512 bytes) instead of following the bogus value.
        "FATBIG: bad-cluster guarded ok",
        "FATBIGPROBE: ok",
        "RUGO: halt ok",
    ])
    assert "FATBIG: chain read fail" not in out
    assert "FATBIG: bad-cluster guard FAIL" not in out
    assert "FATBIGPROBE: FAIL" not in out
