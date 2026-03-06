# M15 Execution Backlog (Hardware Enablement Matrix v2)

Date: 2026-03-06  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Move from QEMU-first confidence to a tiered hardware confidence model with
repeatable evidence on representative real-hardware classes.

M15 source of truth remains `docs/M15_M20_MULTIPURPOSE_PLAN.md`,
`MILESTONES.md`, and this backlog.

## Current State Summary

- M14 release-engineering baseline is complete and gate-wired.
- Existing hardware matrix v1 and v3 roadmap direction are documented.
- M15 must close the v2 transition with explicit tier criteria and CI gating.

## Execution Result

- PR-1: planned
- PR-2: planned
- PR-3: planned

## PR-1: Matrix v2 Contract + Target Classes

### Objective

Freeze hardware matrix v2 claims and acceptance semantics before expanding
driver behavior and bare-metal flow.

### Scope

- Add docs:
  - `docs/hw/support_matrix_v2.md`
  - `docs/hw/device_profile_contract_v2.md`
- Add tests:
  - `tests/hw/test_hardware_matrix_v2.py`
  - `tests/hw/test_probe_negative_paths_v2.py`
- Define Tier 0/Tier 1/Tier 2 pass criteria with artifact schema.

### Primary files

- `docs/hw/support_matrix_v2.md`
- `docs/hw/device_profile_contract_v2.md`
- `tests/hw/test_hardware_matrix_v2.py`
- `tests/hw/test_probe_negative_paths_v2.py`

### Acceptance checks

- `python -m pytest tests/hw/test_hardware_matrix_v2.py tests/hw/test_probe_negative_paths_v2.py -v`

### Done criteria for PR-1

- Matrix tiers are versioned and test-referenced.
- Hardware claims are bounded to matrix evidence only.

## PR-2: Driver Lifecycle + DMA/IOMMU Hardening v2

### Objective

Make driver init/runtime/error behavior deterministic across matrix targets.

### Scope

- Add docs:
  - `docs/hw/dma_iommu_strategy_v2.md`
  - `docs/hw/acpi_uefi_hardening_v2.md`
- Add tests:
  - `tests/hw/test_dma_iommu_policy_v2.py`
  - `tests/hw/test_acpi_boot_paths_v2.py`

### Primary files

- `docs/hw/dma_iommu_strategy_v2.md`
- `docs/hw/acpi_uefi_hardening_v2.md`
- `tests/hw/test_dma_iommu_policy_v2.py`
- `tests/hw/test_acpi_boot_paths_v2.py`

### Acceptance checks

- `python -m pytest tests/hw/test_dma_iommu_policy_v2.py tests/hw/test_acpi_boot_paths_v2.py -v`

### Done criteria for PR-2

- Probe/init/runtime failure semantics are deterministic and documented.
- DMA policy is explicit and validated by executable checks.

## PR-3: Bare-Metal Lane + Release Gate

### Objective

Promote M15 checks to required local and CI gates.

### Scope

- Add docs:
  - `docs/hw/bare_metal_bringup_v2.md`
- Add tests:
  - `tests/hw/test_bare_metal_smoke_v2.py`
  - `tests/hw/test_hw_gate_v2.py`
- Add local gate:
  - `Makefile` target `test-hw-matrix-v2`
- Add CI gate:
  - `.github/workflows/ci.yml` step `Hardware matrix v2 gate`

### Primary files

- `docs/hw/bare_metal_bringup_v2.md`
- `tests/hw/test_bare_metal_smoke_v2.py`
- `tests/hw/test_hw_gate_v2.py`
- `Makefile`
- `.github/workflows/ci.yml`

### Acceptance checks

- `make test-hw-matrix-v2`

### Done criteria for PR-3

- Hardware matrix v2 gate is required in local and CI release lanes.
- M15 can be marked done with artifact-linked evidence.

## Non-goals for M15 backlog

- Immediate broad non-x86 architecture expansion.
- Hardware claims outside tiered matrix pass history.

