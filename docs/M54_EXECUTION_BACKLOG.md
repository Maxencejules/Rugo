# M54 Execution Backlog (Native Storage Drivers v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Move storage support beyond virtio-first assumptions by adding bounded NVMe and
AHCI baseline support with explicit queue, reset, and flush semantics.

M54 source of truth remains:

- `docs/POST_G2_EXTENDED_MILESTONES.md`
- `docs/M53_EXECUTION_BACKLOG.md`
- `docs/storage/fs_v1.md`
- this backlog

## Current State Summary

- M53 establishes the generic native-driver contract needed for safe PCIe/DMA
  expansion.
- Existing storage reliability and feature work assumes virtio-backed block
  paths for the release floor.
- Native NVMe and AHCI behavior is not yet a versioned, release-gated contract.
- M54 must freeze those device-family semantics before broader Tier 1 storage
  claims are made.

## Execution plan

- PR-1: native storage contract freeze
- PR-2: NVMe/AHCI campaign baseline
- PR-3: native storage gate wiring + closure

## Execution status

- PR-1: pending
- PR-2: pending
- PR-3: pending

## PR-1: Native Storage Contract Freeze

### Objective

Define NVMe/AHCI semantics, matrix targets, and block-flush guarantees before
implementation broadens support claims.

### Scope

- Add docs:
  - `docs/hw/nvme_ahci_contract_v1.md`
  - `docs/hw/support_matrix_v7.md`
  - `docs/storage/block_flush_contract_v1.md`
- Add tests:
  - `tests/hw/test_nvme_ahci_docs_v1.py`
  - `tests/storage/test_block_flush_contract_v1.py`

### Primary files

- `docs/hw/nvme_ahci_contract_v1.md`
- `docs/hw/support_matrix_v7.md`
- `docs/storage/block_flush_contract_v1.md`
- `tests/hw/test_nvme_ahci_docs_v1.py`
- `tests/storage/test_block_flush_contract_v1.py`

### Acceptance checks

- `python -m pytest tests/hw/test_nvme_ahci_docs_v1.py tests/storage/test_block_flush_contract_v1.py -v`

### Done criteria for PR-1

- NVMe/AHCI queue, reset, and flush semantics are explicit and versioned.
- Matrix v7 target rows and native-storage negative paths are reviewable before
  driver enablement lands.

## PR-2: NVMe/AHCI Campaign Baseline

### Objective

Implement deterministic evidence collection for native storage classes and bind
them to the storage durability model.

### Scope

- Add tooling:
  - `tools/run_hw_matrix_v7.py`
  - `tools/run_native_storage_diagnostics_v1.py`
- Add tests:
  - `tests/hw/test_nvme_identify_v1.py`
  - `tests/hw/test_nvme_io_queue_v1.py`
  - `tests/hw/test_ahci_rw_v1.py`
  - `tests/storage/test_nvme_fsync_integration_v1.py`
  - `tests/hw/test_native_storage_negative_v1.py`

### Primary files

- `tools/run_hw_matrix_v7.py`
- `tools/run_native_storage_diagnostics_v1.py`
- `tests/hw/test_nvme_identify_v1.py`
- `tests/hw/test_nvme_io_queue_v1.py`
- `tests/hw/test_ahci_rw_v1.py`
- `tests/storage/test_nvme_fsync_integration_v1.py`
- `tests/hw/test_native_storage_negative_v1.py`

### Acceptance checks

- `python tools/run_hw_matrix_v7.py --out out/hw-matrix-v7.json`
- `python tools/run_native_storage_diagnostics_v1.py --out out/native-storage-v1.json`
- `python -m pytest tests/hw/test_nvme_identify_v1.py tests/hw/test_nvme_io_queue_v1.py tests/hw/test_ahci_rw_v1.py tests/storage/test_nvme_fsync_integration_v1.py tests/hw/test_native_storage_negative_v1.py -v`

### Done criteria for PR-2

- Native-storage artifacts are deterministic and machine-readable.
- `NVME: ready`, `AHCI: port up`, and `BLK: fua ok` style markers are stable.
- Storage durability tests can name the native device class used for evidence.

## PR-3: Native Storage Gate + Matrix v7 Sub-gate

### Objective

Make native storage qualification enforceable in local and CI lanes.

### Scope

- Add local gates:
  - `Makefile` target `test-native-storage-v1`
  - `Makefile` target `test-hw-matrix-v7`
- Add CI steps:
  - `Native storage v1 gate`
  - `Hardware matrix v7 gate`
- Add aggregate tests:
  - `tests/hw/test_native_storage_gate_v1.py`
  - `tests/hw/test_hw_gate_v7.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/hw/test_native_storage_gate_v1.py`
- `tests/hw/test_hw_gate_v7.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-native-storage-v1`
- `make test-hw-matrix-v7`

### Done criteria for PR-3

- Native storage and matrix v7 sub-gates are required in local and CI release
  lanes.
- M54 can be marked done only with release-gated NVMe/AHCI evidence and no
  undocumented fallback broadening.

## Non-goals for M54 backlog

- full filesystem feature expansion owned by M58-M61
- native GPU acceleration owned by M55
- Wi-Fi adapter support owned by M56
- support-tier promotion without native storage gate evidence
