# VFS + Directories Contract v1 (SimpleFS v2)

Status: live runtime (boot-verified)
Source: `kernel_rs/src/vfs.rs`, `/data` routing in `kernel_rs/src/lib.rs`,
shell builtins in `services/go/fsh.go`
Proof: `make test-vfs-v1`, `tests/runtime/test_vfs_runtime_v1.py`

Closes gap-analysis build-list item 5: create/stat/list arbitrary files
with directories over a real on-disk filesystem. The shell sees a writable
tree under `/data`, not a fixed-path routing table.

## On-disk layout (boot disk, base sector 512 - clear of the runtime-state sectors 8-11 and the exec app region at 64+, which can grow to its full 16-app table)

| Sectors | Content |
|---|---|
| 512 | superblock: magic `VFS2`, version, node count |
| 513–516 | node table: 64 × 32-byte entries — name[20], parent u8 (0xFE = root), kind u8 (1 file, 2 dir), mode u8 (permission bits), owner u8 (uid), start_block u32, size u32 |
| 517 | block allocation bitmap |
| 518+ | 512-byte data blocks |

Files are contiguously allocated (grow = realloc + copy; 8 KiB max in v1).
Mutations are write-through. First boot formats the region in place
(`VFS: format ok`); later boots remount (`VFS: mount ok files=0x<n>`).
Crash journaling is the documented next step.

## Users and permissions

- Every task carries a uid: boot services are root (uid 0), spawned
  external apps run as uid 100; thread spawns inherit.
- Node mode bits: `1` owner-read, `2` owner-write, `4` other-read,
  `8` other-write. Creation stamps the caller as owner with the default
  mode (owner rw, other r); mode 0 on pre-permission images reads as
  the default.
- Enforcement: `/data` opens check requested read/write rights against
  the mode (root bypasses); unlink needs root, the owner, or
  other-write; chmod (`sys_fs_ctl` op 5, shell `fschmod <path> <mode>`)
  needs root or the owner.
- Proof: `make test-users-v1` — the `fsperm` probe (uid 100) is denied
  write and unlink on a root-owned file, allowed read, and allowed all
  three after a root `fschmod 15`.
- Carry-forward: directory-execute semantics, group bits, setuid-like
  transitions, uid surfacing in `ps`.

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
