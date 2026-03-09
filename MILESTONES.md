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
| G2 | Full Go port | n/a | ✅ | Rugo-only milestone. Done (`tests/go/test_std_go_binary.py`, stock-Go artifact contract path). |
| M8 | Compatibility Profile v1 | n/a | ✅ | Rugo: `tests/compat/*`, `tests/pkg/test_pkg_external_apps.py`; docs in `docs/abi/*` and `docs/M8_EXECUTION_BACKLOG.md`. |
| M9 | Hardware enablement matrix v1 | n/a | ✅ | Rugo: `tests/hw/*`, CI hardware gate, and docs in `docs/hw/*`, `docs/M9_EXECUTION_BACKLOG.md`. |
| M10 | Security baseline v1 | n/a | ✅ | Rugo: `tests/security/*`, security CI gate, and docs in `docs/security/*`, `docs/M10_EXECUTION_BACKLOG.md`. |
| M11 | Runtime + toolchain maturity v1 | n/a | ✅ | Rugo: `tests/runtime/*`, `make test-runtime-maturity`, CI runtime gate, docs in `docs/runtime/*`, `docs/M11_EXECUTION_BACKLOG.md`. |
| M12 | Network stack v1 | n/a | ✅ | Rugo: `tests/net/*`, `make test-network-stack-v1`, CI network gate, docs in `docs/net/*`, `docs/M12_EXECUTION_BACKLOG.md`. |
| M13 | Storage reliability v1 | n/a | ✅ | Rugo: `tests/storage/*`, `make test-storage-reliability-v1`, CI storage gate, docs in `docs/storage/*`, `docs/M13_EXECUTION_BACKLOG.md`. |
| M14 | Productization + release engineering v1 | n/a | done | Rugo: `tests/build/*`, update attack/metadata tests in `tests/pkg/*`, `make test-release-engineering-v1`, CI release-engineering gate, docs in `docs/build/*`, `docs/pkg/*`, `docs/M14_EXECUTION_BACKLOG.md`. |
| M15 | Hardware Enablement Matrix v2 | n/a | done | Rugo: `tests/hw/*_v2`, `make test-hw-matrix-v2`, CI `Hardware matrix v2 gate`, docs in `docs/hw/*_v2`, `docs/M15_EXECUTION_BACKLOG.md`. |
| M16 | Process + Scheduler Model v2 | n/a | done | Rugo: `tests/sched/*_v2`, `tests/user/*_v2`, `make test-process-scheduler-v2`, CI `Process scheduler v2 gate`, docs in `docs/abi/*_v2`, `docs/M16_EXECUTION_BACKLOG.md`. |
| M17 | Compatibility Profile v2 | n/a | done | Rugo: `tests/compat/*_v2` + tier gate tests, `make test-compat-v2`, CI `Compatibility profile v2 gate`, docs in `docs/abi/*_v2`, `docs/runtime/syscall_coverage_matrix_v2.md`, `docs/M17_EXECUTION_BACKLOG.md`. |
| M18 | Storage Reliability v2 | n/a | done | Rugo: `tests/storage/*_v2` + storage gate tests, `make test-storage-reliability-v2`, CI `Storage reliability v2 gate`, docs in `docs/storage/*_v2`, `docs/M18_EXECUTION_BACKLOG.md`. |
| M19 | Network Stack v2 | n/a | done | Rugo: `tests/net/*_v2` + network gate tests, `make test-network-stack-v2`, CI `Network stack v2 gate`, docs in `docs/net/*_v2`, `docs/M19_EXECUTION_BACKLOG.md`. |
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

Track B: Full Go port (G2 baseline complete)
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

## G2: Full Go port (Track B) — Rugo only

Milestone status: done (2026-03-04).

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

- `tests/go/test_std_go_binary.py` asserts `GOSTD: ok` marker flow plus
  stock-Go artifact contract metadata from `out/gostd-contract.env`.
- `tools/build_go_std_spike.sh` now uses stock Go (`go run ./tools/gostd_stock_builder/main.go`)
  to produce `out/gostd.bin` and `out/gostd-contract.env`.
- `make image-go-std` builds the G2 image path (`os-go-std.iso`) with kernel
  `go_std_test` embedding `out/gostd.bin`.
- Go-port contract and bridge map remains documented in
  `docs/abi/go_port_spike_v0.md` (`GOOS=rugo`, `GOARCH=amd64`).
- Execution history and PR sequencing are recorded in
  `docs/G2_EXECUTION_BACKLOG.md`.

