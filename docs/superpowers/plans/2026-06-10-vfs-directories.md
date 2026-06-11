# VFS + Directories Implementation Plan (Phase 5)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create/stat/list arbitrary files with directories over a real
on-disk filesystem — gap-analysis §3.5. The shell sees a writable file tree,
not six hardcoded paths.

**Architecture:** A SimpleFS v2 region on the boot disk at sector 128
(clear of runtime state 8–11 and the app region 64+): superblock, a
64-entry node table (files AND directories, parent-index links), a block
allocation bitmap, and 512-byte data blocks. A new `kernel_rs/src/vfs.rs`
implements mount/create/lookup/read/write/list/unlink against the existing
`block_io_dispatch`. The frozen FD syscalls grow a routing branch: paths
under `/data/` resolve through the VFS into `M8FdKind::VfsFile` (per-fd
node + offset); reading an opened **directory** fd returns packed dirent
records (name[24], kind u8, size u32 — getdents style, no new syscall);
`sys_fs_ctl` (ABI v3.x id 47, the last reserved slot) multiplexes
{mkdir, unlink, stat} by op code. Writes are synchronous write-through in
v1 (no journal yet — documented carry-forward). The shell gains `fs`
builtins (`fswrite <path> <text>`, `fscat <path>`, `fsls <path>`,
`fsmk <path>`, `fsrm <path>`) so runtime tests can drive the tree
end-to-end, including persistence across reboot.

**Mount:** `vfs_mount()` during the go-lane storage boot probe; if the
region magic is absent, format it in place (`VFS: format ok`) — first boot
self-initializes, later boots remount (`VFS: mount ok files=N`).

**Caps:** `/data` access requires the storage capability; per-file
owner/permission bits are the documented next step (gap §3.5 "then file
permissions"), recorded in the node table layout now (mode byte reserved).

**Tests:** `tests/runtime/test_vfs_runtime_v1.py`
- boot 1: `fsmk /data/etc`, `fswrite /data/etc/motd hello-rugo`,
  `fscat /data/etc/motd` → `hello-rugo`, `fsls /data` shows `etc/`,
  `fsls /data/etc` shows `motd`
- boot 2 (same disk): `fscat /data/etc/motd` → `hello-rugo` (persistence),
  `fsrm /data/etc/motd`, `fscat` → error class marker
- absence of `VFS: err` in clean flows; node/bitmap invariants via
  `VFS: mount ok files=N` counts

**Markers:** `VFS: format ok`, `VFS: mount ok files=N`,
`FS: open/create/mkdir/unlink/stat` error classes via deterministic
shell-visible return codes (shell prints `FSH: <op> ok|err`).

**Risks:** sys_open path-match currently strcmp's six fixed strings —
the `/data/` branch must come first and copyinstr caps path length (256);
BLK_DATA_PAGE is the single I/O buffer — VFS ops must not interleave with
storage journal ops mid-syscall (single-threaded kernel: safe); 28 KiB Go
binary cap (~1.6 KiB headroom) — shell fs builtins must be lean, reuse
lineBuilder.
