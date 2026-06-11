# Exec-From-Filesystem Contract v1

Status: live runtime (boot-verified)
Source: `kernel_rs/src/lib.rs` (`sys_spawn_v1`, `exec_load_app`),
`tools/app_disk_v1.py`, `apps/base-shell/`
Proof: `make test-exec-v1`, `tests/runtime/test_exec_from_fs_v1.py`

Closes gap-analysis build-list item 4: load and run an ELF named by path
with a parent/child lifecycle (spawn + wait; Redox/Fuchsia-style spawn, no
fork).

## Syscall

`sys_spawn` (id 46, inside the additive v3.x window `28..47` reserved by
`docs/abi/syscall_v3.md`):

- args: `rdi` = name pointer, `rsi` = name length (1..24 bytes),
  `rdx` = argument-string pointer, `r10` = argument length (0..256)
- returns the child tid, or -1
- caller must hold the storage capability (`taskCapStorage`); `can_spawn`
  remains a thread-spawn-only right
- the kernel copies the argument string, NUL terminated, to the args page
  at `0x017F_F000` (last page of the app window) and hands the child
  `rdi` = args pointer, `rsi` = args length
- the child gets the storage capability (fd limit 4) so file utilities
  can read the `/data` tree, but no network, spawn, or IPC surface;
  isolation domain 5, demand-paged stack stride; reaped with `sys_wait`

## Coreutils

`ls`, `cat`, `echo`, `ps`, and `wc` (`apps/coreutils/*.asm`) ship in the
app region and run as real external programs: the shell's `ls`/`cat`/
`echo`/`ps` commands spawn the on-disk ELF with the command's argument
string and reap it — every output line comes from the program itself
(`ps` enumerates live kernel tasks through `sys_proc_info`).

## Pipes

`sys_fs_ctl` op 4 creates a kernel pipe — a 512-byte in-kernel ring with
reader/writer refcounts — and returns `rfd << 8 | wfd`. Read on an empty
pipe returns -1 while a writer exists (callers yield and retry) and 0
once every writer is gone (EOF). `sys_spawn` takes optional stdin
(`r8`) and stdout (`r9`) pipe fds (`u64::MAX` = none): ownership moves
to the child, so its exit — clean or faulting — releases the end and
propagates EOF; the child sees them in `rdx`/`rcx`.

The shell's `left | right` pipeline joins two external programs this
way (`cat /data/etc/motd | wc`). v1 pipelines run sequentially — left
to completion, then right — bounded by the pipe's ring; concurrent
pipeline stages need per-process address spaces (the exec window is
single-occupancy), which is the documented next architectural step.
Proof: `make test-pipes-v1`, `tests/runtime/test_pipes_runtime_v1.py`.

## On-disk app region

Written by `tools/app_disk_v1.py` at sector 64 of the boot disk (clear of
the runtime-state sectors 8–11):

- sector 64: SimpleFS superblock (magic `SFS1`, file count, data start)
- sector 65: file table, 16 × 32-byte entries (name[24], start sector u32,
  size u32)
- data sectors: PKG v1 frame per file — magic u32, bin_size u32, name[24],
  sha256[32], then the ELF payload (max 16 KiB)

The kernel verifies the SHA-256 of the payload before loading; a mismatch
is `EXEC: <name> badhash` and the spawn fails.

## Exec app window

ET_EXEC ELF64 segments must live entirely in `[0x0140_0000, 0x0180_0000)`
inside the demand-paged region — pages are pre-mapped by `copyout_user`
during the load. v1 semantics: the window is **single-occupancy**; spawning
while an app is resident fails with `EXEC: <name> busy`. The window is
released on any child exit path, including fault containment. Real
multi-program address spaces are a later phase.

## Marker contract

| Marker | Meaning |
|---|---|
| `EXEC: <name> ok` | app verified, loaded, child task created |
| `EXEC: <name> missing` | name not present in the app region |
| `EXEC: <name> badhash` | SHA-256 mismatch — payload rejected |
| `EXEC: <name> busy` | app window occupied |
| `EXEC: <name> noregion / nodisk / ioerr / badpkg / badsize / badelf / full` | other deterministic failure classes |
| `BASESH: hello from disk` | emitted by the running app itself (only exists in the ELF payload) |
| `APP: base-shell ok` | shell reaped the child after a clean exit |
| `APP: base-shell exec err` | spawn or wait failed |
