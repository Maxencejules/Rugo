# Per-process address spaces — contract v1

Status: boot-verified via `make test-concurrent-exec-v1`
Source: `kernel_rs/src/mm.rs` (address-space primitives),
`kernel_rs/src/lib.rs` (`R4Task.pml4_phys`, `r4_switch_to`, `sys_spawn_v1`,
`r4_exit_and_switch`, go-lane boot), `apps/coreutils/asprobe.asm`,
`services/go/coreutils.go` (`asConc`).
Proof: `tests/runtime/test_concurrent_exec_v1.py`.

This is the keystone of the full-OS implementation guide
([`docs/analysis/full-os-implementation-guide.md`](../analysis/full-os-implementation-guide.md)
§ "Per-process address spaces"). It lifts the single shared `USER_PML4`
ceiling so spawned apps run in isolated address spaces and concurrently.

## Model

Each `R4Task` carries a `pml4_phys: u64`:

- `0` — run on whatever table CR3 already holds (other lanes; no reload).
- non-zero — this task owns a private PML4 (its physical address).

The go-lane boot task (tid 0) and its service threads carry
`SHARED_PML4_PHYS` (the shared/boot table set up by `setup_go_user_pages`).
`r4_init_task` makes a new thread inherit its spawner's table; `sys_spawn`
installs a fresh private table for an external app.

`r4_switch_to` reloads CR3 from the target task's `pml4_phys` before
restoring its register frame (only when non-zero). This flushes the TLB on
every task switch — accepted in v1; PCID is a future optimization.

## Address-space layout

A private space is created by `mm::address_space_create(src)`:

1. Allocate three frames: PML4, PDPT, PD (all zeroed by `alloc_frame`).
2. Clone all 512 entries of the source PML4 — this carries the kernel
   half (kernel text, the HHDM, the page-table pool) so the kernel stays
   reachable under any CR3.
3. Override PML4[0] with the fresh private PDPT, PDPT[0] with the fresh PD.

All user memory (exec window `[0x0140_0000,0x0180_0000)`, demand heap
`[0x0100_0000,0x0200_0000)`, demand stacks from `0x0190_0000`, args page
`0x017F_F000`) lives in the first 1 GiB, so a single PD per space suffices.
PD/PT leaf entries are filled on demand by `try_demand_map` (which reads
CR3, so it automatically targets the faulting task's space) and, at load
time, by `mm::as_copyout` / `mm::as_map_zeroed` (which walk an explicit
PML4 so the ELF and args land in the child, not the spawner).

## Spawn

`sys_spawn_v1` is no longer single-occupancy. Per spawn it:

1. Reserves a task slot.
2. `address_space_create(SHARED_PML4_PHYS)` → child PML4.
3. `exec_load_app(child_pml4, elf)` maps + copies PT_LOAD segments into the
   child (exec-window pages stay executable; everything else is NX).
4. Copies the NUL-terminated args into the child's args page.
5. Installs `pml4_phys = child_pml4` after `r4_init_task`.

Any failure after step 2 calls `address_space_release` before returning.

## Exit

`r4_exit_and_switch` (every exit path: clean exit, signal kill, fault
containment) reclaims a private space: it steps CR3 onto the shared table
first (so it is not standing on the tree it frees), then
`address_space_release` walks PML4[0]→PDPT[0]→PD→PTs, returning every user
frame and page-table frame to the PMM and rolling back the demand-frame
accounting. Cloned kernel-half entries are never freed.

## Markers

| Marker | Emitted when |
|--------|--------------|
| `SPAWN: <name> as_ok 0x<pml4_phys>` | a child address space is created |
| `ASRELEASE: tid=0x<tid> as=0x<pml4_phys>` | a private space is reclaimed |
| `ASPROBE: tick id=<X>` | probe progress (interleaved → concurrency) |
| `ASPROBE: iso ok id=<X>` | probe's global survived the sibling (isolation) |
| `ASPROBE: iso FAIL id=<X>` | isolation violated (must never appear) |

`serial_write_hex` emits 16 zero-padded hex digits.

## v1 boundary / carry-forward

- **No CoW, no fork.** Spawned spaces are built fresh from the ELF; there
  is no parent/child page sharing. `fork`/`clone` + copy-on-write are the
  next guide section and build on this.
- **Single PD per space.** User memory is confined to the first 1 GiB; a
  mapping above 1 GiB would need PDPT/PD growth (not used today).
- **Demand-frame quota is global.** `DEMAND_MAPPED` is shared across all
  spaces; per-space quotas are deferred (see the Memory section).
- **No per-fault `PF:` marker.** The guide's illustrative `PF: <task>
  as=0x<id>` marker is intentionally omitted so existing demand-map
  marker-count assertions are not perturbed; CR3 already proves targeting.
- **TLB flush per switch.** No PCID yet.

## Acceptance

`make test-concurrent-exec-v1` boots the go lane, runs `asconc`, and
asserts: two distinct `SPAWN: ... as_ok` lines (window no longer
single-occupancy), `iso ok` for both probes with no `iso FAIL` (the same
VA maps to a private frame in each space), interleaved `tick` markers from
both ids (concurrent execution), two `ASRELEASE` lines (spaces reclaimed),
and a clean shutdown.
