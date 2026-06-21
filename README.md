# Rugo

Rugo is a hybrid OS with a Rust `no_std` kernel and Go user space.

Default product lane:
- Rust kernel in `arch/`, `boot/`, and `kernel_rs/src/`
- TinyGo-first Go services in `services/go/`

Non-default lanes:
- supported stock-Go userspace lane in `services/go_std/`
- historical C + gccgo baseline in `legacy/`

## Start Here

Runtime source map: [docs/architecture/SOURCE_MAP.md](docs/architecture/SOURCE_MAP.md)

Top-level wayfinding:
- [kernel/README.md](kernel/README.md)
- [userspace/README.md](userspace/README.md)
- [validation/README.md](validation/README.md)
- [support/README.md](support/README.md)
- [experimental/README.md](experimental/README.md)

Architecture and roadmap:
- overview: [docs/architecture/README.md](docs/architecture/README.md)
- repo strategy: [docs/architecture/repo-strategy.md](docs/architecture/repo-strategy.md)
- v1 product definition: [docs/RUGO_V1_PRODUCT.md](docs/RUGO_V1_PRODUCT.md)
- default shell-lane Alpha definition: [docs/build/default_alpha_base_os_v1.md](docs/build/default_alpha_base_os_v1.md)
- roadmap summary: [docs/roadmap/README.md](docs/roadmap/README.md)
- milestone framework: [docs/roadmap/MILESTONE_FRAMEWORK.md](docs/roadmap/MILESTONE_FRAMEWORK.md)

## What Is Live

The front door of this repo is the live Rust-kernel plus Go-userspace lane.

Visible proof paths:
- `make image-demo` then `make boot-demo`
  Boots `goinit -> gosvcm -> timesvc -> diagsvc/pkgsvc -> shell` on the default TinyGo lane with the declared q35 + virtio disk/net shell profile.
  Proof: `tests/go/test_go_user_service.py`
- `make image-kernel` then `make boot-kernel`
  Boots the kernel-only lane for boot, paging, trap, and scheduler work.
  Proof: `tests/boot/test_boot_banner.py`, `tests/boot/test_paging_enabled.py`, `tests/trap/test_idt_smoke.py`
- `make image-std` then `make boot-std`
  Boots the supported stock-Go userspace lane.
  Proof: `tests/go/test_std_go_binary.py`
- `make smoke-std`
  Verifies the supported stock-Go lane without pytest.
  Proof: `tools/smoke_boot.sh`, `Makefile` `smoke-std`
- `make test-runtime-maturity`
  Exercises the runtime-facing QEMU lane for stock-Go markers, stress syscall,
  memory pressure, thread spawn, and VM map behavior.
  Proof: `tests/runtime/test_runtime_stress_v1.py`
- `make test-mm-foundation-v1`
  Boots both the kernel-only and default Go images and verifies the dynamic
  memory foundation: Limine-memmap frame allocator, kernel heap self-test,
  and the demand-paged user heap window touched live from Go init.
  Proof: `tests/mm/test_mm_foundation_v1.py`, contract `docs/runtime/memory_v1.md`
- `make test-sched-preempt-v1`
  Boots the default Go image and verifies PIT-driven preemption of user
  tasks in the product lane, including the preemption-safe init protocol.
  Proof: `tests/sched/test_preempt_default_lane_v1.py`, contract
  `docs/runtime/scheduler_v1.md`
- `make test-dynamic-tasks-v1`
  Boots the default Go image and verifies the task population is no longer
  fixed at build time: 9 concurrent tasks at boot on the heap-backed task
  table with demand-paged, guard-zoned stacks.
  Proof: `tests/runtime/test_dynamic_tasks_v1.py`
- `make test-exec-v1`
  Boots the default Go image and verifies the shell executes a real
  external program: `run base-shell` loads a SHA-256-verified ELF from the
  package store on disk via `sys_spawn` (id 46), runs it as a child task,
  and reaps it.
  Proof: `tests/runtime/test_exec_from_fs_v1.py`, contract
  `docs/runtime/exec_v1.md`