---

## M8: Compatibility Profile v1

Milestone status: done (2026-03-04).

### PR-1 deliverables completed

- Versioned ABI contract: `docs/abi/syscall_v1.md`.
- Compatibility profile contract: `docs/abi/compat_profile_v1.md`.
- Initial compatibility suite scaffolding under `tests/compat/`.

### PR-2 deliverables completed

- Loader/process/fd contract documentation:
  - `docs/abi/process_thread_model_v1.md`
  - updated `docs/abi/syscall_v1.md`
- Kernel v1 compatibility primitives in `kernel_rs/src/lib.rs`:
  - ELF64 validation helper policy (`elf_v1_validate_image`)
  - core compatibility syscalls: open/read/write/close/wait/poll
  - deterministic M3 fd-table baseline (`/dev/console`, `/compat/hello.txt`)
- Compatibility tests promoted from TODO skeletons to executable checks:
  - `tests/compat/test_loader_contract.py`
  - `tests/compat/test_process_lifecycle.py`
  - `tests/compat/test_process_wait.py`
  - `tests/compat/test_fd_table.py`
  - updated `tests/compat/test_file_io_subset.py`
- Backlog execution status updated: `docs/M8_EXECUTION_BACKLOG.md`.

### PR-3 deliverables completed

- Remaining POSIX subset closure (time/signal/socket):
  - `tests/compat/test_time_signal_subset.py`
  - `tests/compat/test_socket_api_subset.py`
  - `tests/compat/test_posix_subset.py`
- Package/repository v1 bootstrap:
  - `docs/pkg/package_format_v1.md`
  - `tools/pkg_bootstrap_v1.py`
  - `tests/pkg/test_pkg_external_apps.py`
- Compatibility/profile gating promoted to CI:
  - `.github/workflows/ci.yml` explicit compatibility profile v1 gate step

---

## M9: Hardware Enablement Matrix v1

Milestone status: done (2026-03-04).

### Definition of done

- Published hardware matrix v1 with Tier 0/Tier 1 target classes.
- Deterministic storage + network smoke checks run on both matrix tiers.
- PCI probe/init path is shared across in-tree virtio drivers.
- DMA invalid-input rejection checks are included in matrix acceptance.
- ACPI/UEFI hardening policy and bare-metal bring-up runbook are published.

### Acceptance tests

| Test | Markers |
|------|---------|
| `tests/hw/test_hardware_matrix_v1.py` | `BLK: found virtio-blk`, `BLK: rw ok`, `NET: virtio-net ready`, `NET: udp echo` |
| `tests/hw/test_probe_negative_paths_v1.py` | `BLK: not found`, `NET: not found` |
| `tests/hw/test_dma_safety_v1.py` | `BLK: badlen ok`, `BLK: badptr ok` |

### Rugo evidence

- PCI helper cleanup + arbitration baseline in `kernel_rs/src/lib.rs`:
  - shared device lookup and BAR parsing helpers,
  - shared bus-master enable helper,
  - PCI function claim guard.
- Hardware matrix docs:
  - `docs/hw/support_matrix_v1.md`
  - `docs/hw/dma_iommu_strategy_v1.md`
  - `docs/hw/acpi_uefi_hardening_v1.md`
  - `docs/hw/bare_metal_bringup_v1.md`
- Execution and sequencing history:
  - `docs/M9_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` target `test-hw-matrix`
- `.github/workflows/ci.yml` step `Hardware matrix v1 gate`

---

## M10: Security Baseline v1

Milestone status: done (2026-03-04).

### Definition of done

