# Native Driver Diagnostics Schema v1

Date: 2026-03-11  
Milestone: M53 Native Driver Contract Expansion v1  
Lane: Rugo (Rust kernel + Go user space)  
Status: active report schema

## Schema identity

Schema identifier: `rugo.native_driver_diagnostics_report.v1`
Parent contract ID: `rugo.native_driver_contract.v1`.
PCIe DMA contract ID: `rugo.pcie_dma_contract.v1`.
Firmware blob policy ID: `rugo.firmware_blob_policy.v1`.
Lifecycle contract ID: `rugo.driver_lifecycle_report.v6`.
Support matrix ID: `rugo.hw.support_matrix.v6`.

## Required top-level fields

- `schema`
- `created_utc`
- `contract_id`
- `pcie_dma_contract_id`
- `firmware_blob_policy_id`
- `diag_schema_id`
- `driver_lifecycle_contract_id`
- `support_matrix_id`
- `seed`
- `gate`
- `contract_gate`
- `checks`
- `summary`
- `driver_bindings`
- `irq_audits`
- `dma_policy`
- `dma_audits`
- `firmware_policy`
- `firmware_audits`
- `diagnostic_events`
- `source_reports`
- `artifact_refs`
- `injected_failures`
- `max_failures`
- `total_failures`
- `failures`
- `gate_pass`
- `digest`

## Required `driver_bindings` fields

- `driver`
- `profile`
- `device_class`
- `source_schema`
- `source_digest`
- `states_observed`
- `bind_latency_ms`
- `markers`
- `status`

## Required `diagnostic_events` fields

- `event_id`
- `driver`
- `device_class`
- `profile`
- `phase`
- `severity`
- `marker`
- `status`
- `details`

## Required markers

- `DRV: bind`
- `IRQ: vector bound`
- `DMA: map ok`
- `DMA: map bounce`
- `DMA: deny unsafe`
- `FW: allow signed`
- `FW: denied unsigned`
- `FW: denied missing manifest`
- `FW: denied hash mismatch`

## Required `artifact_refs` fields

- `junit`
- `diagnostics_report`
- `matrix_report`
- `baremetal_io_report`
- `ci_artifact`
- `contract_ci_artifact`

## Gate binding

- Local gate: `make test-native-driver-contract-v1`.
- Local sub-gate: `make test-native-driver-diagnostics-v1`.
- CI gate: `Native driver contract v1 gate`.
- CI sub-gate: `Native driver diagnostics v1 gate`.