- `make test-vfs-v1`
  Boots the default Go image and verifies a writable on-disk file tree
  with directories under `/data`: create, write, read back, list, unlink,
  and persistence across a reboot on the same disk.
  Proof: `tests/runtime/test_vfs_runtime_v1.py`, contract
  `docs/runtime/vfs_v1.md`
- `make test-tcp-v1`
  Boots the default Go image and proves real wire TCP: the guest's kernel
  TCP machine handshakes through QEMU's user-mode network with a
  host-side listener owned by the test and round-trips a payload.
  Proof: `tests/runtime/test_tcp_runtime_v1.py`, contract
  `docs/runtime/tcp_v1.md`
- `make test-netcfg-v1`
  Boots the default Go image and proves the DHCP and DNS clients: a
  real DISCOVER/OFFER exchange with QEMU's built-in DHCP server, and a
  real A query answered by a resolver the test runs on the host side
  of the user-mode network.
  Proof: `tests/runtime/test_netcfg_runtime_v1.py`, contract
  `docs/runtime/netcfg_v1.md`
- `make test-console-v1`
  Boots the default Go image and proves the local console: a full
  health/shutdown session typed through the emulated PS/2 keyboard via
  QMP send-key, with the boot transcript rendered to the framebuffer and
  verified as pixels by a QMP screendump.
  Proof: `tests/runtime/test_console_runtime_v1.py`, contract
  `docs/runtime/console_v1.md`
- `make test-coreutils-v1`
  Boots the default Go image and proves a first coreutils set: the
  shell's ls/cat/echo/ps commands spawn real on-disk ELF programs with
  the command's argument string; every output line comes from the
  spawned program itself.
  Proof: `tests/runtime/test_coreutils_runtime_v1.py`, contract
  `docs/runtime/exec_v1.md`
- `make test-pipes-v1`
  Boots the default Go image and proves pipe IPC: `cat file | wc` joins
  two real external programs through a 512-byte kernel pipe with fd
  handoff at spawn; the byte count only comes out right if the bytes
  crossed the ring.
  Proof: `tests/runtime/test_pipes_runtime_v1.py`, contract
  `docs/runtime/exec_v1.md`
- `make test-libc-v1`
  Boots the default Go image and proves the libc-equivalent: `hello` is
  a real C program compiled with the host gcc against rlibc (`libc/`),
  running from the package store - printf, malloc, and open/read on the
  `/data` tree all go through the POSIX-ish layer.
  Proof: `tests/runtime/test_libc_runtime_v1.py`, contract
  `docs/runtime/libc_v1.md`
- `make test-wx-v1`
  Boots the default Go image and proves W^X on dynamic user memory: the
  nxprobe app copies a `ret` onto its demand-paged stack and calls it -
  the fetch faults (NX) and the kernel kills the probe while the system
  shuts down cleanly.
  Proof: `tests/runtime/test_wx_runtime_v1.py`, contract
  `docs/runtime/memory_v1.md`
- `make test-signals-v1`
  Boots the default Go image and proves signals: the sigprobe app
  registers a handler and signals itself - the handler runs with the
  signal number, sigreturn resumes the interrupted path, and a second
  run without a handler is killed by the default action.
  Proof: `tests/runtime/test_signals_runtime_v1.py`, contract
  `docs/runtime/signals_v1.md`
- `make test-users-v1`
  Boots the default Go image and proves users/permissions: an
  unprivileged (uid 100) program is denied write and unlink on a
  root-owned file but may read it; after a root `fschmod` it may do all
  three.
  Proof: `tests/runtime/test_users_runtime_v1.py`, contract
  `docs/runtime/vfs_v1.md`
- `make test-smp-v1`
  Boots with -smp 4 and proves SMP groundwork: every application
  processor runs kernel code (atomic check-in) and parks, and the
  default Go lane boots and shuts down cleanly on multicore.
  Proof: `tests/runtime/test_smp_runtime_v1.py`, contract
  `docs/runtime/smp_v1.md`
