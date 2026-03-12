# M53 Execution Backlog (Native Driver Contract Expansion v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: done

Archive note: this is a historical execution record. The current repo intro is
architecture-first and now lives in `README.md` plus `docs/roadmap/README.md`.

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

- M43 and M45-M47 established the reusable hardware and lifecycle baseline that
  M53 now anchors into a dedicated native-driver contract set.
- Explicit v1 contracts now define PCIe-native binding, DMA/IOMMU policy,
  firmware blob provenance, and machine-readable native-driver diagnostics.
- Deterministic diagnostics tooling now evaluates current modern-virtio and
  native in-tree classes against one shared bind, IRQ or DMA, and firmware
  evidence floor.
- The M53 contract is now wired into local and CI release gates before M54-M57
  broaden the hardware surface.

## Execution plan

- PR-1: native driver contract freeze
- PR-2: lifecycle and diagnostics campaign baseline
- PR-3: release gate wiring + milestone closure

## Execution Result

- PR-1: complete (2026-03-11)
- PR-2: complete (2026-03-11)
- PR-3: complete (2026-03-11)

## Historical Rugo implementation summary

### Historical Rust kernel surface

- `kernel_rs/src/`: native-driver lifecycle state, DMA or IOMMU policy
  enforcement, firmware provenance checks, and machine-readable diagnostics
  emission boundaries defined for later family-specific work.
- `arch/` and `boot/`: low-level interrupt, descriptor-table, and
  early-enumeration expectations stayed bounded to keeping diagnostics
  deterministic at boot.

### Historical Go user space surface

- `services/go/`: operator-facing diagnostics consumption, policy wiring, and
  failure-marker reporting for native-driver lifecycle and firmware-denial
  paths.
- `services/go_std/`: optional parity spike only. It could inform the contract
  shape, but it did not define M53 completion.

### Historical Language-Native Verification

- `make kernel`
- `make userspace`
- `make image-demo`
- `make smoke-demo`
- `python tools/run_native_driver_diagnostics_v1.py --out out/native-driver-diagnostics-v1.json`
- `make test-native-driver-contract-v1`
- `make test-native-driver-diagnostics-v1`

## PR-1: Native Driver Contract Freeze

### Objective

Define the contract surface for post-M52 native-driver work before new hardware
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

- Native driver lifecycle, DMA or IOMMU, firmware provenance, and diagnostics
  boundaries are explicit and versioned.
- Unsupported firmware and unsafe DMA paths are described as deterministic
  failures, not open-ended TODOs.
- M54-M57 can reference a stable native-driver baseline instead of extending
  matrix v6 ad hoc.

### PR-1 completion summary

- Added contract docs:
  - `docs/hw/native_driver_contract_v1.md`
  - `docs/hw/pcie_dma_contract_v1.md`
  - `docs/hw/firmware_blob_policy_v1.md`
  - `docs/hw/native_driver_diag_schema_v1.md`
- Added executable doc checks:
  - `tests/hw/test_native_driver_docs_v1.py`
  - `tests/hw/test_pcie_dma_contract_v1.py`
  - `tests/hw/test_firmware_blob_policy_v1.py`

## PR-2: Lifecycle and Diagnostics Campaign Baseline

### Objective

Implement deterministic evidence collection for the generic native-driver
contract using current in-tree device classes before NVMe, GPU, or Wi-Fi-
specific feature work lands.

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
- Bind, init, runtime, and negative-path expectations emit stable markers such
  as `DRV: bind`, `IRQ: vector bound`, `DMA: map ok`, and
  `FW: denied unsigned`.
- Existing in-tree native and modern-virtio classes can be evaluated against
  one shared lifecycle and diagnostics baseline.

### PR-2 completion summary

- Added deterministic native-driver diagnostics tooling:
  - `tools/run_native_driver_diagnostics_v1.py`
- Added executable lifecycle and policy checks:
  - `tests/hw/test_driver_bind_lifecycle_v1.py`
  - `tests/hw/test_irq_dma_policy_v1.py`
  - `tests/hw/test_firmware_blob_enforcement_v1.py`
  - `tests/hw/test_native_driver_diagnostics_v1.py`

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

### PR-3 completion summary

- Added local gates:
  - `make test-native-driver-contract-v1`
  - `make test-native-driver-diagnostics-v1`
- Added aggregate gate tests:
  - `tests/hw/test_native_driver_contract_gate_v1.py`
  - `tests/hw/test_native_driver_diag_gate_v1.py`
- Updated repo-level closure documents:
  - `MILESTONES.md`
  - `docs/STATUS.md`
  - `README.md`

## Non-goals for M53 backlog

- implementing full NVMe or AHCI driver support owned by M54
- implementing native GPU acceleration owned by M55
- implementing Wi-Fi device and control-plane support owned by M56
- promoting non-x86 architectures beyond the shadow bring-up owned by M57
- broadening support-tier claims without deterministic native-driver evidence