- Per-handle rights model is documented and enforced in kernel paths.
- Restricted syscall/resource profile exists with deterministic deny behavior.
- Boot artifact manifest signing and verification rejects tampered artifacts.
- Continuous security fuzz gate runs and produces machine-readable reports.
- Incident response and advisory workflow is documented.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/security/test_rights_enforcement.py` | `SEC: rights ok` |
| `tests/security/test_syscall_filter.py` | `SEC: filter ok` |
| `tests/security/test_secure_boot_manifest_v1.py` | tamper rejection + key rotation pass |
| `tests/security/test_security_fuzz_harness_v1.py` | fuzz report schema, `total_violations == 0` |

### Rugo evidence

- Kernel enforcement in `kernel_rs/src/lib.rs`:
  - fd rights fields and checks in `sys_open_v1/sys_read_v1/sys_write_v1/sys_poll_v1`,
  - new security syscalls:
    - `sys_fd_rights_get_v1` (24),
    - `sys_fd_rights_reduce_v1` (25),
    - `sys_fd_rights_transfer_v1` (26),
    - `sys_sec_profile_set_v1` (27),
  - restricted profile allowlist in `m10_syscall_allowed`.
- Security user-mode acceptance payloads:
  - `services/security/sec_rights.asm`
  - `services/security/sec_filter.asm`
- Security tooling:
  - `tools/secure_boot_manifest_v1.py`
  - `tools/run_security_fuzz_v1.py`
- Security contracts/process docs:
  - `docs/security/rights_capability_model_v1.md`
  - `docs/security/syscall_filtering_v1.md`
  - `docs/security/secure_boot_policy_v1.md`
  - `docs/security/fuzzing_v1.md`
  - `docs/security/incident_response_v1.md`
- Execution and sequencing history:
  - `docs/M10_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` target `test-security-baseline`
  - `.github/workflows/ci.yml` step `Security baseline v1 gate`

---

## M11: Runtime + Toolchain Maturity v1

Milestone status: done (2026-03-04).

### Definition of done

- Runtime-port contract is documented for `GOOS=rugo` and `GOARCH=amd64`.
- Runtime syscall coverage matrix exists with explicit owner and target rows.
- Toolchain bootstrap and reproducibility artifacts are generated by scripts.
- Runtime maturity is release-gated in local and CI lanes.
- Maintainership and ABI-window policy are documented.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/runtime/test_runtime_stress_v1.py` | `GOSTD:*` runtime markers + stress/pressure baselines |
| `tests/runtime/test_runtime_abi_window_v1.py` | ABI policy + syscall docs alignment |
| `tests/runtime/test_runtime_contract_docs_v1.py` | runtime docs/tools/gate wiring present |

### Rugo evidence

- Runtime contract docs:
  - `docs/runtime/port_contract_v1.md`
  - `docs/runtime/syscall_coverage_matrix_v1.md`
  - `docs/runtime/abi_stability_policy_v1.md`
  - `docs/runtime/toolchain_bootstrap_v1.md`
  - `docs/runtime/maintainers_v1.md`
- Runtime tooling:
  - `tools/bootstrap_go_port_v1.sh`
  - `tools/runtime_toolchain_contract_v1.py`
- Runtime test gate:
  - `tests/runtime/test_runtime_contract_docs_v1.py`
  - `tests/runtime/test_runtime_abi_window_v1.py`
  - `tests/runtime/test_runtime_stress_v1.py`
- Execution and sequencing history:
  - `docs/M11_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` target `test-runtime-maturity`
  - `.github/workflows/ci.yml` step `Runtime + toolchain maturity v1 gate`

---

## M12: Network Stack v1

Milestone status: done (2026-03-04).

### Definition of done

- Network stack v1 contract docs are published and versioned.
- IPv4/UDP baseline behavior is deterministic and test-backed.
- TCP state-machine and retransmission policy baselines are documented and
  tested.
- IPv6 ND + ICMPv6 baseline is documented and covered by model checks.
- Interop and soak artifact lanes are release-gated locally and in CI.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/net/test_udp_echo.py` | `NET: virtio-net ready`, `NET: udp echo`, no timeout marker |
| `tests/net/test_ipv4_udp_contract_v1.py` | IPv4/UDP contract + tiered net fixture checks |
| `tests/net/test_tcp_state_machine_v1.py` | TCP lifecycle state-transition baseline |
| `tests/net/test_tcp_retransmission_v1.py` | deterministic retransmission/backoff/timeout policy |
| `tests/net/test_ipv6_nd_icmpv6_v1.py` | IPv6 ND + ICMPv6 baseline model checks |
| `tests/net/test_net_interop_matrix_v1.py` | interop report schema + target threshold |
| `tests/net/test_net_soak_v1.py` | soak/fault-injection report schema + failure threshold |

### Rugo evidence

- Network contract docs:
  - `docs/net/network_stack_contract_v1.md`
  - `docs/net/socket_contract_v1.md`
  - `docs/net/ipv4_udp_profile_v1.md`
  - `docs/net/tcp_state_machine_v1.md`
  - `docs/net/retransmission_timer_policy_v1.md`
  - `docs/net/ipv6_baseline_v1.md`
- Network tooling:
  - `tools/net_trace_capture_v1.py`
  - `tools/run_net_interop_matrix_v1.py`
  - `tools/run_net_soak_v1.py`
- Network test gate:
  - `tests/net/test_socket_contract_docs_v1.py`
  - `tests/net/test_socket_poll_semantics_v1.py`
  - `tests/net/test_net_trace_capture_v1.py`
- Execution and sequencing history:
  - `docs/M12_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` target `test-network-stack-v1`
  - `.github/workflows/ci.yml` step `Network stack v1 gate`

---

## M13: Storage Reliability v1

Milestone status: done (2026-03-04).

### Definition of done

- Storage v1 contracts are documented for crash model and durability classes.
- Write-ordering/barrier semantics are explicitly defined and test-backed.
- Recovery and fault-campaign tooling produce machine-readable reports.
- Storage reliability is release-gated in local and CI lanes.
- M13 evidence is linked in backlog/status docs.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/storage/test_storage_contract_docs_v1.py` | storage docs/tools/gate wiring present |
| `tests/storage/test_fsync_semantics_v1.py` | `fdatasync` vs `fsync` durability semantics |
| `tests/storage/test_write_ordering_contract_v1.py` | ordered commit baseline and invalid ordering rejection |
| `tests/storage/test_storage_recovery_v1.py` | recovery report schema + mountability checks |
| `tests/storage/test_storage_fault_campaign_v1.py` | campaign report schema + threshold checks |
| `tests/storage/test_storage_integrity_checker_v1.py` | checksum/corruption detection baseline |
| `tests/storage/test_storage_reliability_gate_v1.py` | make/ci/status wiring for M13 gate |

