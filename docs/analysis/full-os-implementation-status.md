# Rugo — full-os-implementation-guide: implementation status

Date: 2026-06-14
Companion to [`full-os-implementation-guide.md`](full-os-implementation-guide.md).
Branch: `feat/full-os-keystone-addrspace`. Every item below is **boot-verified**
(a `make test-<name>-v1` target booting under QEMU and asserting serial/screen
markers emitted by runtime code) with a `docs/runtime/<name>_v1.md` contract and,
where it added an ABI id/op, an entry in `docs/abi/syscall_v3.md`. Validated by
repeated full `make test-qemu` gates (latest: **876 passed / 0 failed**).

This document records what is implemented at **v1** and gives **concrete next
steps** for the subsystems the guide tags L/XL that remain as carry-forward.

## Implemented (v1) — by guide part

### Part I — Foundations
- **Per-process address spaces** (keystone): per-task PML4, CR3 reload in
  `r4_switch_to`, kernel-half cloning, release on exit. `per_task_as_v1.md`.
- **fork / clone + copy-on-write**: `sys_proc_ctl` op 1/2, `COW_REFCOUNT`,
  `cow_break`. `fork_v1.md`.
- **mmap / brk / munmap / mprotect**: `sys_vm_ctl`. `mmap_v1.md`.
- **SMP bring-up + spinlock**: APs released via Limine, run real kernel code; a
  test-and-set spinlock guards a counter every CPU hammers under contention
  (`-smp 4` → 8000 increments, zero lost updates). `smp_lock_v1.md`.

### Part II — Core subsystems
- **II.5 Filesystem**: tmpfs `/tmp` (`pseudo_fs_v1.md`), MBR partition parse
  (`partitions_v1.md`), FAT16 read + `/mnt` namespace mount + directory list
  (`fat16_v1.md`), write-ahead journaling + replay (`journal_v1.md`); plus the
  pre-existing SimpleFS `/data` tree and `/dev`, `/proc/self/stat`.
- **II.6 Networking**: DHCP/DNS clients (`netcfg_v1.md`), and a comprehensively
  reachable host — ARP responder (`arp_v1.md`), ICMP echo (`icmp_v1.md`),
  ICMPv6 echo (`icmpv6_v1.md`), TCP client **and** passive-open/listener
  (`tcp_listen_v1.md`), UDP echo (`udp_echo_v1.md`).
- **II.7 Drivers**: PCI enumeration + driver registry/ATTACH (`driver_model_v1.md`).

### Part III — Human interface
- Framebuffer blit (`graphics_v1.md`), PC speaker audio (`audio_v1.md`).

### Part IV — System services
- **IV.9**: clock_gettime, nanosleep + scheduler idle/wait-queue, timerfd,
  power/ACPI (`clock_v1.md`, `power_v1.md`, …).
- **IV.10**: getrandom (`rng_v1.md`), stack ASLR, sandbox (`sandbox_v1.md`),
  security audit log (`audit_v1.md`), at-rest disk encryption
  (`disk_crypt_v1.md`), multi-user getuid/setuid (`userid_v1.md`).

### Part V — Userspace & operations
- dmesg ring (`dmesg_v1.md`), pty pair (`pty_v1.md`), /proc-style sysinfo,
  lseek (`lseek_v1.md`), rlibc v1 (POSIX-ish C library), multi-page exec.

## Carry-forward — the L/XL subsystems (with concrete next steps)

These each require dedicated, infra-heavy work; they do not decompose into a
single safe boot-verified slice and several have hard prerequisites.

