# FAT16 read + write — contract v1

Status: boot-verified via `make test-fat16-v1` and `make test-fatwrite-v1`
Source: `kernel_rs/src/lib.rs` (`sys_sysinfo` op 6 read, op 8 list, op 11 write;
`fat16_write_named`), `apps/coreutils/fatprobe.asm`, `apps/coreutils/fatwrprobe.asm`.
Proof: `tests/runtime/test_fat16_v1.py`, `tests/runtime/test_fatwrite_v1.py`.

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

### Directory listing

`sys_sysinfo` (id 61) **op 8** = FAT16 root directory list: walks the root
directory and logs each live 8.3 entry as `FATLS: <name11> size=0x<hex>`
(skipping free `0x00`, deleted `0xE5`, and long-name `attr 0x0F` entries),
returning the entry count. Read-only enumeration; subdirectory recursion and an
`opendir`/`readdir` fd API are carry-forward.

### Namespace mount (`/mnt`)

`open("/mnt/<NAME>")` mounts the FAT16 volume into the namespace: the path's
`name.ext` is converted to a space-padded, upper-cased 8.3 directory name, the
file is read via the same `fat16_read_named` helper into a cache, and a
read-only `FatFile` fd is returned. Any program (e.g. the shell's `fscat`)
reaches FAT files through the normal `open`/`read`/`close` path — no
FAT-specific syscall needed. v1 caches one open `/mnt` file at a time.

### File write (`op 11`)

`sys_sysinfo` (id 61) **op 11** = FAT16 write self-test: `fat16_write_named`
creates a **single-cluster** file in the root directory. To leak nothing on the
deterministic failure paths, it first (a) scans the root directory for a free
slot **and** rejects a duplicate 8.3 name, then (b) finds a free cluster — both
*before* any on-disk mutation. Only then does it commit: mark the cluster
end-of-chain (`0xFFFF`) in **every** FAT copy, write the data into the cluster,
and fill the reserved directory entry (name, attr `0x20`, first cluster, size).
The self-test writes `WRTEST.TXT` and reads it back via `fat16_read_named` to
confirm a byte-exact round-trip (`FATWR: write+read ok`).

## v1 boundary / carry-forward

- **Multi-cluster read via the FAT chain** (`fat16_read_chain`, `sys_sysinfo`
  op 12, proof `test_fat16_chain_v1.py`): reads a file spanning more than one
  cluster by walking the FAT from the directory's first cluster to the
  end-of-chain marker (≥ 0xFFF8), copying each cluster's sectors in order, with an
  iteration guard against a corrupt self-referential chain. The single-cluster
  `fat16_read_named` (op 6, `/mnt`) is retained for the first-cluster fast path.
- **Reads cover chained files; writes are single-cluster, no overwrite.**
  Writes do not yet allocate a chain (a written file must fit in one cluster), no
  append, no create over an existing name (a duplicate name is refused), no
  delete, no timestamps. The free-cluster scan covers only the first FAT sector
  (clusters 2..255).
- **Non-journaling write.** The pre-commit checks remove the deterministic leaks
  (full directory, full FAT, duplicate name). A device-write failure *during* the
  multi-sector commit can still orphan a cluster or leave FAT copies divergent —
  inherent to a non-journaling FAT writer; a `fsck`-style repair (or journaling
  the FAT update) is carry-forward.
- **Root directory only.** No subdirectory traversal, no long file names (LFN).
- **Fixed volume LBA (2048).** The `/mnt` namespace mount works, but the volume
  LBA is fixed — using the MBR parser ([`partitions_v1.md`](partitions_v1.md)) to
  locate the FAT partition dynamically is carry-forward. The `/mnt` cache holds
  one open file at a time.
- FAT12/FAT32 and exFAT are out of scope; the volume is assumed FAT16
  (≥ 4085 clusters), though the single-cluster read path does not depend on the
  FAT entry width.

## Acceptance

`make test-fat16-v1`: the test writes a minimal FAT16 volume (one file
`HELLO.TXT` = `fat16-file-content`) at LBA 2048 of a 4 MiB data disk. `fatprobe`
calls op 6 and echoes the file; the transcript shows `fat16-file-content` then
`FATPROBE: ok`.

`make test-fatwrite-v1`: on that same crafted volume (HELLO.TXT at cluster 2 /
root slot 0), `fatwrprobe` calls op 11; the kernel allocates cluster 3 and root
slot 1, writes `WRTEST.TXT`, and reads it back — the transcript shows
`FATWR: write+read ok`, and host-side the volume holds both `WRTEST  TXT` and the
untouched `HELLO   TXT`.