- `make test-smp-syscall-v1`
  Boots the default Go image with multiple cores and proves an
  application processor servicing *real* syscalls for a migrated ring-3
  task through its own per-CPU `current` (`gs:[16]`), plus AP-side task
  exit and reschedule on a live second core - beyond the check-in-and-park
  groundwork of `test-smp-v1`.
  Proof: `tests/runtime/test_smp_syscall_v1.py`, contract
  `docs/runtime/smp_v1.md`
- `make test-pidns-v1`
  Boots the default Go image and proves PID + UTS namespaces via
  `sys_nsctl` (id 57): an unshared task sees only its own namespace,
  reads a namespace-local pid starting at 1 (becomes its namespace's
  `init`), and carries a per-namespace hostname.
  Proof: `tests/runtime/test_pidns_v1.py`, contract
  `docs/runtime/pidns_v1.md`
- `make test-tls-v1`
  Boots the default Go image and proves per-task thread-local storage via
  `sys_vm_ctl` op 5 (`set_tls`): each task gets its own `%fs` base,
  restored on every context switch, so `%fs:offset` reaches a per-task TLS
  block across yields.
  Proof: `tests/runtime/test_tls_v1.py`, contract
  `docs/runtime/tls_v1.md`
- `make test-dynlink-v1` (plus `make test-dlopen-ondisk-v1`)
  Boots the default Go image and proves real ELF dynamic loading via
  `sys_dlctl` (id 60): `dlopen` maps a shared object (embedded or read
  off `/data`) at a randomized code-base-ASLR load base, applies
  RELATIVE / GLOB_DAT / JUMP_SLOT relocations, and `dlsym` resolves
  symbols that run in ring 3; a concurrent handle table (`dlsym_h` /
  `dlclose`, with handles reclaimed on task exit) keeps multiple objects
  live at once.
  Proof: `tests/runtime/test_dynlink_v1.py`,
  `tests/runtime/test_dlhandles_v1.py`,
  `tests/runtime/test_dlopen_ondisk_v1.py`, contract
  `docs/runtime/dynlink_v1.md`
- `make test-winsrv-v1`
  Boots the default Go image and proves a standing window server with a
  persistent, owner-stamped surface registry and per-client lifecycle
  (`sys_ioctl` ops 8-10): multiple clients' windows coexist and a dead
  client's surfaces are reclaimed on task exit.
  Proof: `tests/runtime/test_winsrv_v1.py`, contract
  `docs/desktop/window_manager_contract_v1.md`
- `make test-pqsig-v1`
  Boots the default Go image and proves public-key package signing: a
  real asymmetric Lamport one-time signature (the kernel embeds only the
  256-pair SHA-256 public key) replaces the old symmetric HMAC the kernel
  could itself forge.
  Proof: `tests/runtime/test_pqsig_v1.py`
- epoll, distinct errno, and per-address-space `brk` across clone are
  proved by additional ring-3 lanes (run directly with pytest):
  `sys_epoll` (id 55) level-triggered readiness over fds/pipes with
  instances reclaimed on task exit
  (`tests/runtime/test_epoll_v1.py`); `sys_errno` (id 62) per-task error
  codes - well-defined paths stamp ENOENT/EBADF/... instead of collapsing
  every `-1` to EIO (`tests/runtime/test_errno_v1.py`); and POSIX `brk`
  kept consistent across pml4-sharing clone threads by
  copy-at-clone + propagate-on-write
  (`tests/runtime/test_clonebrk_v1.py`). ABI surface in
  `docs/abi/syscall_v3.md`.
- `make test-perf-regression-v1`
  Boots `out/os-go.iso`, captures boot-backed runtime metrics, and enforces
  performance regression budgets on the shipped default image.
  Proof: `tests/runtime/test_booted_runtime_capture_v1.py`,
  `tests/runtime/test_perf_gate_v1.py`
