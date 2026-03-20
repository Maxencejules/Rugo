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
- roadmap summary: [docs/roadmap/README.md](docs/roadmap/README.md)
- milestone framework: [docs/roadmap/MILESTONE_FRAMEWORK.md](docs/roadmap/MILESTONE_FRAMEWORK.md)

## What Is Live

The front door of this repo is the live Rust-kernel plus Go-userspace lane.

Visible proof paths:
- `make image-demo` then `make boot-demo`
  Boots `goinit -> gosvcm -> timesvc -> diagsvc/pkgsvc -> shell` on the default TinyGo lane.
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
make boot-demo    # boot the default demo ISO in QEMU
make smoke-demo   # boot + verify demo serial markers without Python
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

## Architecture And Archive

- build guide: [docs/BUILD.md](docs/BUILD.md)
- runtime source map: [docs/architecture/SOURCE_MAP.md](docs/architecture/SOURCE_MAP.md)
- architecture overview: [docs/architecture/README.md](docs/architecture/README.md)
- exhaustive milestone ledger: [MILESTONES.md](MILESTONES.md)
- detailed validation ledger: [docs/STATUS.md](docs/STATUS.md)

Historical milestone backlogs are archived in [docs/archive/README.md](docs/archive/README.md).
Execution backlog index: [docs/archive/EXECUTION_BACKLOGS.md](docs/archive/EXECUTION_BACKLOGS.md)
