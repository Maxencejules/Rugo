# VFS + Directories Contract v1 (SimpleFS v2)

Status: live runtime (boot-verified)
Source: `kernel_rs/src/vfs.rs`, `/data` routing in `kernel_rs/src/lib.rs`,
shell builtins in `services/go/fsh.go`
Proof: `make test-vfs-v1`, `tests/runtime/test_vfs_runtime_v1.py`

Closes gap-analysis build-list item 5: create/stat/list arbitrary files
with directories over a real on-disk filesystem. The shell sees a writable
tree under `/data`, not a fixed-path routing table.

## On-disk layout (boot disk, base sector 128)

| Sectors | Content |
|---|---|
| 128 | superblock: magic `VFS2`, version, node count |
| 129–132 | node table: 64 × 32-byte entries — name[20], parent u8 (0xFE = root), kind u8 (1 file, 2 dir), mode u8 (reserved for permissions), start_block u32, size u32 |
| 133 | block allocation bitmap |
| 134+ | 512-byte data blocks |

Files are contiguously allocated (grow = realloc + copy; 8 KiB max in v1).
Mutations are write-through. First boot formats the region in place
(`VFS: format ok`); later boots remount (`VFS: mount ok files=0x<n>`).
Crash journaling and per-file permissions (the reserved mode byte) are the
documented next steps.

## Syscall surface

- The frozen FD syscalls (`sys_open`/`sys_read`/`sys_write`/`sys_close`)
  route `/data/...` paths into the VFS. Open flags gain a create bit
  (`0x4`) for `/data` only; the storage capability is required.
- Reading an opened **directory** fd returns packed dirent records
  (name[24], kind u8, pad[3], size u32 — getdents style, no new syscall).
- `sys_fs_ctl` (ABI v3.x id 47, the last reserved slot): op 1 = mkdir,
  2 = unlink (files and empty directories), 3 = stat (returns
  `kind << 32 | size`).

## Shell builtins

`fswrite <path> <text>`, `fscat <path>`, `fsls <path>`, `fsmk <path>`,
`fsrm <path>` — emitting `FSH: <op> ok` / `FSH: err`.

## Image growth note

Phase 5 also grew the Go userspace image window from 7 to 8 code pages
(32 KiB cap): the TinyGo binary's BSS had crossed the 7-page boundary.
`tools/build_go.sh` enforces the new cap.
