# SMP spinlock — contract v1

Status: boot-verified via `make test-smp-runtime-v1` (boots `-smp 4` and `-smp 2`)
Source: `kernel_rs/src/smp.rs` (`SMP_LOCK`, `smp_lock_acquire`/`release`,
`smp_lock_hammer`, the `ap_entry` + `smp_init` contention pass).
Proof: `tests/runtime/test_smp_runtime_v1.py`.

Full-OS implementation guide Part I.3 (SMP), kernel-locking **and IPI** slice —
the first *real* multi-CPU mutual-exclusion primitive (validated under genuine
contention across application processors) plus a working inter-processor
interrupt (the prerequisite for TLB shootdown and a per-CPU scheduler).

## Behaviour

`SMP_LOCK` is a test-and-set spinlock (`AtomicU32`, acquire/release
`compare_exchange`). `smp_lock_hammer` performs `SMP_LOCK_ITERS` (2000)
**deliberately non-atomic** increments of a shared `SMP_GUARDED: u64`, each
under the lock (volatile read-modify-write so the compiler cannot fuse them).

At boot (`smp_init`, on the BSP, before the PMM/heap/scheduler):
1. every AP is released and immediately runs `smp_lock_hammer` in `ap_entry`
   **before** checking in on `APS_ONLINE`;
2. the BSP runs `smp_lock_hammer` concurrently;
3. the BSP waits until all APs have checked in (so all locked increments are
   committed) and verifies `SMP_GUARDED == cpus * SMP_LOCK_ITERS`, emitting
   `SMP: lock count=0x<total> ok` (or `FAIL`).

Because the counter is non-atomic, a broken lock would lose updates under real
contention and the total would be short — so `ok` is a true mutual-exclusion
proof, not a tautology. On a single CPU (`-smp 1`, the default lane) the total
is `1 * 2000 = 0x7D0`; on `-smp 4` it is `0x1F40`; on `-smp 2`, `0xFA0`.

## IPI (inter-processor interrupt)

After the lock self-test, `smp_init` (gated `cpu_count > 1`) brings up real
cross-CPU signalling:

- Each AP, in `ap_entry`, loads the **kernel GDT** (`gdt_init` — Limine hands
  the AP its own GDT, so the kernel CS `0x08` used by the IDT gates must be made
  valid first), then the shared **IDT** (`load_idt`), enables its **x2APIC**
  (`IA32_APIC_BASE` bits 10/11 + SVR software-enable), checks in, and parks with
  `sti; hlt`.
- The BSP enables its own x2APIC and broadcasts a fixed IPI on vector 240
  (`isr_stub_240` → `trap_handler` → `ipi_handler`, the EOI via the x2APIC EOI
  MSR), then waits for every AP to acknowledge — emitting
  `SMP: ipi ack=0x<n>` (`n` = number of APs).

`smp_init` runs **after** `gdt_init`/`idt_init` in `kmain` so the AP loads a
populated IDT. The whole path is gated on `cpu_count > 1`, so the default
`-smp 1` lanes never touch the LAPIC (verified: the `-smp 2` go-lane still
preempts and shuts down cleanly, so enabling the BSP LAPIC does not disturb
PIC-delivered timer interrupts). Requires CPU x2APIC support
(`-cpu qemu64,+x2apic`).

## v1 boundary / carry-forward

- The spinlock and IPI are exercised by the boot self-test; the spinlock is
  **not yet used to guard the kernel's shared data structures** (scheduler
  tables, PMM, dmesg/audit rings), and the IPI is not yet used for TLB
  shootdown. Wiring them into those paths — plus a **per-CPU run queue /
  scheduler** and per-CPU GDT/TSS — is the carry-forward that makes the kernel
  genuinely SMP-concurrent (today the scheduler still runs only on the BSP; APs
  park after acknowledging the IPI).

## Acceptance

`make test-smp-runtime-v1`: booted `-smp 4`, the transcript shows
`SMP: cpus=0x..04`, `SMP: aps online=0x..03`,
`SMP: lock count=0x0000000000001F40 ok`, and `SMP: ipi ack=0x..03` (all three
APs acknowledged the IPI); booted `-smp 2` (go lane), the count is `0x..0FA0 ok`,
the IPI ack is `0x..01`, and the lane still reaches `GOSH: session ready` and
shuts down cleanly (preemption intact).
