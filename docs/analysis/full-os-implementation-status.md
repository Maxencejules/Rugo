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
   The APs now run real dispatched kernel work, not just park. **Remaining (the
   capstone, a dedicated core effort):** wire `tlb_shootdown` into the `munmap`/
   `mprotect`/CoW paths; turn the single-item work mailbox into a per-CPU run
   queue holding `current`/tasks; add per-CPU GDT/TSS so an AP takes a ring3->ring0
   trap on its own kernel stack; and make the scheduler/syscall/page-fault paths
   SMP-safe (today they assume one CPU with interrupts-off syscalls) so the
   scheduler can run USER tasks on the APs.
2. **II.7 USB / XHCI + HID, DMA pool, e1000 — controller DETECTION done.** The OS
   now discovers a USB xHCI host controller (`-device qemu-xhci`), maps its BAR
   (the PCI MMIO hole is not in the HHDM, so `mmio_map_4k` walks CR3 and installs
   an uncacheable leaf), and reads its capability registers
   (`xhci_v1.md`: `XHCI: found ver=0x100 caplen=0x40 ports=8 slots=64`). What
   remains: an XHCI controller **driver**
   (command/event rings, port reset, device enumeration) and a HID boot-protocol
   driver. DMA pool = a contiguous-frame allocator over the PMM (the bitmap
   allocator is single-frame today).
3. **III input + compositor/window-server + richer audio — mouse + z-order compositor done.**
   The mouse device is reset + identified at boot (`mouse_v1.md`), and the
   compositor composites multiple surfaces to the framebuffer in **z-order**
   (`compositor_v1.md`: a red window drawn over a blue background, both verified
   present via QMP screendump). What remains: enabling continuous mouse data
   reporting + parsing movement/button packets needs QMP `input-send-event`
   injection (the `_boot` fixture only feeds a fixed keyboard string); per-client
   **shared-memory pixel surfaces** (vs v1 solid-color rects), damage regions,
   alpha; a standing compositor **process** owning the FB; and an input event
   queue routing clicks to the top window.
4. **V.11 dynamic loading — dlopen/dlsym mechanism done; real ELF .so still blocked.**
   `sys_dlctl` (id 60) implements `dlopen`/`dlsym` (`dynlink_v1.md`): it loads a
   position-independent module the kernel ships embedded into a fresh executable
   user region (map RW → copy → mprotect R-X), resolves a symbol from the loaded
   image's export table, and `dlprobe` calls it (`DLPROBE: dlsym ok`). The
   loading + resolution + execute mechanism works. What remains is a real ELF
   **.so** linker (dynamic relocations, GOT/PLT), which is still **blocked** on
   the PE→ELF toolchain: mingw's refptr/auto-import + `tools/pe_to_elf_v1.py`
   break C binaries past 2 pages (proved via `page3probe`). Fix `pe_to_elf` (or
   switch the C apps to a real ELF linker) to produce `.so` files, then extend
   `sys_dlctl` with ELF parsing + relocation.
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
  7 ICMPv6, 8 UDP-echo, 9 NDP, 10 TCP-RTO (4–10 are self-tests).
- `sys_ioctl` (56): 1 fb-blit, 2 openpty, 3 beep, 4 compositor-compose.
- `sys_dlctl` (60): 1 dlopen, 2 dlsym.
- `sys_sysinfo` (61): 1 tasks, 2 free-frames, 3 uptime, 4 dmesg, 5 MBR,
  6 FAT-read, 7 audit, 8 FAT-list, 9 disk-crypt, 10 journal, 11 FAT-write.
- `sys_proc_ctl` (51): 1 fork, 2 clone, 3 getuid, 4 setuid.
- SMP self-tests (boot markers, no syscall): spinlock lock-count, IPI ack,
  per-CPU LAPIC timers, TLB shootdown, per-CPU GS, cross-CPU work dispatch.
