# M53 Execution Backlog (Native Driver Contract Expansion v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Freeze the post-M52 native-driver expansion boundary so storage, GPU, Wi-Fi,
and later multi-arch work can build on explicit lifecycle, DMA, firmware, and
diagnostics contracts instead of implied behavior.

M53 source of truth remains:

- `docs/POST_G2_EXTENDED_MILESTONES.md`
- `docs/hw/support_matrix_v6.md`
- `docs/hw/driver_lifecycle_contract_v6.md`
- this backlog

## Current State Summary

- M43 and M45-M47 established broader native-driver, firmware, and promotion
  evidence, but the contracts are still split across matrix-specific and
  device-class-specific docs.
- M48-M52 made the desktop stack usable, increasing pressure to support native
  NVMe, GPU, and Wi-Fi classes without weakening the current claim discipline.
- There is no dedicated v1 contract yet for PCIe-native binding, DMA/IOMMU
  safety, firmware blob provenance, or machine-readable native-driver
  diagnostics.
- M53 must close those contract gaps before M54-M57 broaden support claims or
  introduce new hardware-family implementation pressure.

## Execution plan

- PR-1: native driver contract freeze
- PR-2: lifecycle and diagnostics campaign baseline
- PR-3: release gate wiring + milestone closure

## Execution status

- PR-1: pending
- PR-2: pending
- PR-3: pending

## PR-1: Native Driver Contract Freeze

### Objective

Define the contract surface for post-M52 native driver work before new hardware
families expand the support matrix.

### Scope

- Add docs:
  - `docs/hw/native_driver_contract_v1.md`
  - `docs/hw/pcie_dma_contract_v1.md`
  - `docs/hw/firmware_blob_policy_v1.md`
  - `docs/hw/native_driver_diag_schema_v1.md`
- Add tests:
  - `tests/hw/test_native_driver_docs_v1.py`
  - `tests/hw/test_pcie_dma_contract_v1.py`
  - `tests/hw/test_firmware_blob_policy_v1.py`

### Primary files

- `docs/hw/native_driver_contract_v1.md`
- `docs/hw/pcie_dma_contract_v1.md`
- `docs/hw/firmware_blob_policy_v1.md`
- `docs/hw/native_driver_diag_schema_v1.md`
- `tests/hw/test_native_driver_docs_v1.py`
- `tests/hw/test_pcie_dma_contract_v1.py`
- `tests/hw/test_firmware_blob_policy_v1.py`

### Acceptance checks

- `python -m pytest tests/hw/test_native_driver_docs_v1.py tests/hw/test_pcie_dma_contract_v1.py tests/hw/test_firmware_blob_policy_v1.py -v`

### Done criteria for PR-1

- Native driver lifecycle, DMA/IOMMU, firmware provenance, and diagnostics
  boundaries are explicit and versioned.
- Unsupported firmware and unsafe DMA paths are described as deterministic
  failures, not open-ended TODOs.
- M54-M57 can reference a stable native-driver baseline instead of extending
  matrix v6 ad hoc.

## PR-2: Lifecycle and Diagnostics Campaign Baseline

### Objective

Implement deterministic evidence collection for the generic native-driver
contract using current in-tree device classes before NVMe/GPU/Wi-Fi-specific
feature work lands.

### Scope

- Add tooling:
  - `tools/run_native_driver_diagnostics_v1.py`
- Add tests:
  - `tests/hw/test_driver_bind_lifecycle_v1.py`
  - `tests/hw/test_irq_dma_policy_v1.py`
  - `tests/hw/test_firmware_blob_enforcement_v1.py`
  - `tests/hw/test_native_driver_diagnostics_v1.py`

### Primary files

- `tools/run_native_driver_diagnostics_v1.py`
- `tests/hw/test_driver_bind_lifecycle_v1.py`
- `tests/hw/test_irq_dma_policy_v1.py`
- `tests/hw/test_firmware_blob_enforcement_v1.py`
- `tests/hw/test_native_driver_diagnostics_v1.py`

### Acceptance checks

- `python tools/run_native_driver_diagnostics_v1.py --out out/native-driver-diagnostics-v1.json`
- `python -m pytest tests/hw/test_driver_bind_lifecycle_v1.py tests/hw/test_irq_dma_policy_v1.py tests/hw/test_firmware_blob_enforcement_v1.py tests/hw/test_native_driver_diagnostics_v1.py -v`

### Done criteria for PR-2

- Deterministic native-driver diagnostics artifacts are machine-readable and
  schema-validated.
- Bind/init/runtime/negative-path expectations emit stable markers such as
  `DRV: bind`, `IRQ: vector bound`, `DMA: map ok`, and
  `FW: denied unsigned`.
- Existing in-tree native and modern-virtio classes can be evaluated against
  one shared lifecycle and diagnostics baseline.

## PR-3: Native Driver Gate + Diagnostics Sub-gate

### Objective

Make the M53 contract enforceable in local and CI lanes before M54-M57 widen
the hardware surface.

### Scope

- Add local gates:
  - `Makefile` target `test-native-driver-contract-v1`
  - `Makefile` target `test-native-driver-diagnostics-v1`
- Add CI steps:
  - `Native driver contract v1 gate`
  - `Native driver diagnostics v1 gate`
- Add aggregate tests:
  - `tests/hw/test_native_driver_contract_gate_v1.py`
  - `tests/hw/test_native_driver_diag_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/hw/test_native_driver_contract_gate_v1.py`
- `tests/hw/test_native_driver_diag_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-native-driver-contract-v1`
- `make test-native-driver-diagnostics-v1`

### Done criteria for PR-3

- Native-driver contract and diagnostics sub-gates are required in local and CI
  release lanes.
- Firmware provenance drift, missing diagnostics fields, and unsafe DMA policy
  regressions block milestone closure.
- M53 can be marked done only when M54-M57 dependencies point to release-gated
  native-driver contracts rather than roadmap-only intent.

## Non-goals for M53 backlog

- implementing full NVMe/AHCI driver support owned by M54
- implementing native GPU acceleration owned by M55
- implementing Wi-Fi device and control-plane support owned by M56
- promoting non-x86 architectures beyond the shadow bring-up owned by M57
- broadening support-tier claims without deterministic native-driver evidence
