# Rugo Milestones

Date: 2026-02-17
Target: x86-64, QEMU-first, Limine boot protocol

---

## How to read this document

This repository contains two implementation lanes for the same OS design:

| Lane | Language | Location | Purpose |
|------|----------|----------|---------|
| **Legacy** | C + Go (gccgo) | `legacy/` | Completed reference baseline (M0-M7 + G0). Preserved as a working fallback and architectural reference. |
| **Rugo** | Rust (no_std) + Go (TinyGo, planned) | repo root | Rewrite target. The default build. Milestones are re-implemented in Rust with the same acceptance criteria. |

**Legacy is not Rugo.** Legacy is a reference implementation that proved out the
milestone design. Rugo is the production rewrite. Both share the same milestone
definitions and acceptance tests, but track completion independently.

Milestones M0-M7 define kernel functionality (boot, memory, scheduling, user
mode, IPC, drivers, filesystem, networking). Go milestones define user-space
service integration: G0 is legacy-only (gccgo kernel entry), G1 and G2 are
Rugo-only (TinyGo services and full Go port).

---

## Repo lanes and how to run

### Rugo (default)

```bash
# Native (requires nightly Rust, nasm, ld, xorriso, qemu, pytest)
make build        # Compile Rust kernel → out/kernel.elf
make image        # Build bootable ISO → out/os.iso
make image-panic  # Build panic-test ISO → out/os-panic.iso
make test-qemu    # Build all Rugo test images, run pytest tests/

# Docker (cross-platform, no host toolchain needed)
make docker-all
```

Tests: `tests/` (boot, trap, sched, user, ipc, drivers, fs, pkg, net, go, stress, quota, pressure)

### Legacy

```bash
# From repo root (requires nasm, gcc, gccgo, ld, objcopy, xorriso, qemu, pytest)
make -C legacy build
make -C legacy image
make -C legacy test-qemu   # 16 tests
```

Tests: `legacy/tests/` (boot, trap, sched, user, ipc, drivers, fs, pkg, net)

**Note:** Both lanes output to `out/`. Building one lane overwrites the other's
`kernel.elf` and `os.iso`.

---

## Milestone status matrix

| # | Milestone | Legacy | Rugo | Evidence |
|---|-----------|--------|------|----------|
| M0 | Boot + serial | ✅ | ✅ | Legacy: `legacy/tests/boot/test_boot_banner.py` (KERNEL markers). Rugo: `tests/boot/test_boot_banner.py` + `tests/boot/test_panic_path.py` (RUGO markers). |
| M1 | Paging + traps | ✅ | ✅ | Legacy: `legacy/tests/boot/test_paging_enabled.py`, `legacy/tests/trap/test_page_fault_report.py`. Rugo: `tests/boot/test_paging_enabled.py`, `tests/trap/test_page_fault_report.py`, `tests/trap/test_idt_smoke.py`. |
| M2 | Scheduler + threads | ✅ | ✅ | Legacy: `legacy/tests/sched/test_timer_ticks.py`, `legacy/tests/sched/test_two_threads.py`. Rugo: `tests/sched/test_timer_ticks.py`, `tests/sched/test_two_threads.py`. |
| M3 | User mode + syscalls | ✅ | ✅ | Legacy: `legacy/tests/user/test_enter_user_mode.py`, `test_syscall_roundtrip.py`, `test_user_fault.py`. Rugo: `tests/user/test_enter_user_mode.py`, `tests/user/test_syscall_roundtrip.py`, `tests/user/test_thread_exit.py`, `tests/user/test_user_fault.py`. |
| M4 | IPC + shared memory | ✅ | ✅ | Legacy: `legacy/tests/ipc/test_ping_pong.py`, `legacy/tests/ipc/test_shm_bulk.py`. Rugo: `tests/ipc/test_ping_pong.py`, `tests/ipc/test_shm_bulk.py`. |
| M5 | VirtIO block + syscalls | ✅ | ✅ | Legacy: `legacy/tests/drivers/test_virtio_blk_identify.py`, `test_virtio_blk_rw.py`. Rugo: `tests/drivers/test_virtio_blk_identify.py`, `tests/drivers/test_virtio_blk_rw.py`. |
| M6 | Filesystem + pkg + shell | ✅ | ✅ | Legacy: `legacy/tests/fs/test_fsd_smoke.py`, `legacy/tests/pkg/test_pkg_install_run.py`. Rugo: `tests/fs/test_fsd_smoke.py`, `tests/pkg/test_pkg_install_run.py`. |
| M7 | VirtIO net + UDP echo | ✅ | ✅ | Legacy: `legacy/tests/net/test_udp_echo.py`. Rugo: `tests/net/test_udp_echo.py` (`NET: udp echo`). |
| G0 | Go kernel entry (gccgo) | ✅ | n/a | Legacy: `legacy/tests/boot/test_go_entry.py`. Legacy-only milestone. |
| G1 | Go services (TinyGo) | n/a | ✅ | Rugo: `tests/go/test_go_user_service.py` (`GOUSR: ok`). TinyGo bare-metal x86_64. |
| G2 | Full Go port | n/a | ◐ | Rugo-only milestone. In progress (prep). Long-term. |

