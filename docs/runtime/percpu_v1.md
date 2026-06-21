# Per-CPU data via GS base — contract v1

Status: boot-verified via `make test-smp-runtime-v1` (boots `-smp 4` and `-smp 2`)
Source: `kernel_rs/src/smp.rs` (`PerCpu`, `PERCPU`, `PERCPU_NEXT`, `percpu_init`,
the `gs:` access in `lapic_timer_handler`, the BSP verification in `smp_init`).
Proof: `tests/runtime/test_smp_runtime_v1.py`.

Full-OS implementation guide Part I.3 (SMP), the per-CPU-storage slice — the
addressing model a per-CPU scheduler needs. With per-process address spaces and
APs that run real kernel code, each CPU needs its own `current` task, run queue,
and stats reachable from an interrupt handler **without** a lock or a "which CPU
am I" lookup. x86-64 provides exactly this through the **GS base**: point each
CPU's `IA32_GS_BASE` at its own data block and `gs:`-relative loads/stores reach
that CPU's block automatically.

## Behaviour

- `PERCPU` is an array of `PerCpu { cpu_index, timer_ticks }` slots; slot 0 is
  reserved for the BSP, and APs claim slots 1.. via the atomic `PERCPU_NEXT`.
- Each AP, in `ap_entry` (after enabling its x2APIC, before arming its LAPIC
  timer), calls `percpu_init(slot)`: it writes `IA32_GS_BASE` = `&PERCPU[slot]`
  and then records its index by storing **through GS** (`mov gs:[0], slot`). The
  store-through-GS is the load-bearing proof: a wrong base would land the write
  in another slot.
- The per-CPU LAPIC-timer ISR (`lapic_timer_handler`) increments **this CPU's**
  `timer_ticks` through GS (`add gs:[8], 1`) — no lock and no CPU-id lookup,
  because the GS base already selects the right slot. This is the exact access
  pattern `current`/run-queue manipulation uses.
- The BSP verifies, after the AP timers are confirmed firing, that every online
  AP's slot holds the index that AP wrote through GS (`PERCPU[s].cpu_index == s`),
  emitting `SMP: percpu ok`. `cpu_index` is written once before the AP's SeqCst
  check-in, so the BSP's later read is ordered after it (no data race); the
  still-incrementing `timer_ticks` is deliberately not read.

The BSP's own GS base is **left untouched**: in the go lane the BSP runs
userspace (TinyGo), and only APs take the LAPIC-timer vector, so no kernel `gs:`
access ever executes without a base set.

## v1 boundary / carry-forward

- **Bounds + degraded boots.** Slots are capped at `MAX_CPUS` (64); a surplus AP
  (more CPUs than slots) sets neither its GS base nor its timer, so the ISR never
  runs `gs:` without a base. The BSP only runs the verification on the success
  path (every AP checked in), so it never races an AP still writing its slot.
- v1 proves the **mechanism** (per-CPU GS addressing, used by an ISR). The slots
  do not yet hold a real `current` task pointer or run queue, and the BSP is not
  yet given a populated slot 0. Wiring a per-CPU `current` + per-CPU run queue
  (and per-CPU GDT/TSS so an AP can take a ring3→ring0 trap on its own kernel
  stack) is the remaining capstone that lets the scheduler run user tasks on the
  APs instead of parking them (status doc item 1).

## Acceptance

`make test-smp-runtime-v1`: booted `-smp 4`, the transcript shows `SMP: percpu ok`
after `SMP: tlb shootdown ok` (all 3 APs recorded their index through their own
GS base); booted `-smp 2` (go lane), the single AP records its index
(`SMP: percpu ok`) and the lane still reaches `GOSH: session ready` and shuts
down cleanly (the BSP's GS base was not disturbed, so userspace is unaffected).
