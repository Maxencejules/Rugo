# Rugo

Rugo is a hybrid OS with a Rust `no_std` kernel and Go user space.

Default product lane:
- Rust kernel in `arch/`, `boot/`, and `kernel_rs/src/`
- TinyGo-first Go services in `services/go/`

Non-default lanes:
- experimental stock-Go bring-up in `services/go_std/`
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
- roadmap summary: [docs/roadmap/README.md](docs/roadmap/README.md)
- milestone framework: [docs/roadmap/MILESTONE_FRAMEWORK.md](docs/roadmap/MILESTONE_FRAMEWORK.md)

## What Is Live

The front door of this repo is the live Rust-kernel plus Go-userspace lane.

Visible proof paths:
- `make image-demo` then `make boot-demo`
  Boots `goinit -> gosvcm -> gosh -> timesvc` on the default TinyGo lane.
  Proof: `tests/go/test_go_user_service.py`
- `make image-kernel` then `make boot-kernel`
  Boots the kernel-only lane for boot, paging, trap, and scheduler work.
  Proof: `tests/boot/test_boot_banner.py`, `tests/boot/test_paging_enabled.py`, `tests/trap/test_idt_smoke.py`
- `make image-go-std`
  Boots the experimental stock-Go lane.
  Proof: `tests/go/test_std_go_binary.py`
- `make test-runtime-maturity`
  Exercises the runtime-facing QEMU lane for stock-Go markers, stress syscall,
  memory pressure, thread spawn, and VM map behavior.
  Proof: `tests/runtime/test_runtime_stress_v1.py`
- `make test-native-driver-contract-v1`
  Freezes the native-driver lifecycle, DMA, firmware, and diagnostics contract
  for post-M52 hardware expansion.
  Proof: `tests/hw/test_native_driver_contract_gate_v1.py`
- `make test-native-driver-diagnostics-v1`
  Emits the machine-readable M53 diagnostics bundle for bind, IRQ or DMA, and
  firmware allow or deny paths.
  Proof: `tests/hw/test_native_driver_diag_gate_v1.py`
- external package bootstrap and run path
  Proof: `tests/pkg/test_pkg_external_apps.py`

## Read The Repo Correctly

Implementation tree:
- kernel mechanisms: `arch/`, `boot/`, `kernel_rs/src/`
- default Go userspace: `services/go/`
- experimental stock-Go userspace: `services/go_std/`

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
make boot-demo    # boot the default demo ISO in QEMU
make smoke-demo   # boot + verify demo serial markers without Python
make gate-all     # full pytest-backed acceptance suite
make image-go-std # experimental stock-Go port image
```

Detailed build and host prerequisites live in [docs/BUILD.md](docs/BUILD.md).

## Scoreboard

| Track | What counts as progress | Current phase | Historical mapping |
|------|--------------------------|---------------|--------------------|
| Core Hybrid OS | The default Rust-kernel plus Go-service lane boots, runs native services, persists data, performs network I/O, and enforces runtime isolation on declared baseline targets. | `C5` complete; next core phase is `C6 Runtime Quality Under Load`. | `M0-M7`, `G1`, `M10`, `M12`, `M13`, `M16`, `M18`, `M19`, `M22`, `M25`, `M42` |
| Tooling / Validation / Release Infrastructure | Confidence, reproducibility, qualification, release, and fleet discipline around the core lane improve. | `T4` complete; next infrastructure phase is `T5 Advanced Trust and Compliance Infrastructure`. | `G2`, `M11`, `M14`, `M20`, `M21`, `M24`, `M28`, `M29`, `M30-M34`, `M40` |
| Expansion / Research / Platform Breadth | Compatibility, hardware breadth, desktop breadth, packaging breadth, and other product-surface expansion increase. | `X4` complete; next breadth phase is `X5 Next-Wave Breadth Research`. | `M8`, `M9`, `M15`, `M17`, `M23`, `M26`, `M27`, `M35-M39`, `M41`, `M43-M53` |

Primary scoring rule:
- the first row is the answer to "how close is the repo to the stated product?"
- `G1` is the default Go-service lane
- `G2` is experimental stock-Go qualification, not the default repo state

## Architecture And Archive

- build guide: [docs/BUILD.md](docs/BUILD.md)
- runtime source map: [docs/architecture/SOURCE_MAP.md](docs/architecture/SOURCE_MAP.md)
- architecture overview: [docs/architecture/README.md](docs/architecture/README.md)
- exhaustive milestone ledger: [MILESTONES.md](MILESTONES.md)
- detailed validation ledger: [docs/STATUS.md](docs/STATUS.md)

Historical milestone backlogs are archived in [docs/archive/README.md](docs/archive/README.md).
Execution backlog index: [docs/archive/EXECUTION_BACKLOGS.md](docs/archive/EXECUTION_BACKLOGS.md)