Legend: ✅ = done with passing tests, ◐ = in progress (prep), — = not started, n/a = not applicable to this lane.

---

## Intent and architecture boundary

- Rust owns kernel mechanisms (correctness-critical, no_std).
- Go owns high-level user space services and the ecosystem.
- Go Track A (TinyGo-first) is the short-term path to get Go services running.
- Go Track B (full Go port) is explicitly long-term and should start only after
  the kernel ABI/process model stabilizes.

### Kernel (Rust) provides mechanisms

- Memory management: physical allocator, paging, per-process page tables
- Traps: IDT, exception handling, syscall entry dispatch
- Scheduling: threads, runnable queues, timers
- IPC and shared memory primitives
- Security boundary: handle table or capabilities
- Minimal drivers required to boot and test under QEMU (VirtIO preferred early)

### User space (Go services) provides policy and ecosystem

- Service manager: svcman
- Filesystem service: fsd
- Package manager: pkg and pkgd
- Network stack service: netd
- Shell and core utilities

---

## Key constraints and design choices

### Rust kernel target and no_std stance

- Kernel code is Rust `no_std` compiled for `x86_64-unknown-none`.
- A thin assembly layer exists for CPU entry (`arch/x86_64/entry.asm`).
- Build uses `build-std = ["core"]` with `compiler-builtins-mem`.

### Go in user space, not kernel

- Go is used for high-level services in user space (fsd, pkgd, netd, svcman).
- Running standard Go binaries on a new OS is a porting effort (a new
  GOOS/GOARCH combination). This is not the short-term path.

### Go track decision (explicit)

Track A: TinyGo-first (short-term)
- Goal: get 1-3 Go services running early with a constrained subset of Go/stdlib.
- Expectation: avoid heavy reflection patterns; keep deps minimal.

Track B: Full Go port (long-term)
- Goal: run standard Go binaries produced by the stock toolchain.
- Start condition: syscall ABI and process model have stopped changing.

---

## M0: Boot + serial logging

### Definition of done

- Limine boots the kernel in QEMU.
- Serial output prints deterministic markers and exits cleanly via debug-exit.
- Panic path prints a deterministic marker and exits.

### Acceptance tests (marker-based)

| Test | Markers | ISO |
|------|---------|-----|
| `test_boot_banner` | `RUGO: boot ok`, `RUGO: halt ok` | Normal (`out/os.iso`) |
| `test_panic_path` | `RUGO: panic code=...` | Panic (`out/os-panic.iso`, built with `panic_test` feature) |

### Legacy evidence

- `legacy/tests/boot/test_boot_banner.py` asserts `KERNEL: boot ok`, `KERNEL: halt ok`
- `legacy/kernel/main.c`, `legacy/arch/x86_64/entry.asm`

### Rugo evidence

