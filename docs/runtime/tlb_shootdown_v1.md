# TLB shootdown — contract v1

Status: boot-verified via `make test-smp-runtime-v1` (boots `-smp 4` and `-smp 2`)
Source: `kernel_rs/src/smp.rs` (`tlb_shootdown`, `tlb_shootdown_handler`,
`tlb_invalidate`, `SHOOTDOWN_ADDR`/`SHOOTDOWN_ACK`/`SMP_AP_COUNT`), vector 242
(`arch/x86_64/isr.asm`, `arch_x86::idt_init`, `trap::trap_handler`).
Proof: `tests/runtime/test_smp_runtime_v1.py`.

Full-OS implementation guide Part I.3 (SMP), the cross-CPU TLB-invalidation
slice — built directly on the (adversarially reviewed) x2APIC IPI. This is the
mechanism the VM (`munmap`/`mprotect`/CoW page-table edits) and a per-CPU
scheduler require: when one CPU changes a page-table entry, every other CPU that
may have cached the old translation must drop it before the change is safe.

## Behaviour

A shootdown is a directed IPI on vector **242**:

1. The initiator (the BSP today) calls `tlb_shootdown(addr)`. It first
   invalidates `addr` in **its own** TLB (`invlpg`, or a `CR3` reload when
   `addr == 0` to request a full flush).
2. It publishes `addr` in `SHOOTDOWN_ADDR`, snapshots the `SHOOTDOWN_ACK`
   counter, and broadcasts vector 242 to **all CPUs except itself** (x2APIC ICR
   destination shorthand "all-but-self").
3. Each AP takes the interrupt (`isr_stub_242` → `trap_handler` → 
   `tlb_shootdown_handler`), reads `SHOOTDOWN_ADDR`, runs the same
   `invlpg`/`CR3`-reload, increments `SHOOTDOWN_ACK`, and EOIs.
4. The initiator spins (bounded) until `SHOOTDOWN_ACK` has advanced by the
   number of online APs (`SMP_AP_COUNT`, recorded in `smp_init` once the APs
   check in), then returns `true`. On a uniprocessor — or before the APs are up
   — `SMP_AP_COUNT == 0` and the call is purely the local invalidation.

`SHOOTDOWN_ADDR` uses release/acquire ordering and `SHOOTDOWN_ACK` is `SeqCst`,
so an AP that observes the new address has the initiator's store visible and the
initiator only returns once every AP's increment is visible.

## v1 boundary / carry-forward

- **What is proven:** the *directed cross-CPU invalidation path* — the BSP can
  make every AP execute a TLB invalidation for an initiator-chosen address and
  acknowledge it, with the initiator blocking until all acks land. `invlpg`
  itself is architecturally guaranteed to drop the entry; the self-test verifies
  the plumbing (publish → broadcast → per-AP invalidate → ack → wait), not that
  a specific stale entry was observed and corrected (the parked APs run no user
  workload that would cache a user translation to go stale).
- **Single shared request slot.** One `SHOOTDOWN_ADDR` and a BSP-only initiator
  are safe for the boot self-test and for callers serialized by the kernel's big
  lock. A fully concurrent kernel needs per-CPU shootdown mailboxes (or a lock
  around the request) so two simultaneous initiators cannot clobber the address.
- **Not yet wired into the VM paths.** `sys_vm_unmap`/`mprotect`/`cow_break`
  still do a local `invlpg` only; that is correct *today* because the scheduler
  runs user code on the BSP alone, so no AP caches a user translation. Once the
  per-CPU scheduler runs user tasks on APs, those paths must call
  `tlb_shootdown` — the API is in place for exactly that.

## Acceptance

`make test-smp-runtime-v1`: booted `-smp 4`, the transcript shows
`SMP: tlb shootdown ok` after `SMP: ap timers ok` (the BSP directed all 3 APs to
invalidate and every one acknowledged); booted `-smp 2` (go lane), the single AP
acknowledges (`SMP: tlb shootdown ok`) and the lane still reaches
`GOSH: session ready` and shuts down cleanly.
