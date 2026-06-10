# Dynamic Task Structures Implementation Plan (Phase 3)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Lift the 6-task static limit with dynamically allocated process
structures — gap-analysis §2.2/§3.2 (second half). Process population stops
being fixed at build time.

**Architecture:** The static `R4_TASKS` array becomes a kernel-heap
`alloc::vec::Vec<R4Task>` (the Phase 1 allocator makes this possible),
growing on spawn up to a soft cap of 32. Task slots ≥ 5 get their stacks in
a demand-paged stack area — the Phase 1 window widens to
`[0x100_0000, 0x200_0000)` with per-slot 128 KiB strides from `0x190_0000`
(top-down growth, untouched guard pages between strides; first touch maps
zeroed frames, so new stacks need no eager allocation). Slots 0–4 keep
their existing static strides so every tid-pinning test stays valid.

**Tech Stack:** Rust no_std + alloc, Phase 1 PMM/demand paging, TinyGo
spawn-stress probe, pytest QEMU marker tests.

**Key integration points (from source recon):**
- `R4_TASKS`/`R4_MAX_TASKS`/`R4_NUM_TASKS`: `lib.rs:2178-2247`
- `r4_find_spawn_slot` (slot reuse + growth): `lib.rs:2448-2460`
- `r4_stack_top_for_slot` (8 KiB strides below 0x800000): `lib.rs:2243-2252`
- spawn quota `MAX_THREADS_GLOBAL` via `runtime::isolation::under_quota`:
  `lib.rs:2647`
- demand window constants: `kernel_rs/src/mm.rs` (`DEMAND_BASE/END`, quota)
- Go spawn handshake (`spawnServiceID`/`spawnAck`/gate):
  `services/go/runtime.go` (`launchService`, `goSpawnedThreadMain`)

### Task 1: Failing test — spawn stress beyond the static limit

`tests/runtime/test_dynamic_tasks_v1.py`: boot `qemu_serial_go`, assert
`GOINIT: spawn stress ok n=8` (8 extra worker tasks spawned, run, exited,
and reaped during init — total live tasks 4 services + init + 8 workers
= 13 > 6), no `GOINIT: err`, plus the normal `GOINIT: ready` shutdown
chain. Also a kernel marker `SCHED: tasks high=0x...` (max concurrent
tasks) printed at clean shutdown ≥ 13.

### Task 2: Vec-backed task table

- Replace the static array with `static mut R4_TASKS: Option<Vec<R4Task>>`
  initialized in the go-lane boot (and every R4 lane boot) with the lane's
  initial task count; accessor `fn r4_task(tid) -> &mut R4Task` to keep
  call sites mechanical. `r4_find_spawn_slot` pushes `R4Task::EMPTY` up to
  `R4_SOFT_MAX_TASKS = 32`.
- Raise `MAX_THREADS_GLOBAL` to 32 in `runtime/isolation`.

### Task 3: Demand-paged stacks for high slots

- Widen window: `DEMAND_END = 0x0200_0000`, quota 2048 frames.
- `r4_stack_top_for_slot(slot)`: slots ≥ 5 return
  `0x0190_0000 + (slot as u64 - 4) * 0x2_0000` (top of a 128 KiB stride;
  ~112 KiB usable above the next stride's guard).
- memory_v1.md layout table gains the stack area.

### Task 4: Go-side worker stress probe

- `services/go/spawnstress.go`: `runSpawnStress()` called from
  `bootRuntime` after the demand probe: spawn 8 workers through the
  ack/gate handshake (`workerGoFlag`), each yields twice and exits;
  `sysWait` reaps all 8; log `GOINIT: spawn stress ok n=8`.
- Watch the 28 KiB binary cap (currently 26,548).

### Task 5: Gates, docs, full sweep

- Makefile `test-dynamic-tasks-v1`; SOURCE_MAP + README rows;
  memory_v1/scheduler_v1 contract updates; full `make test-qemu`; commit.

## Self-Review Notes
- The Vec lives in the kernel heap (4 MiB) — 32 × ~300 B is trivial.
- Slot reuse keeps tids dense, so `TASK: x tid=N` assertions stay stable
  for the four services.
- True per-process address spaces and exec-from-filesystem are Phase 4.