- `tests/boot/test_boot_banner.py` asserts `RUGO: boot ok`, `RUGO: halt ok`
- `tests/boot/test_panic_path.py` asserts `RUGO: panic code=`
- `kernel_rs/src/lib.rs` (kmain + panic handler)
- `arch/x86_64/entry.asm` (sets stack, calls kmain)

---

## M1: Paging + traps + page fault reporting

### Definition of done

- Paging enabled with known mapping strategy.
- IDT installed.
- Page fault handler logs fault address and error bits.

### Acceptance tests

| Test | Markers |
|------|---------|
| `test_paging_enabled` | `MM: paging=on` |
| `test_page_fault_report` | `PF: addr=0x... err=0x...` |
| `test_idt_smoke` | Deterministic trap marker for a forced exception |

### Legacy evidence

- `legacy/tests/boot/test_paging_enabled.py`, `legacy/tests/trap/test_page_fault_report.py`
- `legacy/kernel/vmm.c`, `legacy/kernel/pmm.c`, `legacy/arch/x86_64/idt.c`, `legacy/arch/x86_64/trap.c`

### Rugo evidence

- `tests/boot/test_paging_enabled.py`, `tests/trap/test_page_fault_report.py`, `tests/trap/test_idt_smoke.py`
- `kernel_rs/src/lib.rs` (paging enable marker, IDT install, fault/trap handlers)
- `arch/x86_64/isr.asm` (ISR stubs)

---

## M2: Scheduler + kernel threads

### Definition of done

- Timer interrupts tick.
- At least two kernel threads can run and context switch.
- A minimal lock primitive exists and is safe with interrupts.

### Acceptance tests

| Test | Markers |
|------|---------|
| `test_timer_ticks` | `TICK: 100` |
| `test_two_threads` | Interleaved `A` and `B` |

### Legacy evidence

- `legacy/tests/sched/test_timer_ticks.py`, `legacy/tests/sched/test_two_threads.py`
- `legacy/kernel/sched.c`, `legacy/arch/x86_64/pit.c`, `legacy/arch/x86_64/context.asm`

### Rugo evidence

- `tests/sched/test_timer_ticks.py`, `tests/sched/test_two_threads.py`
- `kernel_rs/src/lib.rs` (timer tick path and scheduler test flow)
- `arch/x86_64/context.asm` (context switch assembly)

---

## M3: User mode + syscall ABI v0

### Definition of done

- Kernel can enter ring 3.
- Syscalls round-trip safely.
- User faults are contained and do not crash the kernel.

### Syscall ABI v0 (minimum)

- debug: `sys_debug_write`
- tasking: `sys_thread_spawn`, `sys_thread_exit`, `sys_yield`
- memory: `sys_vm_map`, `sys_vm_unmap`
- shm: `sys_shm_create`, `sys_shm_map`
- ipc: `sys_ipc_send`, `sys_ipc_recv`
- time: `sys_time_now`

### Acceptance tests

| Test | Markers |
|------|---------|
| `test_enter_user_mode` | `USER: hello` |
| `test_syscall_roundtrip` | `SYSCALL: ok` |
| `test_thread_exit` | `THREAD_EXIT: ok` and clean halt |
| `test_user_fault` | `USER: killed` and kernel continues |

### Legacy evidence

- `legacy/tests/user/test_enter_user_mode.py`, `test_syscall_roundtrip.py`, `test_user_fault.py`
- `legacy/kernel/syscall.c`, `legacy/kernel/process.c`, `legacy/user/init.c`, `legacy/user/fault.c`

### Rugo evidence

- `tests/user/test_enter_user_mode.py`, `tests/user/test_syscall_roundtrip.py`, `tests/user/test_thread_exit.py`, `tests/user/test_user_fault.py`
- `docs/abi/syscall_v0.md` (ABI contract)
- `kernel_rs/src/lib.rs` (ring 3 entry, syscall dispatch, user-fault containment)

---

## M4: IPC + shared memory + service registry v0

### Definition of done

- Two user processes can exchange messages through kernel IPC endpoints.
- Shared memory can be created and mapped by two processes.
- A service registry exists for name-to-endpoint resolution.
- Service registration rejects inactive or out-of-range endpoint IDs.

