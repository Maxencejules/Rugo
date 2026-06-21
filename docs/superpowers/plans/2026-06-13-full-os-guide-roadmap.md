# Full-OS implementation guide — execution roadmap

Date: 2026-06-13
Tracks: [`docs/analysis/full-os-implementation-guide.md`](../../analysis/full-os-implementation-guide.md)
Branch: `feat/full-os-keystone-addrspace` (off `master`)

This is the live execution roadmap for implementing the full-OS guide. Every
landed item is a real runtime feature with a `make test-<name>-v1` boot-verified
marker test, a `docs/runtime/<name>_v1.md` contract, and (for new syscall ids)
an entry in `docs/abi/syscall_v3.md`. Same discipline as the gap-closure effort:
one unified `go_test` product lane, marker QEMU tests, no seeded JSON, additive
ABI, zero external crates, asm probe apps (gousr.bin is near its 8-page cap).

## ABI v3.2 id map (as built)

| id | syscall | ops |
|----|---------|-----|
| 48 | sys_signal_ctl | register / kill / sigreturn |
| 49 | sys_net_query | dhcp / dns / poll |
| 50 | sys_vm_ctl | mmap / munmap / brk / mprotect |
| 51 | sys_proc_ctl | fork / clone |
| 52 | sys_futex | wait / wake |
| 53 | sys_time | clock_gettime / nanosleep |
| 54 | sys_getrandom | — |
| 56 | sys_ioctl | fb blit (generic device control) |
| 58 | sys_power | shutdown / reboot |
| 59 | sys_sandbox | syscall allowlist |
| 61 | sys_sysinfo | tasks / free-frames / uptime |
| fs_ctl(47) | +op 6 | lseek |

Free per guide §0.2: 55 (mount), 57 (poll-rich), 60 (dlctl). 62 reserved,
63 = v4 escape.

## Done (boot-verified, committed)

Spanning all 5 guide parts:

- **I.1 per-process address spaces** (keystone) — `6d22295`, `per_task_as_v1.md`
- **I.2 fork + copy-on-write** — `f0d4119`, `fork_v1.md`
- **I.3 clone threads + futex** — `72e0061`, `futex_v1.md`
- **I.4 mmap/brk/munmap + mprotect + PTE_COW** — `14fb689`/`9141df3`, `mmap_v1.md`
- **II.5 /dev pseudo-fs + 32-app region** — `37beb70`, `pseudo_fs_v1.md`
- **II.5 /proc/self/stat** — `b2701c5`
- **II.6 DHCP full DORA** — `c6471c5`, `netcfg_v2.md`
- **III graphics (framebuffer blit)** — `48a93fd`, `graphics_v1.md`
- **IV.9 clock_gettime** — `1c9821e`, `clock_v1.md`
- **IV.9 nanosleep + scheduler idle/wait-queue infra** — `d0f8fc0`
- **IV.9 power/ACPI shutdown+reboot** — `aa7c753`, `power_v1.md`
- **IV.10 getrandom / CSPRNG** — `a10a808`, `rng_v1.md`
- **IV.10 sandbox / syscall allowlist** — `d0dae42`, `sandbox_v1.md`
- **V sysinfo metrics** — `e556a92`, `sysinfo_v1.md`
- **V lseek** — `f38b5c2`, `lseek_v1.md`

- **IV.9 timerfd** — `c5854e9`, `clock_v1.md` (sys_time op 3)
- **II.7 PCI device enumeration** — `3c59d83`, `driver_model_v1.md`
- **IV.10 stack ASLR** — `d6ef4da`

Key infra unlocked: per-task CR3 + private address spaces; CoW refcounts;
PTE_COW software bit; a CoW-aware user-write path (copyout breaks CoW); a real
scheduler **idle loop** (`r4_idle_loop` + `r4_enter_idle_or_done` +
`r4_wake_sleepers`) that parks safely when only a timed wakeup is pending — the
prerequisite for nanosleep/timerfd/blocking reads.

## Verification & hardening

- Full `make test-qemu` gate: **854 passed** across all ~50 lanes (run twice;
  once after the slices, once after the review fixes — no regressions).
- **Adversarial review** (ultracode review→verify workflow over the diff) found
  **7 real latent bugs** the boot tests never exercised — all fixed in `0047e55`:
  nanosleep RAX clobber, fork promoting RO pages, waitpid status cross-AS write
  (+ a CoW-aware user-write path), futex wake(0) aliasing, IPC cross-AS delivery,
  orphan clone-zombie leak, an ABI-doc register error. Regression: `test-waitpid-v1`.
- Lesson: boot-marker tests prove it boots, not memory/concurrency safety; run an
  adversarial review over subtle memory/scheduler code.

## Remaining (large subsystems — focused sessions, careful regression mgmt)

Ordered by dependency / value:

1. **I.3 SMP** per-CPU scheduling + ticket spinlocks + IPIs + TLB shootdown (XL).
   Implement in phases: spinlocks → per-CPU GDT/TSS/IDT → per-CPU LAPIC timer →
   IPIs → per-CPU run queues. APs currently park.
2. **II.5 FS maturity rest** — write-ahead journal + replay, block/buffer cache
   (LRU), mount table + FAT12/16 read, MBR/GPT partitions, /tmp tmpfs, per-`<tid>`
   /proc. Modifies `vfs.rs` — preserve `test-vfs-v1`.
3. **II.6 TCP maturity** — per-connection state, retransmit/RTO, wire listen/accept,
   ICMP echo, routing-table connect, IPv6 on the wire, interrupt-driven RX.
   Modifies `tcp.rs`/`net.rs` — preserve `test-tcp-v1`.
4. **II.7 driver model** — device/driver registry, PCIe ECAM, DMA pool, refactor
   virtio/NVMe into it; then USB/XHCI+HID, e1000.
5. **III rest** — input (PS/2 + USB-HID mouse, extended scancodes, input ring +
   `sys_input_poll`), compositor + damage tracking + window server, audio
   (AC97/virtio-snd `sys_audio_write`).
6. **IV.9/10 rest** — timerfd (now buildable on the idle infra), ASLR (randomize
   spawned stack base via the RNG), audit log, multi-user login service +
   `/etc/passwd`, measured/secure boot, in-repo AEAD for TLS.
7. **V.11 userspace** — rlibc v2 (errno, buffered FILE*, env vars), TTY/pty + job
   control, dynamic linker (.so / PIE / GOT-PLT), package manager fetch/update,
   installer, UEFI/bare-metal, crash dumps, dmesg/syslog, self-hosting.

## Process notes

- Prefer ADDITIVE slices (new syscall/op/fd-kind + asm probe) over modifying
  working subsystems; when a working subsystem is touched, keep its existing test.
- Adding an `M8FdKind` variant requires arms in `m10_rights_for_kind` (exhaustive)
  + `sys_read_v1`/`sys_write_v1`/`sys_poll_v1`, gated `all(go_test, not compat)`.
- Do NOT `sti; hlt` inside a syscall (nested IRQ on the syscall stack hangs);
  reach the idle loop through the trap frame instead.
- Run the full `make test-qemu` (~30 min, ~50 lanes) before merging to `master`.
