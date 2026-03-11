# Rugo

Rugo is a hybrid OS with a Rust `no_std` kernel and Go user space. The product
lane is the Rust kernel plus TinyGo-first services; the legacy C lane is kept
as a historical reference, and the stock-Go port remains experimental.

## What Matters First

| Lane | Status | Current source paths | Role |
|------|--------|----------------------|------|
| Hybrid OS | default | `arch/`, `boot/`, `kernel_rs/`, `services/go/` | Primary runtime: Rust kernel plus TinyGo-first user-space integration. |
| Experimental Go port | non-default | `services/go_std/`, `tools/build_go_std_spike.sh`, `tools/gostd_stock_builder/` | Stock-Go bring-up and ABI experiments. |
| Legacy baseline | reference only | `legacy/` | Historical C + gccgo implementation kept for comparison and regression context. |
| Support and validation | secondary | `tools/`, `tests/`, `docs/`, `.github/` | Build, packaging, CI, evidence collection, and acceptance gates. |

Architecture and repo strategy:
- overview: [docs/architecture/README.md](docs/architecture/README.md)
- target layout strategy: [docs/architecture/repo-strategy.md](docs/architecture/repo-strategy.md)
- current roadmap: [docs/roadmap/README.md](docs/roadmap/README.md)
- historical backlog/archive index: [docs/archive/README.md](docs/archive/README.md)

## Quick Start

```bash
make demo-go      # recommended: Rust kernel + Go bootstrap (goinit -> gosvcm -> gosh -> timesvc)
make run-kernel   # kernel-only boot path
make run          # compatibility alias for make run-kernel
make validate     # compatibility alias for make test-qemu
make image-go-std # experimental stock-Go port image
```

Detailed build and host prerequisites live in [docs/BUILD.md](docs/BUILD.md).

## Architecture

The current source tree is still transitional, but the architectural split is
simple:

- Core runtime: `arch/`, `boot/`, `kernel_rs/`
- Userspace runtime: `services/go/`
- Tooling and support: `tools/`, `tests/`, `.github/`, `vendor/`
- Legacy and archive: `legacy/`, historical execution backlogs in `docs/`
- Experimental and research: `services/go_std/`, stock-Go builder tooling, and
  extended milestone research docs

The next structural step is to move toward an explicit `kernel/`,
`userspace/`, `support/`, `validation/`, and `experimental/` layout without
breaking the current build or test paths. That migration plan is documented in
[docs/architecture/repo-strategy.md](docs/architecture/repo-strategy.md).

## Milestone Status

| Lane | Kernel milestones | Go milestones |
|------|-------------------|---------------|
| Legacy (`legacy/`) | M0-M7: done | G0: done |
| Rugo (default hybrid OS lane) | M0-M52: done | G1: done, G2: done |

Checkpoint strings retained for gate history: `M0-M40: done`, `M0-M41: done`,
`M0-M42: done`, `M0-M43: done`, `M0-M44: done`, `M0-M45: done`, `M0-M46: done`,
`M0-M47: done`, `M0-M48: done`, `M0-M49: done`, `M0-M50: done`, `M0-M51: done`,
`M0-M52: done`.

Latest completed GUI milestone: `M52`.
Latest completed hardware promotion phase: `M45-M47`.

Historical GUI checkpoint strings retained for gate history:
Latest completed GUI milestone: `M49`.
Latest completed GUI milestone: `M50`.
Latest completed GUI milestone: `M51`.
Latest completed GUI milestone: `M52`.

Completed architecture streams:
- compatibility and userspace: `M8`, `M16`, `M17`, `M25`, `M27`, `M36`, `M41`
- security, runtime, and release: `M10`, `M11`, `M14`, `M20`, `M21`, `M22`,
  `M24`, `M26`, `M28`, `M29`, `M30`, `M31`, `M32`, `M33`, `M34`
- hardware and platform breadth: `M9`, `M15`, `M18`, `M19`, `M23`, `M37`,
  `M38`, `M39`, `M43`, `M44`, `M45`, `M46`, `M47`
- desktop stack and workflows: `M35`, `M48`, `M49`, `M50`, `M51`, `M52`

For the exhaustive completion matrix, see [MILESTONES.md](MILESTONES.md). For
the detailed validation ledger that CI gates still reference, see
[docs/STATUS.md](docs/STATUS.md).

## Demo And Validation Paths

- Recommended demo path: `make demo-go`
  This is the clearest expression of the intended product direction: a Rust
  kernel booting a Go init task, a Go service manager, a Go shell, and a
  syscall-backed Go service.
- Kernel-only smoke path: `make run-kernel`
  Useful when working on boot, paging, traps, or scheduler mechanics.
- Full acceptance suite: `make test-qemu`
  Builds all current QEMU images, including TinyGo and stock-Go lanes.
- Stock-Go experiment: `make image-go-std`
  This remains an experimental porting lane, not the default repo story.

## Detailed Docs

- Build guide: [docs/BUILD.md](docs/BUILD.md)
- Architecture overview: [docs/architecture/README.md](docs/architecture/README.md)
- Repo migration strategy: [docs/architecture/repo-strategy.md](docs/architecture/repo-strategy.md)
- Current roadmap and milestone streams: [docs/roadmap/README.md](docs/roadmap/README.md)
- Historical archive index: [docs/archive/README.md](docs/archive/README.md)
- Legacy lane notes: [legacy/README.md](legacy/README.md)

## Milestone Closure Records

Historical milestone backlogs remain available, but they are archive material
rather than the primary product narrative.

- M40 execution backlog (completed): `docs/M40_EXECUTION_BACKLOG.md`
- M41 execution backlog (completed): `docs/M41_EXECUTION_BACKLOG.md`
- M42 execution backlog (completed): `docs/M42_EXECUTION_BACKLOG.md`
- M43 execution backlog (completed): `docs/M43_EXECUTION_BACKLOG.md`
- M44 execution backlog (completed): `docs/M44_EXECUTION_BACKLOG.md`
- M45 execution backlog (completed): `docs/M45_EXECUTION_BACKLOG.md`
- M46 execution backlog (completed): `docs/M46_EXECUTION_BACKLOG.md`
- M47 execution backlog (completed): `docs/M47_EXECUTION_BACKLOG.md`
- M48 execution backlog (completed): `docs/M48_EXECUTION_BACKLOG.md`
- M49 execution backlog (completed): `docs/M49_EXECUTION_BACKLOG.md`
- M50 execution backlog (completed): `docs/M50_EXECUTION_BACKLOG.md`
- M51 execution backlog (completed): `docs/M51_EXECUTION_BACKLOG.md`
- M52 execution backlog (completed): `docs/M52_EXECUTION_BACKLOG.md`

Earlier completed backlogs for `M8-M39` are indexed in
[docs/archive/README.md](docs/archive/README.md).
