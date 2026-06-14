# FAT16 read — contract v1

Status: boot-verified via `make test-fat16-v1`
Source: `kernel_rs/src/lib.rs` (`sys_sysinfo` op 6),
`apps/coreutils/fatprobe.asm`.
Proof: `tests/runtime/test_fat16_v1.py`.

Full-OS implementation guide Part II.5 (filesystem maturity), FAT slice —
read a file from a foreign FAT16 volume on the block device. This is the first
read of a non-native (industry-standard) on-disk filesystem.

## Behaviour

`sys_sysinfo` (id 61) **op 6** = FAT16 read: reads the BPB at a fixed volume LBA
(2048), then:

1. parses `bytes_per_sector` (must be 512), `sectors_per_cluster`,
   `reserved_sectors`, `num_fats`, `root_entries`, `sectors_per_fat`;
2. computes `root_lba = vol + reserved + num_fats*spf` and
   `data_lba = root_lba + ceil(root_entries*32 / 512)`;
3. scans the root directory for the 8.3 name `HELLO   TXT`, reading
   `first_cluster` (offset 26) and `file_size` (offset 28);
4. reads that cluster (`data_lba + (first_cluster-2)*spc`) and copies
   `min(file_size, cap)` bytes (≤ one sector) into the caller's buffer.

Returns the byte count, or `u64::MAX` on any error (no disk, bad BPB, file
absent, empty file).

## v1 boundary / carry-forward

- **Read-only, one fixed file, single cluster.** No FAT chain walk (files must
  fit in one cluster), no write, no create/delete, no timestamps.
- **Root directory only.** No subdirectory traversal, no long file names (LFN).
- **Fixed volume LBA (2048).** No partition-aware mount yet — combining this with
  the MBR parser ([`partitions_v1.md`](partitions_v1.md)) to locate the FAT
  partition, and a general `open("/mnt/...")` path, are carry-forward.
- FAT12/FAT32 and exFAT are out of scope; the volume is assumed FAT16
  (≥ 4085 clusters), though the single-cluster read path does not depend on the
  FAT entry width.

## Acceptance

`make test-fat16-v1`: the test writes a minimal FAT16 volume (one file
`HELLO.TXT` = `fat16-file-content`) at LBA 2048 of a 4 MiB data disk. `fatprobe`
calls op 6 and echoes the file; the transcript shows `fat16-file-content` then
`FATPROBE: ok`.