### Acceptance tests

| Test | Markers |
|------|---------|
| `test_ping_pong` | `PING: ok`, `PONG: ok` |
| `test_shm_bulk` | `SHM: checksum ok` |

### Legacy evidence

- `legacy/tests/ipc/test_ping_pong.py`, `legacy/tests/ipc/test_shm_bulk.py`
- `legacy/kernel/ipc.c`, `legacy/kernel/shm.c`, `legacy/kernel/service_registry.c`
- `legacy/user/ping.c`, `legacy/user/pong.c`, `legacy/user/shm_writer.c`, `legacy/user/shm_reader.c`

### Rugo evidence

- `tests/ipc/test_ping_pong.py`, `tests/ipc/test_shm_bulk.py`
- `tests/ipc/test_svc_overwrite.py`, `tests/ipc/test_svc_full.py`, `tests/ipc/test_svc_bad_endpoint.py`
- `kernel_rs/src/lib.rs` (IPC, SHM, and service-registry syscall paths)

---

## M5: VirtIO block + block syscalls

### Definition of done

- VirtIO block works in QEMU.
- Kernel exposes block read/write syscalls.
- Read/write passes deterministic data integrity tests.

### Acceptance tests

| Test | Markers |
|------|---------|
| `test_virtio_blk_identify` | `BLK: found virtio-blk` |
| `test_virtio_blk_rw` | `BLK: rw ok` |
| `test_virtio_blk_init_invariants` | `BLK: invariants ok` |
| `test_virtio_blk_init_failure` | `BLK: init failed` |

### Legacy evidence

- `legacy/tests/drivers/test_virtio_blk_identify.py`, `legacy/tests/drivers/test_virtio_blk_rw.py`
- `legacy/kernel/virtio_blk.c`, `legacy/kernel/pci.c`

### Rugo evidence

- `tests/drivers/test_virtio_blk_identify.py`, `tests/drivers/test_virtio_blk_rw.py`
- `tests/drivers/test_virtio_blk_init_invariants.py`, `tests/drivers/test_virtio_blk_init_failure.py`
- `kernel_rs/src/lib.rs` (VirtIO block init + `sys_blk_read`/`sys_blk_write`)
- `docs/abi/syscall_v0.md` (`sys_blk_read` and `sys_blk_write`)

---

## M6: Filesystem + package manager + shell

### Definition of done

- A filesystem service exists that can open, read, write, list.
- A package tool can install a package and run an app.
- A shell can run a packaged hello app.

### Acceptance tests

| Test | Markers |
|------|---------|
| `test_fsd_smoke` | `FSD: mount ok` |
| `test_pkg_install_run` | `APP: hello world` |

### Legacy evidence

- `legacy/tests/fs/test_fsd_smoke.py`, `legacy/tests/pkg/test_pkg_install_run.py`
- `legacy/user/fsd.c`, `legacy/user/pkg.c`, `legacy/user/sh.c`, `legacy/user/hello.c`

### Rugo evidence

- `tests/fs/test_fsd_smoke.py` asserts `FSD: mount ok`
- `tests/pkg/test_pkg_install_run.py` asserts `APP: hello world`
- `kernel_rs/src/lib.rs` (fs_test feature: SimpleFS v0 mount + PKG v0 parse + hello user-mode)
- `tools/mkfs.py` (deterministic disk image builder)
- `docs/storage/fs_v0.md` (on-disk format documentation)

---

## M7: VirtIO net + net syscalls + UDP echo (optional)

### Definition of done

- VirtIO net works in QEMU.
- Kernel exposes send/recv syscalls.
- A UDP echo demo passes.

### Acceptance tests

| Test | Markers |
|------|---------|
| `test_udp_echo` | `NET: udp echo` |

### Legacy evidence

- `legacy/tests/net/test_udp_echo.py`
- `legacy/kernel/virtio_net.c`, `legacy/user/netd.c`

### Rugo evidence