### Rugo evidence

- Storage contract docs:
  - `docs/storage/fs_v1.md`
  - `docs/storage/durability_model_v1.md`
  - `docs/storage/write_ordering_policy_v1.md`
  - `docs/storage/recovery_playbook_v1.md`
  - `docs/storage/fault_campaign_v1.md`
- Storage tooling:
  - `tools/storage_recover_v1.py`
  - `tools/run_storage_fault_campaign_v1.py`
- Storage test gate:
  - `tests/storage/test_storage_recovery_v1.py`
  - `tests/storage/test_storage_fault_campaign_v1.py`
  - `tests/storage/test_storage_reliability_gate_v1.py`
- Execution and sequencing history:
  - `docs/M13_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` target `test-storage-reliability-v1`
  - `.github/workflows/ci.yml` step `Storage reliability v1 gate`

---

## M14: Productization + Release Engineering v1

Milestone status: done (2026-03-04).

### Definition of done

- Release/channel/versioning policy is documented and executable.
- Signed update metadata and client verification enforce rollback/replay/freeze
  protection.
- Supply-chain artifacts (SBOM + provenance) are generated in local/CI gates.
- Release engineering is release-gated in local and CI lanes.
- Installer/recovery/support-bundle baseline is documented and test-backed.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/build/test_release_contract_docs_v1.py` | release policy/versioning/checklist docs and tool presence |
| `tests/build/test_release_contract_report_v1.py` | release contract report schema + channel metadata |
| `tests/pkg/test_update_metadata_v1.py` | signed update metadata + verify roundtrip |
| `tests/pkg/test_update_rollback_protection_v1.py` | replay/rollback rejection by monotonic sequence |
| `tests/pkg/test_update_attack_suite_v1.py` | attack-suite report schema + zero failures target |
| `tests/build/test_release_engineering_gate_v1.py` | make/ci/docs wiring + supply-chain/support artifacts |

### Rugo evidence

- Release and productization docs:
  - `docs/build/release_policy_v1.md`
  - `docs/build/versioning_scheme_v1.md`
  - `docs/build/release_checklist_v1.md`
  - `docs/build/supply_chain_policy_v1.md`
  - `docs/build/installer_recovery_baseline_v1.md`
  - `docs/pkg/update_protocol_v1.md`
  - `docs/pkg/update_repo_layout_v1.md`
  - `docs/security/update_signing_policy_v1.md`
- Tooling:
  - `tools/release_contract_v1.py`
  - `tools/update_repo_sign_v1.py`
  - `tools/update_client_verify_v1.py`
  - `tools/run_update_attack_suite_v1.py`
  - `tools/generate_sbom_v1.py`
  - `tools/generate_provenance_v1.py`
  - `tools/collect_support_bundle_v1.py`
- Release test gate:
  - `tests/build/test_release_engineering_gate_v1.py`
  - `tests/pkg/test_update_metadata_v1.py`
  - `tests/pkg/test_update_rollback_protection_v1.py`
  - `tests/pkg/test_update_attack_suite_v1.py`
- Execution and sequencing history:
  - `docs/M14_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` target `test-release-engineering-v1`
  - `.github/workflows/ci.yml` step `Release engineering v1 gate`

---

## M15: Hardware Enablement Matrix v2

Milestone status: done (2026-03-06).

### Definition of done

- Tiered hardware matrix v2 criteria are versioned and executable.
- Tier 0/Tier 1 checks are release-gated in local and CI lanes.
- Bare-metal promotion policy is evidence-bound and deterministic.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/hw/test_hardware_matrix_v2.py` | Tier 0/Tier 1 storage/network marker parity |
| `tests/hw/test_probe_negative_paths_v2.py` | deterministic probe-missing baseline |
| `tests/hw/test_dma_iommu_policy_v2.py` | DMA invalid input rejection baseline |
| `tests/hw/test_acpi_boot_paths_v2.py` | deterministic firmware boot/halt markers |
| `tests/hw/test_bare_metal_smoke_v2.py` | runbook evidence bundle + smoke markers |
| `tests/hw/test_hw_gate_v2.py` | make/ci/backlog gate wiring |