1. **I.3 per-CPU scheduler — primitives + TLB shootdown DONE, run queue remains.**
   The spinlock (locking), a working x2APIC IPI, per-CPU LAPIC timers, **and**
   cross-CPU TLB shootdown are implemented (`smp_lock_v1.md`,
   `tlb_shootdown_v1.md`): every AP runs its own periodic preemption clock
   (`SMP: ap timers ok`) and acknowledges directed TLB invalidations
   (`SMP: tlb shootdown ok`). The spinlock+IPI details:
   the BSP broadcasts vector 240 to the APs, which acknowledge (`SMP: ipi ack`);
   the BSP broadcasts vector 242 for shootdown, the APs `invlpg`+ack.
   The four bugs the first attempt hit, now fixed: (a) the AP must `gdt_init`
   (Limine hands it its own GDT) before loading the IDT; (b) `-cpu qemu64,+x2apic`
   is required (plain qemu64 #GPs on the x2APIC enable); (c) IDT vectors 240–242
   must be installed in **every** lane (the `-smp 4` test uses the base `os.iso`,
   not `go_test`) — including the LAPIC spurious vector 65 (review fix); (d)
   `smp_init` must run **after** `idt_init` in `kmain` so a released AP loads a
   populated IDT. Gating on `cpu_count > 1` keeps `-smp 1` lanes off the LAPIC;
   verified the `-smp 2` go-lane still preempts. Three adversarial-review bugs in
   the triad were fixed (unconditional vector-65 gate, two-step x2APIC enable,
   success-gated lock-count read). **Remaining:** wire `tlb_shootdown` into the
   `munmap`/`mprotect`/CoW paths and build a per-CPU run queue + per-CPU GDT/TSS
   so the scheduler runs user tasks on the APs instead of parking them.
2. **II.7 USB / XHCI + HID, DMA pool, e1000** — needs `-device qemu-xhci` (and
   `-device e1000`) in a dedicated test profile, then an XHCI controller driver
   (command/event rings, port reset, device enumeration) and a HID boot-protocol
   driver. DMA pool = a contiguous-frame allocator over the PMM (the bitmap
   allocator is single-frame today).
3. **III input + compositor/window-server + richer audio** — PS/2 mouse needs
   QMP `input-send-event` injection (the `_boot` fixture only feeds a fixed input
   string; add a QMP-capable boot helper like `tests/runtime/test_smp_runtime_v1.py`
   builds its own QEMU cmd). The window-server needs concurrent processes (have)
   + shared-memory surfaces + an input event queue.
4. **V.11 dynamic linker / .so** — **blocked** on the PE→ELF toolchain: mingw’s
   refptr/auto-import + the homemade `tools/pe_to_elf_v1.py` break C binaries that
   cross 2 pages (proved via `page3probe`: the kernel handles 3-page apps; the
   toolchain does not). Fix `pe_to_elf` (or switch the C apps to a real ELF
   linker) first, then add `sys_dlctl` (map-segment/resolve) + ELF dynamic
   relocation.
5. **V.11 installer + UEFI + package fetch + self-hosting** — UEFI is a second
   Limine boot path; the installer writes the SimpleFS/app-region image to a
   target disk; package fetch needs the TCP client (have) + a repo server.
6. **II.6 TCP congestion control, IPv6 SLAAC, routing** — the client/listener,
   v4/v6 echo, IPv6 **Neighbor Discovery responder** (`ndp_v1.md`), and **TCP
   retransmission/RTO** (`tcp_rto_v1.md`: arm-on-send, ACK-clears, exponential
   backoff, give-up after max retries) exist. What remains: RTT estimation
   (SRTT/RTTVAR, Karn), fast retransmit + congestion control (slow start/cwnd)
   and a real send window beyond one outstanding segment; the guest *sending* its
   own NDP solicitations + a neighbor cache (NUD) + DAD on its own address; SLAAC
   / Router Discovery for a global address; and routing.

## ABI op map (current)
- `sys_net_query` (49): 1 DHCP, 2 DNS, 3 poll, 4 ICMP, 5 ARP, 6 TCP-listen,
  7 ICMPv6, 8 UDP-echo (4–8 are self-tests).
- `sys_ioctl` (56): 1 fb-blit, 2 openpty, 3 beep.
- `sys_sysinfo` (61): 1 tasks, 2 free-frames, 3 uptime, 4 dmesg, 5 MBR,
  6 FAT-read, 7 audit, 8 FAT-list, 9 disk-crypt, 10 journal.
- `sys_proc_ctl` (51): 1 fork, 2 clone, 3 getuid, 4 setuid.
