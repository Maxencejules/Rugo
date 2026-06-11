# What Rugo Is Missing To Be A Fully Functioning OS

Date: 2026-06-10
Method: source-level audit of `kernel_rs/src/` (~9,357 LOC Rust), `services/go/`
(~3,103 LOC Go), `services/go_std/` (117 LOC), `arch/`, `boot/`, the QEMU test
layer, and the milestone/qualification docs — compared against Linux, Windows
NT, macOS/XNU, and popular open-source OS projects (FreeBSD, SerenityOS, Redox,
Haiku, xv6).

This document deliberately separates what is **boot-verified runtime code**
from what is **documentation and seeded qualification reports**, following the
repo's own rule in `docs/architecture/SOURCE_MAP.md`.

## 1. Verified Baseline (what Rugo really has today)

| Capability | Evidence |
|---|---|
| x86-64 boot via Limine, paging on, IDT, exceptions | `tests/boot/*`, `tests/trap/*` (serial markers, clean QEMU exit) |
| Ring 3 user mode + `int 0x80` syscalls (28 frozen in ABI v3) | `tests/user/*`, `docs/abi/syscall_v3.md` |
| Cooperative threading (4–6 static task slots); preemptive PIT timer only in the `sched_test` feature build | `kernel_rs/src/lib.rs`, `kernel_rs/src/sched.rs` |
| IPC endpoints (256-byte messages) + 4 KB shared-memory mapping | `tests/ipc/*` |
| VirtIO-blk, VirtIO-net, NVMe drivers (NVMe boot-exercised on q35); AHCI code present, weaker evidence | `kernel_rs/src/runtime/native.rs`, `tests/conftest.py:571,959` |
| Fixed-path file layer (~6 hardcoded paths → block sectors) + SimpleFS images; persistence across reboot verified | `tests/pkg/test_default_shell_app_runtime_v1.py` |
| ARP + IPv4/UDP echo in kernel; socket syscall surface with loopback-style test roundtrips | `tests/net/test_udp_echo.py`, `kernel_rs/src/net.rs` |
| Real Go init/service supervisor (4 services: timesvc, diagsvc, pkgsvc, shell) with restart policies, dependency ordering, graceful shutdown | `services/go/runtime.go`, `services/go/services.go` |
| Interactive serial shell REPL with 11 built-in commands | `services/go/shell_session.go` |
| Package state machine with HMAC-SHA256 verification; external static ELF execution via package path | `services/go/pkgsvc.go`, `tests/pkg/test_pkg_external_apps.py` |
| Static ELF64 loading at fixed addresses | `kernel_rs/src/lib.rs:1742-1811` |

Also real and valuable: the ABI freeze discipline, marker-based QEMU acceptance
testing, and reproducible image tooling. These exceed most hobby OS projects.

**Inflated claims to discount:** of the 54 milestones marked done, roughly 8
correspond to runtime code. The desktop/GUI stack (M35–M52) is 62 lines of
hardcoded serial print statements (`services/go/desktop.go`) — no framebuffer
is ever written. Hardware matrices, installer/recovery drills, fleet ops,
performance baselines, and security fuzz "evidence" are seeded JSON generated
by `tools/run_*.py`, not measurements of a running system.

## 2. The Gap, Subsystem By Subsystem