### Rugo evidence

- Contract docs:
  - `docs/hw/support_matrix_v2.md`
  - `docs/hw/device_profile_contract_v2.md`
  - `docs/hw/dma_iommu_strategy_v2.md`
  - `docs/hw/acpi_uefi_hardening_v2.md`
  - `docs/hw/bare_metal_bringup_v2.md`
- Test gate:
  - `tests/hw/test_hardware_matrix_v2.py`
  - `tests/hw/test_probe_negative_paths_v2.py`
  - `tests/hw/test_dma_iommu_policy_v2.py`
  - `tests/hw/test_acpi_boot_paths_v2.py`
  - `tests/hw/test_bare_metal_smoke_v2.py`
  - `tests/hw/test_hw_gate_v2.py`
- Execution history:
  - `docs/M15_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` target `test-hw-matrix-v2`
  - `.github/workflows/ci.yml` step `Hardware matrix v2 gate`

---

## M16: Process + Scheduler Model v2

Milestone status: done (2026-03-06).

### Definition of done

- Process/thread lifecycle semantics are versioned in v2 ABI docs.
- Preemption and fairness regressions are deterministic and gate-blocking.
- Faulted user tasks are contained without scheduler collapse.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/user/test_process_wait_kill_v2.py` | wait/kill lifecycle contract + fault/exit markers |
| `tests/user/test_signal_delivery_v2.py` | deterministic signal delivery and queue bounds |
| `tests/sched/test_preempt_timer_quantum_v2.py` | timer quantum preemption determinism |
| `tests/sched/test_priority_fairness_v2.py` | equal-priority and weighted fairness checks |
| `tests/sched/test_scheduler_soak_v2.py` | seeded soak determinism + machine-readable failures |
| `tests/sched/test_scheduler_gate_v2.py` | make/ci/docs gate wiring and closure |

### Rugo evidence

- Contract docs:
  - `docs/abi/process_thread_model_v2.md`
  - `docs/abi/scheduling_policy_v2.md`
- Scheduler/process model:
  - `tests/sched/v2_model.py`
- Test gate:
  - `tests/user/test_process_wait_kill_v2.py`
  - `tests/user/test_signal_delivery_v2.py`
  - `tests/sched/test_preempt_timer_quantum_v2.py`
  - `tests/sched/test_priority_fairness_v2.py`
  - `tests/sched/test_scheduler_soak_v2.py`
  - `tests/sched/test_scheduler_gate_v2.py`
- Execution history:
  - `docs/M16_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` target `test-process-scheduler-v2`
  - `.github/workflows/ci.yml` step `Process scheduler v2 gate`

---

## M17: Compatibility Profile v2

Milestone status: done (2026-03-06).

### Definition of done

- ABI/loader v2 contracts are versioned and executable-check referenced.
- Supported vs unsupported compatibility surfaces are explicit.
- External app tier thresholds are deterministic and release-gated.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/compat/test_abi_profile_v2_docs.py` | v2 ABI/profile/loader doc contracts + kernel dispatch wiring |
| `tests/compat/test_elf_loader_dynamic_v2.py` | deterministic static/dynamic loader acceptance and rejection rules |
| `tests/compat/test_posix_profile_v2.py` | required/optional/unsupported profile surface model checks |
| `tests/compat/test_external_apps_tier_v2.py` | signed artifact tier thresholds + Tier B runtime marker checks |
| `tests/compat/test_compat_gate_v2.py` | make/ci/docs gate wiring and closure |

