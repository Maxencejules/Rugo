# M9 Execution Backlog (Hardware Enablement Matrix v1)

Date: 2026-03-04  
Lane: Rugo (Rust kernel + Go user space)

## Goal

Drive M9 from post-M8 baseline to an executable hardware enablement matrix:

- Define tiered hardware support contract.
- Add deterministic matrix smoke checks for storage/network profiles.
- Clean up PCI probe/init flow for multi-driver growth.
- Document DMA/IOMMU, ACPI/UEFI hardening, and bare-metal bring-up rules.

M9 source of truth remains `MILESTONES.md`, `docs/hw/*`, and this backlog.

## Current State Summary

- M8 compatibility profile v1 is complete and release-gated.
- Driver bring-up is functional but primarily QEMU `q35`-centric.
- PCI probe logic existed in duplicated per-driver code paths.
- No explicit M9 hardware matrix gate existed in CI.

## Execution Result

- PR-1: complete (2026-03-04)
- PR-2: complete (2026-03-04)
- PR-3: complete (2026-03-04)

## PR-1: Matrix Contract + Harness

### Objective

Define the v1 hardware matrix and make it executable in tests.

### Scope

- Add support-matrix contract: `docs/hw/support_matrix_v1.md`.
- Extend QEMU fixtures for Tier 0 (`q35`) and Tier 1 (`pc`/i440fx).
- Add `tests/hw/` matrix checks:
  - storage smoke on both tiers,
  - network smoke on both tiers,
  - deterministic missing-device probe paths.

### Primary files

- `docs/hw/support_matrix_v1.md`
- `tests/conftest.py`
- `tests/hw/test_hardware_matrix_v1.py`
- `tests/hw/test_probe_negative_paths_v1.py`

### Acceptance checks

- `python -m pytest tests/hw/test_hardware_matrix_v1.py -v`
- `python -m pytest tests/hw/test_probe_negative_paths_v1.py -v`

### Done criteria for PR-1

- Tiered matrix is documented and executable.
- Probe/init positive and negative paths are deterministic.

## PR-2: PCI Cleanup + DMA Safety Gate

### Objective

Standardize PCI probe/init flow and enforce DMA negative-path checks inside M9.

### Scope

- Refactor PCI access into shared helpers:
  - generic device find,
  - BAR0 I/O extraction,
  - bus-master enable helper.
- Add baseline PCI function claim path (resource arbitration guard).
- Add DMA safety gate tests under `tests/hw/`.

### Primary files

- `kernel_rs/src/lib.rs`
- `tests/hw/test_dma_safety_v1.py`

### Acceptance checks

- `python -m pytest tests/hw/test_dma_safety_v1.py -v`
- `python -m pytest tests/drivers/test_virtio_blk_badlen.py tests/drivers/test_virtio_blk_badptr.py -v`

### Done criteria for PR-2

- PCI probe/init logic is shared and no longer duplicated across blk/net paths.
- DMA invalid-input rejections are explicitly included in M9 hardware gate.

## PR-3: CI Gate + Hardening Docs + Milestone Closure

### Objective

Promote M9 matrix checks to CI and close milestone status docs.

### Scope

- Add local gate target: `make test-hw-matrix`.
- Add CI matrix gate and JUnit history artifact.
- Publish hardening/runbook docs:
  - `docs/hw/dma_iommu_strategy_v1.md`
  - `docs/hw/acpi_uefi_hardening_v1.md`
  - `docs/hw/bare_metal_bringup_v1.md`
- Mark M9 as done in milestone/status docs.

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-hw-matrix`
- CI step `Hardware matrix v1 gate` passes and uploads `hw-matrix-junit`.

### Done criteria for PR-3

- M9 gate is release-enforced in CI.
- M9 status is marked `done` with clear evidence pointers.

## X2 runtime-backed closure addendum (2026-03-18)

- This backlog is now governed by the shared x2 hardware runtime qualification lane.
- The historical matrix rows for q35 and i440fx now feed the shared x2
  hardware runtime qualification gate instead of standing alone as matrix-only
  evidence.
- `make test-x2-hardware-runtime-v1` promotes the transitional block and net
  target classes into the shared device registry and target qualification
  report.
- Any future M9-era target expansion must extend the shared X2 qualification
  lane rather than adding a detached matrix-only claim.

## Non-goals for M9 backlog

- Full non-virtio hardware family expansion (NVMe/AHCI/native NICs).
- Automated physical-lab CI coverage for Tier 2 bare-metal.
- Complete IOMMU implementation.