Reference points: Linux ≈ 460+ syscalls / tens of millions LOC; NT and XNU of
similar scale; SerenityOS/Haiku/Redox each have full TCP/IP, USB/HID input,
GUI; even xv6 (a ~10k LOC teaching kernel, Rugo's size class) has fork/exec, a
real on-disk FS with directories, and pipes.

### 2.1 Memory management — the deepest gap
Missing entirely (every mainstream kernel has all of these):
- **Physical frame allocator** (Linux buddy; NT PFN database). Rugo uses static
  compile-time page pools.
- **Kernel heap allocator** (slab/zone). Rugo is `no_std` with no dynamic
  allocation anywhere in the kernel.
- **Demand paging** — page faults are fatal (`trap.rs:49-75` kills the task).
- **Copy-on-write, swap, general `mmap`, huge pages, per-process address
  spaces in the default lane** (tasks share one user VMA window).
- **ASLR** — user code/stack live at fixed addresses (0x40_0000 / 0x80_0000).

Without a real allocator, almost every other gap below is unbuildable.

### 2.2 Process model and scheduling
- **No fork/exec/spawn-from-filesystem.** `sys_fork`/`sys_clone` return -1
  (`lib.rs:1037-1048`). Process population is fixed at build time (max 6 task
  slots in R4). Mainstream OSes — and xv6 — create processes dynamically.
- **No preemption in the product lane.** The default Go lane is cooperative
  (`sys_yield`); the PIT-driven scheduler exists only behind the `sched_test`
  feature flag. Linux (EEVDF), NT, and XNU are fully preemptive with priority
  and fairness models.
- **No SMP.** No AP startup, per-CPU data, IPIs, or kernel locking model. All
  mainstream kernels and most hobby OSes are multicore.
- **No signals, process groups, sessions, or job control.**

### 2.3 The kernel is not one kernel
M3 (cooperative threads, file rights) and R4 (multi-task IPC, sockets,
storage) are **mutually exclusive compile-time feature lanes** of one 5,690-line
`lib.rs`. There is no single kernel binary that has all demonstrated
capabilities at once. Mainstream kernels are a single image plus runtime-loaded
modules. Unifying the lanes is a prerequisite for everything else.

### 2.4 Filesystem
- The "filesystem" the shell sees is a **hardcoded FD routing table** mapping
  ~6 fixed path strings to reserved disk sectors (`lib.rs:1234-1245`,
  `sys_open_v1` at `lib.rs:539-649`). SimpleFS exists as an on-disk format for
  package payloads.
- Missing: **VFS layer, directories, arbitrary file creation, file
  permissions/ownership, mount points, page cache, journaling/crash
  consistency, any second filesystem** (ext4/NTFS/APFS analogs, FAT for
  interop). Linux 0.01 (1991) had a real Minix FS; xv6 has directories and a
  log.

### 2.5 Networking
- Kernel networking is an **ARP responder + UDP port-7 echo** (`net.rs:264-396`).
  Socket syscalls exist but terminate in test scaffolding, not a stack.
- Missing: **TCP** (the single biggest functional absence), DHCP, DNS, ICMP,
  IPv6, real routing, a socket layer wired to a real protocol stack, any
  firewall, TLS anywhere. SerenityOS, Redox, Haiku, and every BSD have full
  TCP/IP; lwIP-class stacks are the common hobby-OS shortcut.

### 2.6 Hardware support and drivers
Present: VirtIO-blk, VirtIO-net, NVMe, partial AHCI, serial console.
Missing relative to any desktop OS — and to SerenityOS/Haiku:
- **Keyboard/mouse input** (no PS/2 or USB HID — the shell is serial-only)
- **USB stack** (no XHCI), **GPU/framebuffer output** (no pixel is ever drawn),
  audio, ACPI power management (suspend/resume, shutdown is QEMU debug-exit),
  general PCIe enumeration/hotplug, common NICs (e1000, RTL), Wi-Fi/Bluetooth.
- No driver *model*: no module loading, device tree/registry, or driver ABI —
  drivers are hardcoded probes. (Linux: unified driver model + sysfs; NT: WDM/
  PnP manager; XNU: IOKit.)

### 2.7 Userspace
- **No libc / POSIX layer.** Go services make raw syscalls via `unsafe` stubs.
  No malloc, no stdio, no POSIX file/process APIs. Redox built relibc for
  exactly this reason; Serenity built its own libc.
- **No coreutils**: no ls, cat, cp, ps, mkdir — nothing. The shell cannot
  launch external programs; `run base-shell` prints `APP: base-shell ok`
  without spawning anything (`shell_session.go:256`).
- **Shell** has no pipes, redirection, globbing, scripting, or job control.
- **No dynamic linking** (static ELF only, no PIE, no relocations), so no
  shared libraries.
- **Stock-Go lane is a 117-line spike** exercising 7 syscalls. Full Go runtime
  behavior (GC under load, channels, netpoller) is unproven on this kernel.
- **No multi-user model**: no UIDs/GIDs, login, authentication, or sessions.

### 2.8 Security
Present: ring 0/3 split, per-task capability bits (storage/network), per-FD
rights (R/W/poll), syscall allowlist in one lane.
Missing: users/permissions, ASLR, enforced W^X/NX, real sandboxing
(seccomp/Landlock; NT tokens/ACLs; macOS entitlements/sandbox), audit logging,
verified/secure boot (markers only), disk encryption, kernel hardening
(stack canaries beyond Rust defaults, KASLR).

### 2.9 Graphics / desktop
Nothing real exists — no framebuffer writes, no compositor, no window manager,
no widget toolkit, no fonts. The entire M35–M52 desktop narrative is serial
markers plus seeded reports. For contrast: Haiku and SerenityOS ship complete
desktops; Redox has Orbital; even bare framebuffer terminals (the usual first
step) are absent here.

### 2.10 Platform and operations
- No real installer (policy docs + a 29-byte stub ISO; no boot test runs an
  install). No UEFI boot path in tests (BIOS/Limine only). No bare-metal
  evidence at all — every proof is QEMU q35/i440fx.
- No real crash dumps, time sync, system logging daemon, or update fetch over
  a network (package payloads are compiled in).
- Self-hosting (building Rugo on Rugo) is not remotely in scope yet — expected;
  even Redox treats this as a long-term goal.

## 3. Priority-Ordered Build List

Implementation status (kept current as items close; details and proof
paths in `docs/superpowers/plans/2026-06-10-full-os-gap-roadmap.md`):

Foundations (everything else is blocked on these):
1. Kernel **physical frame allocator + kernel heap**, then demand paging with a
   real page-fault handler.
   — **DONE** (`make test-mm-foundation-v1`, `docs/runtime/memory_v1.md`)
2. **Unify the M3/R4 feature lanes** into one kernel binary; lift the 6-task
   static limit with dynamically allocated process structures.
   — task-limit lift **DONE** (`make test-dynamic-tasks-v1`); lane
   unification proceeds incrementally — every new subsystem (mm, vfs, tcp)
   lands unconditionally or in the product lane, not behind new test lanes
3. **Preemptive timer scheduling in the default lane** (move the PIT/APIC path
   out of `sched_test`).
   — **DONE** (`make test-sched-preempt-v1`, `docs/runtime/scheduler_v1.md`)
4. **exec-from-filesystem**: load and run an ELF named by path, parent/child
   lifecycle (spawn+wait is fine; full fork optional — Redox/Fuchsia-style
   spawn is a defensible design).
   — **DONE** (`make test-exec-v1`, `docs/runtime/exec_v1.md`; `sys_spawn`
   id 46, shell `run base-shell` executes a hash-verified on-disk ELF)

Usability (turns the demo into an operable system):
5. **Real VFS + directories** over SimpleFS: create/stat/list arbitrary files;
   then file permissions.
   — **DONE** including permissions (`make test-vfs-v1`,
   `make test-users-v1`, `docs/runtime/vfs_v1.md`; per-node owner+mode,
   per-task uid, enforcement on open/unlink/chmod)
6. **TCP/IP stack** (port lwIP or smoltcp — smoltcp is Rust and fits the
   kernel) wired to the existing socket syscalls; DHCP + DNS client.
   — **DONE**: wire TCP client (`make test-tcp-v1`,
   `docs/runtime/tcp_v1.md`, in-repo stack, no external crates) plus
   DHCP and DNS clients (`make test-netcfg-v1`,
   `docs/runtime/netcfg_v1.md`; a real DISCOVER/OFFER against QEMU's
   DHCP server and a real A query answered by a test-owned resolver)
7. **Keyboard input + framebuffer text console**, so the OS is usable outside
   a serial pipe.
   — **DONE** (`make test-console-v1`, `docs/runtime/console_v1.md`; a full
   session typed via emulated PS/2, transcript rendered as framebuffer
   pixels and verified by screendump)
8. Shell that **executes external programs** with arguments, plus a first
   coreutils set (ls, cat, echo, ps); pipes need pipe IPC in the kernel.
   — **DONE** (`make test-coreutils-v1`, `make test-pipes-v1`; the
   utilities are real on-disk ELFs in `apps/coreutils/`, spawned with
   the command's argument string, and `cat file | wc` joins two of them
   through a kernel pipe with fd handoff; pipeline stages run
   sequentially until per-process address spaces land)
9. A **libc-equivalent** (POSIX-ish syscall layer) — prerequisite for porting
   any existing software, which is how every hobby OS bootstraps an ecosystem.
   — rlibc v1 **DONE** (`make test-libc-v1`, `docs/runtime/libc_v1.md`;
   `libc/` provides crt0 + open/read/write/close/mkdir/unlink/stat/
   pipe/spawn/wait/malloc/strings/printf, and `hello` — a real C
   program compiled with the host gcc — runs from the package store);
   errno/FILE*/lseek and a third-party port are the carry-forward

Parity (credible-OS tier):
10. SMP bring-up + kernel locking; signals; users/permissions; dynamic linking;
    USB/HID; ASLR + W^X; a real installer; then — and only then — an honest
    graphics stack (framebuffer → compositor → toolkit).
    — W^X on dynamic user memory **DONE** (`make test-wx-v1`; EFER.NXE +
    NX data pages, a stack-execution probe dies with the fetch-fault
    error code); signals **DONE** (`make test-signals-v1`,
    `docs/runtime/signals_v1.md`; `sys_signal_ctl` id 48 opens the
    additive v3.2 window — handler delivery via frame rewrite,
    sigreturn, default kill); users/permissions **DONE**
    (`make test-users-v1`; per-task uid — root services, uid-100 apps —
    gating file open/unlink/chmod by owner+mode); SMP bring-up
    groundwork **DONE** (`make test-smp-v1`, `docs/runtime/smp_v1.md`;
    every AP runs kernel code and parks, the default lane boots clean
    on multicore — per-CPU scheduling and kernel locking remain);
    dynamic linking, USB/HID, ASLR, installer, and graphics are pending

## 4. Honest Positioning

Rugo today is an **xv6-class teaching/demo kernel** (~12.5k LOC of real runtime
code) with unusually strong test, ABI, and release discipline — but it is below
xv6 on process model (no fork/exec, no real FS directories, no pipes) while
being ahead of it on service supervision, IPC design, NVMe, and validation
infrastructure. The distance to "fully functioning OS" in the
SerenityOS/Haiku/Redox sense is dominated by five absences: dynamic memory
management, dynamic process creation, a real filesystem, TCP/IP, and any form
of local input/output beyond the serial port. The 54-milestone ledger should be
read as ~8 runtime milestones plus ~46 qualification-scaffolding milestones;
the repo's own `SOURCE_MAP.md` says as much, and this analysis confirms it at
the source level.