### Rugo evidence

- Contract docs:
  - `docs/abi/syscall_v2.md`
  - `docs/abi/compat_profile_v2.md`
  - `docs/abi/elf_loader_contract_v2.md`
  - `docs/runtime/syscall_coverage_matrix_v2.md`
- Compatibility model:
  - `tests/compat/v2_model.py`
- Test gate:
  - `tests/compat/test_abi_profile_v2_docs.py`
  - `tests/compat/test_elf_loader_dynamic_v2.py`
  - `tests/compat/test_posix_profile_v2.py`
  - `tests/compat/test_external_apps_tier_v2.py`
  - `tests/compat/test_compat_gate_v2.py`
- Execution history:
  - `docs/M17_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` target `test-compat-v2`
  - `.github/workflows/ci.yml` step `Compatibility profile v2 gate`

---

## M18: Storage Reliability v2

Milestone status: done (2026-03-06).

### Definition of done

- Storage v2 contract docs freeze journaling, durability, and recovery semantics.
- Recovery and power-fail artifacts are deterministic and machine-readable.
- Storage v2 gate is release-blocking in local and CI lanes.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/storage/test_journal_recovery_v2.py` | recovery report schema + mountability and journal state invariants |
| `tests/storage/test_metadata_integrity_v2.py` | metadata corruption detection, bounds checks, and doc contract tokens |
| `tests/storage/test_powerfail_campaign_v2.py` | deterministic power-fail campaign thresholds and report schema |
| `tests/storage/test_storage_gate_v2.py` | make/ci/docs gate wiring and closure |

### Rugo evidence

- Contract docs:
  - `docs/storage/fs_v2.md`
  - `docs/storage/durability_model_v2.md`
  - `docs/storage/write_ordering_policy_v2.md`
  - `docs/storage/recovery_playbook_v2.md`
  - `docs/storage/fault_campaign_v2.md`
- Tooling:
  - `tools/storage_recover_v2.py`
  - `tools/run_storage_powerfail_campaign_v2.py`
- Test gate:
  - `tests/storage/test_journal_recovery_v2.py`
  - `tests/storage/test_metadata_integrity_v2.py`
  - `tests/storage/test_powerfail_campaign_v2.py`
  - `tests/storage/test_storage_gate_v2.py`
- Execution history:
  - `docs/M18_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` target `test-storage-reliability-v2`
  - `.github/workflows/ci.yml` step `Storage reliability v2 gate`

---

## M19: Network Stack v2

Milestone status: done (2026-03-08).

### Definition of done

- Network v2 contract docs freeze TCP/IPv6/DNS baseline behavior.
- Interop and soak evidence is deterministic and machine-readable.
- Network v2 gate is release-blocking in local and CI lanes.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/net/test_tcp_interop_v2.py` | deterministic TCP peer interop profile checks |
| `tests/net/test_ipv6_interop_v2.py` | deterministic IPv6 ND/ICMPv6 + dual-stack fallback checks |
| `tests/net/test_dns_stub_v2.py` | DNS-stub TTL/NXDOMAIN semantics + v2 interop/soak schema checks |
| `tests/net/test_network_gate_v2.py` | make/ci/docs gate wiring and closure |

### Rugo evidence

- Contract docs:
  - `docs/net/network_stack_contract_v2.md`
  - `docs/net/socket_contract_v2.md`
  - `docs/net/tcp_profile_v2.md`
  - `docs/net/interop_matrix_v2.md`
- Tooling:
  - `tools/run_net_interop_matrix_v2.py`
  - `tools/run_net_soak_v2.py`
- Test gate:
  - `tests/net/test_tcp_interop_v2.py`
  - `tests/net/test_ipv6_interop_v2.py`
  - `tests/net/test_dns_stub_v2.py`
  - `tests/net/test_network_gate_v2.py`
- Execution history:
  - `docs/M19_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` target `test-network-stack-v2`
  - `.github/workflows/ci.yml` step `Network stack v2 gate`

---

## References

- Rust target `x86_64-unknown-none` platform support notes (no std; allocator
  required for alloc).
- Go porting policy (ports are OS/arch; avoid broken/incomplete ports).
- TinyGo documentation (targets; language and stdlib support notes).
- Build toolchain details: `docs/build/rust_toolchain.md`
- Legacy Go toolchain: `docs/build/go_toolchain.md`

