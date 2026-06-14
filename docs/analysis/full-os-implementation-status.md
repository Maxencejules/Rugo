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
  (`partitions_v1.md`), FAT16 read + `/mnt` namespace mount + directory list +
  **single-cluster file write** (`fat16_v1.md`; write allocates a free cluster,
  marks it EOC in every FAT copy, fills a free root-dir entry, verified by
  read-back: `FATWR: write+read ok`), write-ahead journaling + replay
  (`journal_v1.md`); plus the pre-existing SimpleFS `/data` tree and `/dev`,
  `/proc/self/stat`.
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

1. **I.3 per-CPU scheduler — primitives + TLB shootdown + per-CPU GS DONE, run queue remains.**
   The spinlock (locking), a working x2APIC IPI, per-CPU LAPIC timers,
   cross-CPU TLB shootdown, **and** per-CPU GS-base storage are implemented
   (`smp_lock_v1.md`, `tlb_shootdown_v1.md`, `percpu_v1.md`): every AP runs its own periodic preemption clock
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
   success-gated lock-count read). Per-CPU GS storage (`SMP: percpu ok`): each AP
   points `IA32_GS_BASE` at its own `PerCpu` slot, records its index through GS,
   and its LAPIC-timer ISR bumps a per-CPU counter through GS — lock-free per-CPU
   addressing (the BSP's GS base is left untouched so the go lane's userspace is
   undisturbed). **Cross-CPU work dispatch** is also in place (`ap_work_v1.md`):
   the BSP hands a kernel work item to the APs, exactly one AP claims it (atomic
   CAS) and runs it on its own core, reporting the result (`SMP: ap work ok`).
   The APs now run real dispatched kernel work, not just park. **Capstone DONE —
   a ring-3 USER task runs on an application processor** (`smp_user_task_v1.md`):
   the single shared TSS became **per-CPU** (one TSS + private `rsp0`/kernel stack
   per CPU, descriptors at `GDT[5+2·slot]`, selector `0x28+0x10·slot`), so an AP
   takes a ring-3→ring-0 trap on its **own** kernel stack. The BSP builds a
   private address space (the spawned-app `mm` path) with a ring-3 payload,
   dispatches it (work kind 2); an AP claims it, loads that CR3, `iretq`s to ring
   3 with the arg in RDI, runs the code on its own core, reports `arg·2+1` via a
   DPL=3 gate (`int 0x81`), and is trampolined back into the kernel on its own
   stack — `SMP: ap user task ok` (result verified + reporting CPU slot ≥ 1, the
   BSP never enters ring 3). The kernel CR3 is restored before `WORK_DONE` is
   published, so the BSP safely reclaims the address space (no UAF/leak).
   **Remaining (turning the mechanism into a running multi-CPU scheduler):** wire
   `tlb_shootdown` into the `munmap`/`mprotect`/CoW paths; turn the single-item
   work mailbox into per-CPU run queues holding `current`/tasks; migrate R4 tasks
   onto APs with per-CPU `current`; and make the scheduler/syscall/page-fault
   paths SMP-safe on every core (today the BSP's are interrupts-off single-CPU) so
   ordinary user tasks are scheduled across all CPUs, not just the boot self-test.
2. **II.7 USB / XHCI + HID, DMA pool, e1000 — detection + DMA pool done.** The OS
   discovers a USB xHCI host controller (`-device qemu-xhci`, `xhci_v1.md`) and an
   Intel **e1000 NIC** (`-device e1000`, `e1000_v1.md`: maps BAR0, reads STATUS +
   the MAC out of the EEPROM via EERD), maps device MMIO (`mmio_map_4k`), and has
   a **DMA allocator** (`dma_v1.md`: a contiguous-frame pool carved from the PMM,
   first-fit `dma_alloc`/`dma_free`, self-test `DMA: selftest ok`), and reads PCI
   config space through **PCIe ECAM** (`ecam_v1.md`: the memory-mapped window from
   the q35 PCIEXBAR, cross-checked against the legacy I/O path). What remains: an
   XHCI controller **driver** (command/event rings, port reset, device
   enumeration) + a HID boot-protocol driver; an e1000 TX/RX-ring driver (built on
   the DMA pool); MSI/MSI-X; and migrating the virtio/NVMe probes onto `dma_alloc`
   + routing all config access through ECAM.
3. **III input + compositor/window-server + audio — mouse, z-order compositor, HD-Audio detection done.**
   The mouse device is reset + identified at boot (`mouse_v1.md`), the compositor
   composites multiple surfaces to the framebuffer in **z-order**
   (`compositor_v1.md`, verified via QMP screendump), the PC speaker beeps
   (`audio_v1.md`), and an Intel **HD Audio** controller is discovered + its
   GCAP/version read (`hda_v1.md`, `-device intel-hda`). What remains: HDA
   CORB/RIRB rings + codec enumeration + **PCM playback** (on the DMA pool);
   continuous mouse movement/button packets (needs QMP `input-send-event`
   injection); per-client **shared-memory pixel surfaces**, damage regions, alpha;
   a standing compositor **process**; and an input event queue routing clicks.
4. **V.11 dynamic loading — real ELF `.so` dynamic linker done.**
   `sys_dlctl` (id 60) is a genuine ELF64 dynamic linker (`dynlink_v1.md`):
   `dlopen` parses the embedded `.so`'s program headers, maps each `PT_LOAD`
   segment at `DLOPEN_BASE+p_vaddr` (map RW → `copyout_user` file bytes → apply
   relocations → re-protect per `p_flags` with W^X), **applies its
   `R_X86_64_RELATIVE` relocations** from `PT_DYNAMIC`, and `dlsym` resolves a
   name from the `.dynsym`/`.dynstr` (count via the SysV `.hash` `nchain`).
   `dlprobe` calls `getval()` (returns 42 *only if* the relative relocation was
   applied) and `addtwo(40)==42` in ring 3 (`DLPROBE: dlsym ok`). The C `.so`
   toolchain blocker (mingw refptr/auto-import + `tools/pe_to_elf_v1.py` break C
   binaries past 2 pages, proved via `page3probe`) is **routed around**: the
   shared object is authored in assembly (`apps/dl/libdl.asm`) and linked as a
   real PIC ELF `.so` via `nasm -f elf64` + `rust-lld -shared` (`make dl-module`),
   so the linker is exercised on a genuine `.so` without the C path. What remains:
   symbolic relocations (`GLOB_DAT`/`JUMP_SLOT` → GOT/PLT, lazy binding),
   `DT_NEEDED` dependency chains, multiple loaded objects + `dlclose`, on-disk
   `.so` loading, and an allocator-chosen load base (v1 uses one fixed slot).
5. **V.11 installer + UEFI + package fetch + self-hosting — disk provisioning + UEFI boot done.**
   The installer finds a target disk, writes a boot record, and verifies the
   write/read round-trip (`installer_v1.md`, confirmed host-side). The kernel also
   **boots under UEFI** (`uefi_boot_v1.md`: OVMF/edk2 → Limine `BOOTX64.EFI` →
   `RUGO: boot ok` → clean shutdown, identical to the BIOS lane — the bring-up is
   firmware-agnostic via Limine requests). What remains: a full bootable install
   (partition table, copy the kernel + a SimpleFS/app-region image onto a target
   partition, install the bootloader so the target boots standalone); folding a
   UEFI El-Torito entry into the ISO build so `os-go.iso` is itself hybrid (needs
   xorriso/mtools); Secure Boot; package fetch over the TCP client (have) + a repo
   server; and self-hosting.
6. **II.6 TCP reliability — RTO + RTT estimation + congestion control done.** The
   client/listener, v4/v6 echo, IPv6 **Neighbor Discovery responder**
   (`ndp_v1.md`), **TCP retransmission/RTO** (`tcp_rto_v1.md`), **RTT estimation**
   (`tcp_rtt_v1.md`: RFC 6298 SRTT/RTTVAR in integer fixed-point + Karn's
   algorithm, driving an adaptive RTO), and **congestion control**
   (`tcp_congestion_v1.md`: RFC 5681 slow start + congestion avoidance + timeout
   collapse), **longest-prefix-match routing** (`routing_v1.md`), and a
   **guest-initiated IPv6 neighbor cache / NUD** (`nud_v1.md`: the guest sends its
   own Neighbor Solicitation + caches the MAC from the returning advertisement)
   exist. What remains: a real send window beyond one outstanding segment (cwnd is
   computed but the single-segment send path does not yet clamp to it), fast
   retransmit / fast recovery (3-dup-ACK); the full NUD state machine
   (STALE/DELAY/PROBE timers) + DAD on the guest's own address; SLAAC / Router
   Discovery for a global address; and per-route gateway resolution before send.

## ABI op map (current)
- `sys_net_query` (49): 1 DHCP, 2 DNS, 3 poll, 4 ICMP, 5 ARP, 6 TCP-listen,
  7 ICMPv6, 8 UDP-echo, 9 NDP, 10 TCP-RTO, 11 TCP-RTT, 12 TCP-congestion,
  13 routing, 14 IPv6-NUD, 15 IPv6-SLAAC, 16 TCP-fast-retransmit (4–16 are
  self-tests).
- `sys_ioctl` (56): 1 fb-blit, 2 openpty, 3 beep, 4 compositor-compose.
- `sys_dlctl` (60): 1 dlopen, 2 dlsym.
- `sys_sysinfo` (61): 1 tasks, 2 free-frames, 3 uptime, 4 dmesg, 5 MBR,
  6 FAT-read, 7 audit, 8 FAT-list, 9 disk-crypt, 10 journal, 11 FAT-write,
  12 FAT-chain-read.
- `sys_proc_ctl` (51): 1 fork, 2 clone, 3 getuid, 4 setuid, 5 login.
- Boot self-tests (markers, no syscall): SMP (spinlock, IPI, per-CPU timers, TLB
  shootdown, per-CPU GS, work dispatch, **ring-3 user task on an AP**), DMA pool,
  block buffer cache, AES-128 (FIPS-197 KAT, backs disk crypto), **SHA-256
  (FIPS 180-4 KAT) + measured-boot PCR**, **2 MiB huge page**, **TTY line
  discipline**, **GPT parse**, **mount table**; PCI detection (xHCI, e1000,
  HD-Audio, **PCIe ECAM**, **MSI-X enable**).
