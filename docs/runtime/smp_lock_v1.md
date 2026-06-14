# SMP spinlock — contract v1

Status: boot-verified via `make test-smp-runtime-v1` (boots `-smp 4` and `-smp 2`)
Source: `kernel_rs/src/smp.rs` (`SMP_LOCK`, `smp_lock_acquire`/`release`,
`smp_lock_hammer`, the `ap_entry` + `smp_init` contention pass).
Proof: `tests/runtime/test_smp_runtime_v1.py`.

Full-OS implementation guide Part I.3 (SMP), kernel-locking slice — the first
*real* multi-CPU mutual-exclusion primitive, validated under genuine contention
across application processors (which the existing bring-up groundwork only
parked).

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

## v1 boundary / carry-forward

- The spinlock is exercised by the boot self-test; it is **not yet used to
  guard the kernel's shared data structures** (scheduler tables, the PMM, the
  dmesg/audit rings). Wiring it into those hot paths — and a per-CPU run
  queue / scheduler — is the carry-forward that makes the kernel genuinely
  SMP-concurrent (today the scheduler still runs only on the BSP; APs park
  after the contention pass).
- No IPIs / TLB shootdown / per-CPU GDT-TSS yet.

## Acceptance

`make test-smp-runtime-v1`: booted `-smp 4`, the transcript shows
`SMP: cpus=0x..04`, `SMP: aps online=0x..03`, and
`SMP: lock count=0x0000000000001F40 ok`; booted `-smp 2` (go lane), the count
is `0x..0FA0 ok` and the lane still shuts down cleanly.