- `make test-observability-v2`
  Captures structured runtime logs, trace bundles, diagnostic snapshots, and
  panic-linked crash artifacts from the booted default image flow.
  Proof: `tests/runtime/test_observability_gate_v2.py`,
  `tests/runtime/test_crash_dump_gate_v1.py`
- `make test-evidence-integrity-v1`
  Audits runtime-backed performance, diagnostics, and crash evidence for
  default-image provenance and boot-instance linkage.
  Proof: `tests/runtime/test_evidence_integrity_gate_v1.py`,
  `tests/runtime/test_synthetic_evidence_ban_v1.py`
- `make test-security-hardening-v3`, `make test-conformance-v1`,
  `make test-fleet-ops-v1`, `make test-maturity-qual-v1`
  Run the default-lane hardening, profile qualification, runtime-lab rollout,
  and bounded LTS gates against boot-backed runtime evidence.
  Proof: `tests/security/test_security_hardening_gate_v3.py`,
  `tests/runtime/test_conformance_gate_v1.py`,
  `tests/runtime/test_fleet_ops_gate_v1.py`,
  `tests/build/test_maturity_gate_v1.py`
- `make test-native-driver-contract-v1`
  Freezes the native-driver lifecycle, DMA, firmware, and diagnostics contract
  for post-M52 hardware expansion.
  Proof: `tests/hw/test_native_driver_contract_gate_v1.py`
- `make test-native-driver-diagnostics-v1`
  Emits the machine-readable M53 diagnostics bundle for bind, IRQ or DMA, and
  firmware allow or deny paths.
  Proof: `tests/hw/test_native_driver_diag_gate_v1.py`
- `make test-x2-hardware-runtime-v1`
  Aggregates the historical X2 hardware backlog into one runtime-backed device
  registry, firmware/SMP, and target-qualification bundle.
  Proof: `tests/hw/test_x2_hardware_gate_v1.py`,
  `tests/hw/test_x2_hardware_runtime_v1.py`
- `make test-x3-platform-runtime-v1`
  Aggregates the historical X3 package, storage-platform, and catalog backlog
  into one boot-backed `pkgsvc` qualification bundle with signed metadata,
  replay update flow, and persistent runtime-media evidence.
  Proof: `tests/pkg/test_x3_platform_runtime_gate_v1.py`,
  `tests/pkg/test_x3_platform_runtime_service_v1.py`
- `make test-desktop-profile-runtime-v1`
  Aggregates the historical X4 desktop backlog into one boot-backed
  desktop-profile qualification bundle on `out/os-go-desktop.iso`.
  Proof: `tests/desktop/test_desktop_profile_runtime_gate_v1.py`,
  `tests/desktop/test_desktop_profile_runtime_v1.py`
- `make test-product-alpha-v1`
  Boots the alpha-candidate native desktop image on q35 plus NVMe and binds
  desktop, package/update, installer/recovery, and diagnostics evidence to one
  product-level report.
  Proof: `tests/build/test_product_alpha_gate_v1.py`,
  `tests/build/test_product_alpha_qualification_v1.py`
- `make test-hw-matrix-v7`
  Emits the machine-readable M54 matrix bundle for q35 NVMe and i440fx AHCI
  coverage on top of the v6 baseline.
  Proof: `tests/hw/test_hw_gate_v7.py`
- `make test-native-storage-v1`
  Freezes the M54 native storage contract for identify, queue, reset, and
  flush semantics.
  Proof: `tests/hw/test_native_storage_gate_v1.py`
- external package bootstrap and run path
  Proof: `tests/pkg/test_pkg_external_apps.py`

## Read The Repo Correctly

Implementation tree:
- kernel mechanisms: `arch/`, `boot/`, `kernel_rs/src/`
- default Go userspace: `services/go/`
- supported non-default stock-Go userspace: `services/go_std/`

Support tree:
- qualification and build tooling: `tools/`
- QEMU, contract, and gate tests: `tests/`
- contracts, policies, and backlog history: `docs/`

