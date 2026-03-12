# Hardware Support Matrix v7

Date: 2026-03-12  
Milestone: M54 Native Storage Drivers v1  
Lane: Rugo (Rust kernel + Go user space)  
Status: active gate contract

## Goal

Extend the active hardware matrix beyond virtio-only storage by making
emulated NVMe and AHCI storage behavior deterministic, machine-readable, and
release-gated on top of the M53 native-driver baseline.

## Tier definitions and pass criteria

| Tier | Target class | Minimum pass criteria | Gate policy |
|---|---|---|---|
| Tier 0 | QEMU reference (`q35`) | NVMe identify, admin queue, I/O queue, reset, and FUA flush stay green with no v6 regression | Release-blocking |
| Tier 1 | QEMU compatibility (`pc`/i440fx) | AHCI port-up, DMA read or write, cache flush, and reset stay green with no v6 regression | Release-blocking |
| Tier 2 | Audited bare-metal storage desktops | AHCI or NVMe evidence may inform promotion only after repeatable reset and durability campaigns stay green | Manual promotion only |
| Tier 3 | Bare-metal storage breadth candidates | Evidence-only expansion staging | Never release-blocking until promoted |
| Tier 4 | Exploratory storage profiles | Bring-up notes only | Never used for release support claims |

### Tier policy details

- Tier 0 and Tier 1 must pass `make test-hw-matrix-v7` with zero failing
  checks.
- `make test-native-storage-v1` must stay green for queue, reset, and flush
  semantics before matrix v7 evidence is considered complete.
- `rugo.hw.support_matrix.v6` remains the inherited baseline that must stay
  green while v7 broadens the storage surface.
- Emulated NVMe is release-blocking only for the bounded q35 profile declared
  below.
- Bare-metal AHCI and NVMe rows remain evidence-only until manual promotion.

## Matrix targets (v7)

| Tier | Machine profile | Native storage device | Required markers | Expected outcome |
|---|---|---|---|---|
| Tier 0 | `-machine q35` | `nvme` | `NVME: ready`, `NVME: identify ok`, `NVME: io queue ok`, `BLK: fua ok` | Deterministic release pass |
| Tier 1 | `-machine pc` (`i440fx`) | `ahci` | `AHCI: port up`, `AHCI: rw ok`, `AHCI: flush ok`, `BLK: flush ordered` | Deterministic release pass |
| Tier 2+ evidence classes | Bare-metal campaign input | audited AHCI/NVMe candidates | stable reset and durability evidence required before promotion | Policy-bounded evidence |

## Evidence artifact schema (v7)

Schema identifier: `rugo.hw_matrix_evidence.v7`

Contract identities:

- Matrix contract ID: `rugo.hw.support_matrix.v7`
- Prior matrix contract ID: `rugo.hw.support_matrix.v6`
- Driver contract ID: `rugo.driver_lifecycle_report.v6`
- Native driver contract ID: `rugo.native_driver_contract.v1`
- Native storage contract ID: `rugo.nvme_ahci_contract.v1`
- Block flush contract ID: `rugo.block_flush_contract.v1`

Required top-level fields:

- `schema`
- `created_utc`
- `matrix_contract_id`
- `prior_matrix_contract_id`
- `driver_contract_id`
- `native_driver_contract_id`
- `native_storage_contract_id`
- `block_flush_contract_id`
- `seed`
- `gate`
- `checks`
- `summary`
- `tier_results`
- `device_class_coverage`
- `storage_protocol_matrix`
- `controller_lifecycle`
- `flush_contract_checks`
- `negative_paths`
- `source_reports`
- `artifact_refs`
- `total_failures`
- `gate_pass`
- `digest`

Required `artifact_refs` fields:

- `junit`: path to `out/pytest-hw-matrix-v7.xml`
- `matrix_report`: path to `out/hw-matrix-v7.json`
- `prior_matrix_report`: path to `out/hw-matrix-v6.json`
- `native_storage_report`: path to `out/native-storage-v1.json`
- `ci_artifact`: `hw-matrix-v7-artifacts`
- `native_storage_artifact`: `native-storage-v1-artifacts`

## Executable conformance suite

- `tests/hw/test_nvme_ahci_docs_v1.py`
- `tests/hw/test_nvme_identify_v1.py`
- `tests/hw/test_nvme_io_queue_v1.py`
- `tests/hw/test_ahci_rw_v1.py`
- `tests/hw/test_native_storage_negative_v1.py`
- `tests/storage/test_nvme_fsync_integration_v1.py`
- `tests/hw/test_hw_gate_v7.py`

## Gate binding

- Local gate: `make test-hw-matrix-v7`.
- Local sub-gate: `make test-native-storage-v1`.
- CI gate: `Hardware matrix v7 gate`.
- CI sub-gate: `Native storage v1 gate`.

## Hardware claims boundary

- Matrix v7 broadens storage claims only for the declared emulated NVMe and
  AHCI profiles.
- Emulated NVMe is release-blocking for the declared q35 profile only.
- Bare-metal AHCI or NVMe evidence does not broaden support claims by itself.
- `BLK: fua ok` and `BLK: flush ordered` are release markers, not optional
  debug strings.
