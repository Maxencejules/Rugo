# SMP Groundwork Contract v1

Status: live runtime (boot-verified)
Source: `kernel_rs/src/smp.rs`
Proof: `make test-smp-v1`, `tests/runtime/test_smp_runtime_v1.py`

Slice of gap-analysis item 10 ("SMP bring-up + kernel locking"): every
application processor is brought under kernel control and proven alive;
scheduling stays on the BSP — that boundary is the contract.

## Mechanics

- A Limine SMP request enumerates the CPUs. For each non-BSP CPU the
  kernel writes `goto_address`, releasing the AP into `ap_entry` with
  its own Limine-provided stack and the kernel's page tables.
- Each AP runs real kernel code — an atomic check-in
  (`AtomicU64::fetch_add`) — and parks (`cli; hlt`). APs never print
  (the serial path is not multi-CPU safe) and never touch non-atomic
  kernel state.
- The BSP waits (bounded spin) for all check-ins and reports.

## Markers

| Marker | Meaning |
|---|---|
| `SMP: cpus=0x<n>` | CPU count from the Limine SMP response (1 if absent) |
| `SMP: aps online=0x<n>` | APs that executed kernel code and parked |

## Multicore-exposed protocol fix

Bringing up `-smp` surfaced a real service-manager bug: the start-wait
loop treated only ready/failed as terminal, but with buffered input a
session service can sprint ready→stopping→stopped between manager
polls. Single-CPU interleaving always observed the transient ready;
multicore scheduling does not. `stateStopped` is now terminal-success
in the start wait (the reap path handles the rest), and the default
lane boots and shuts down cleanly on multicore (second test in the
proof file).

## v1 carry-forward

Per-CPU run queues and kernel locking (the kernel's single-CPU
assumption is pervasive: IF=0 critical sections, global task table),
IPI plumbing, per-CPU GDT/TSS/IDT state, x2APIC.