Important interpretation rule:
- many later `tools/run_*` programs and `tests/*gate*` suites produce
  deterministic qualification reports
- those reports are useful release and repo-discipline tooling
- they are not the same thing as additional runtime source under
  `kernel_rs/src/` or `services/`

Build output note:
- `kernel_rs/target/` and `out/` are build output, not architecture

## Quick Start

```bash
make help         # show the primary developer workflows
make kernel       # build the Rust kernel ELF
make userspace    # build the default TinyGo userspace payload
make image-demo   # build the default demo ISO
make boot-demo    # boot the default demo ISO as the persistent q35 shell target
make smoke-demo   # drive the shell's health + shutdown path and verify markers
make test-product-alpha-v1 # qualify the alpha candidate native desktop image
make image-std    # build the supported stock-Go ISO
make boot-std     # boot the supported stock-Go ISO in QEMU
make smoke-std    # boot + verify stock-Go serial markers without Python
make gate-all     # full pytest-backed acceptance suite
```

Detailed build and host prerequisites live in [docs/BUILD.md](docs/BUILD.md).

## Scoreboard

| Track | What counts as progress | Current phase | Historical mapping |
|------|--------------------------|---------------|--------------------|
| Core Hybrid OS | The default Rust-kernel plus Go-service lane boots, runs native services, persists data, performs network I/O, and enforces runtime isolation on declared baseline targets. | `C3` done; `C4` done; `C5` done. | `M0-M7`, `G1`, `M10`, `M12`, `M13`, `M16`, `M18`, `M19`, `M22`, `M25`, `M42` |
| Tooling / Validation / Release Infrastructure | Confidence, reproducibility, qualification, release, and fleet discipline around the core lane improve. | `T4` complete; next infrastructure phase is `T5 Advanced Trust and Compliance Infrastructure`. | `G2`, `M11`, `M14`, `M20`, `M21`, `M24`, `M28`, `M29`, `M30-M34`, `M40` |
| Expansion / Research / Platform Breadth | Compatibility, hardware breadth, desktop breadth, packaging breadth, and other product-surface expansion increase. | `X4` complete; next breadth phase is `X5 Next-Wave Breadth Research`. | `M8`, `M9`, `M15`, `M17`, `M23`, `M26`, `M27`, `M35-M39`, `M41`, `M43-M54` |

Primary scoring rule:
- the first row is the answer to "how close is the repo to the stated product?"
- current core closure order is `M10/M16 -> M25 -> M12/M13 -> boot-backed artifacts -> M18/M19 -> M22/M42 runtime-backed closure`
- `G1` is the default Go-service lane
- `G2` is the supported stock-Go lane, not the default repo state
- the 2026-06 daily-driver general-purpose parity audit then closed its 10
  "missing pieces vs a real daily-driver OS" findings into live ring-3 lanes
  (PID/UTS namespaces, per-task TLS, real `dlopen`/`dlsym` dynamic linking,
  `epoll`, distinct `errno`, a standing window server, AP-side syscall/exit,
  CoW `fork`, and Lamport public-key package signing); see
  [docs/analysis/daily-driver-gap-remediation.md](docs/analysis/daily-driver-gap-remediation.md)

## Architecture And Archive

- build guide: [docs/BUILD.md](docs/BUILD.md)
- runtime source map: [docs/architecture/SOURCE_MAP.md](docs/architecture/SOURCE_MAP.md)
- architecture overview: [docs/architecture/README.md](docs/architecture/README.md)
- exhaustive milestone ledger: [MILESTONES.md](MILESTONES.md)
- detailed validation ledger: [docs/STATUS.md](docs/STATUS.md)
- daily-driver general-purpose parity audit + what landed: [docs/analysis/daily-driver-gap-remediation.md](docs/analysis/daily-driver-gap-remediation.md)

Historical milestone backlogs are archived in [docs/archive/README.md](docs/archive/README.md).
Execution backlog index: [docs/archive/EXECUTION_BACKLOGS.md](docs/archive/EXECUTION_BACKLOGS.md)
