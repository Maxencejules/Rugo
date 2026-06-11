# Exec-From-Filesystem Implementation Plan (Phase 4)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Load and run an ELF named by path, with parent/child lifecycle —
gap-analysis §3.4. The shell's `run base-shell` stops printing a canned line
and actually executes a program from the package store on disk.

**Architecture:** A new `sys_spawn` (id 46, inside the reserved v3.x window
28..47) takes a file name, resolves it in a SimpleFS app region on the boot
disk (superblock at sector 64, clear of the runtime-state sectors 8–11),
verifies the PKG v1 SHA-256, validates the ELF with the existing
`elf_v1_validate_image`, loads PT_LOAD segments into the **app window**
`[0x0140_0000, 0x0180_0000)` of the demand-paged region (pages map on
copy), and starts it as a dynamically allocated task (Phase 3) with a
demand-paged stack and minimal capabilities. v1 semantics: the app window
is single-occupancy — a second spawn while an app is resident returns -1;
the parent reaps with the existing `sys_wait`. Real multi-program address
spaces are a later phase; this is the honest Redox/Fuchsia-style
spawn+wait the gap analysis calls "a defensible design".

**Pieces:**
1. `tools/app_disk_v1.py` — writes the SimpleFS app region (superblock at
   sector 64, file table, PKG v1-wrapped ELFs) onto a disk image; reuses
   the `pkg_bootstrap_v1.py` PKG/SimpleFS encoding.
2. `apps/hello/hello.asm` (or reuse the X1 ELF source pattern) — a tiny
   static ET_EXEC app linked at `0x0140_0000` printing `HELLO: app ok`
   via `sys_debug_write`, exiting 0 via `sys_thread_exit`.
3. Kernel `sys_spawn` (id 46): storage-capability check (the caller needs
   `taskCapStorage`; `can_spawn` stays thread_spawn-only), SimpleFS lookup
   by name, PKG SHA-256 verify (kernel hash code from the pkg_hash lane),
   ELF validate + segment copy into the app window, task slot via
   `r4_find_spawn_slot`, `r4_init_task(entry, demand stack slot)`. Marker:
   `EXEC: <name> ok entry=0x...` / `EXEC: <name> denied|missing|badhash`.
4. Shell: `run base-shell` → `sysSpawn("base-shell")` + `sysWait(tid)` →
   `APP: base-shell ok` only when the child really ran and exited 0.
5. conftest: the go-lane boot disk gains the app region via
   `tools/app_disk_v1.py` in the disk-prep fixtures.
6. ABI: document id 46 in the v3.x additive window (same place ids 28–45
   live); extractor picks it up automatically.
7. Boot test `tests/runtime/test_exec_from_fs_v1.py`: ordered chain
   `rugo> run base-shell` → `EXEC: base-shell ok entry=0x` →
   `HELLO: app ok` → `APP: base-shell ok`, plus denial case for a missing
   file and the single-occupancy -1 path.

**Risks:** kernel SHA-256 only exists in the pkg_hash lane (check cfg and
make it available to go_test); app window overlaps the demand heap region
(heap starts at 0x0110_0000 and grows up — document that the heap quota
keeps it below 0x0140_0000 in practice, or move the heap ceiling); the
shell capability model must not let diagsvc/timesvc spawn.