- `tests/net/test_udp_echo.py` asserts `NET: udp echo`
- `kernel_rs/src/lib.rs` (net_test feature: VirtIO net driver + ARP/UDP echo)
- `docs/net/udp_echo_v0.md` (protocol documentation)
- `docs/abi/syscall_v0.md` (sys_net_send/sys_net_recv at syscalls 15–16)

---

## G0: Go kernel entry (gccgo) — Legacy only

This milestone exists only in the legacy lane. It demonstrated Go code running
in kernel context via gccgo. The Rugo lane does not use gccgo; Go services will
run in user space (see G1, G2).

### Acceptance tests

| Test | Markers |
|------|---------|
| `test_go_entry` | `GO: kmain ok` |

### Legacy evidence

- `legacy/tests/boot/test_go_entry.py`
- `legacy/kernelgo/entry.go`, `legacy/rtshim/bridge.c`, `legacy/rtshim/runtime_stubs.c`

---

## G1: Go services bringup (Track A: TinyGo-first) — Rugo only

### Definition of done

- At least one user space service is written in Go and runs on the Rugo kernel.
- The service uses the existing syscall ABI and IPC primitives.
- A test asserts `GOUSR: ok`.

### Acceptance tests

| Test | Markers |
|------|---------|
| `test_go_user_service` | `GOUSR: ok` |

### Rugo evidence

- `tests/go/test_go_user_service.py` asserts `GOUSR: ok`
- `services/go/main.go` (TinyGo user program, prints "GOUSR: ok" via sys_debug_write)
- `services/go/start.asm` (NASM: entry point, syscall wrappers, runtime stubs, bump allocator)
- `services/go/linker.ld` (user-space linker script, code at VA 0x400000)
- `tools/build_go.sh` (TinyGo build pipeline: NASM + TinyGo + LLD + objcopy)
- `kernel_rs/src/lib.rs` (go_test feature: embeds gousr.bin, enters ring 3)
- `Dockerfile` (updated with Go 1.25 + TinyGo 0.40.1 for Docker builds)

---

## G2: Full Go port (Track B: long-term) — Rugo only

This milestone is intentionally long-term. Treat it as optional until the OS
stops churning.

### Start conditions (gate)

- Syscall ABI has been stable across multiple releases.
- Process/thread model is stable and documented.
- Filesystem and networking contracts are stable enough that the Go runtime can
  rely on them.

### Definition of done

- Standard Go toolchain can produce binaries for your GOOS/GOARCH.
- Go runtime syscalls and thread management work on the OS.
- A standard Go hello binary runs and prints `GOSTD: ok`.

### Acceptance tests

| Test | Markers |
|------|---------|
| `test_std_go_binary` | `GOSTD: ok` |

### Rugo evidence

- Prep in progress: `tests/go/test_std_go_binary.py` plus `go_std_test`
  spike lane target (`GOSTD: ok` marker).
- Contract freeze active (2026-03-03 to 2026-03-17 or 2 releases):
  `docs/abi/syscall_v0.md` locks syscall semantics and marks FS/NET syscall
  contracts no-breaking-change during freeze.
- Process/thread model documented for freeze: `docs/abi/process_thread_model_v0.md`.
- Go-port spike contract and bridge map: `docs/abi/go_port_spike_v0.md`
  (`GOOS=rugo`, `GOARCH=amd64` contract).
- Minimal spike binary target wired: `tools/build_go_std_spike.sh`,
  `make image-go-std`, and kernel `go_std_test` feature (embeds `out/gostd.bin`).
- Execution backlog for G2 closure: `docs/G2_EXECUTION_BACKLOG.md`
  (three-PR sequence from kernel primitives to stock-Go path).
- Full runtime/toolchain porting remains blocked on M3+ ABI/process-model stability.

---

## References

- Rust target `x86_64-unknown-none` platform support notes (no std; allocator
  required for alloc).
- Go porting policy (ports are OS/arch; avoid broken/incomplete ports).
- TinyGo documentation (targets; language and stdlib support notes).
- Build toolchain details: `docs/build/rust_toolchain.md`
- Legacy Go toolchain: `docs/build/go_toolchain.md`
