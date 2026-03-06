# M23 Execution Backlog (Hardware Enablement Matrix v3)

Date: 2026-03-06  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Expand hardware matrix confidence to v3 with deterministic driver lifecycle and
firmware/measured-boot attestation evidence.

M23 source of truth remains `docs/M21_M34_MATURITY_PARITY_ROADMAP.md`,
`MILESTONES.md`, and this backlog.

## Current State Summary

- Hardware matrix baselines exist from prior v1/v2 planning.
- M23 adds firmware resiliency and measured-boot evidence as required sub-gate.
- No v3 matrix gate artifacts are present yet.

## Execution Result

- PR-1: planned
- PR-2: planned
- PR-3: planned

## PR-1: Matrix v3 + Firmware Contracts

### Objective

Freeze hardware support tiers, driver lifecycle, and firmware-attestation policy.

### Scope

- Add docs:
  - `docs/hw/support_matrix_v3.md`
  - `docs/hw/driver_lifecycle_contract_v3.md`
  - `docs/hw/firmware_resiliency_policy_v1.md`
  - `docs/security/measured_boot_attestation_v1.md`
- Add tests:
  - `tests/hw/test_hardware_matrix_v3.py`
  - `tests/hw/test_driver_lifecycle_v3.py`
  - `tests/hw/test_firmware_resiliency_docs_v1.py`

### Primary files

- `docs/hw/support_matrix_v3.md`
- `docs/hw/driver_lifecycle_contract_v3.md`
- `docs/hw/firmware_resiliency_policy_v1.md`
- `docs/security/measured_boot_attestation_v1.md`
- `tests/hw/test_hardware_matrix_v3.py`
- `tests/hw/test_driver_lifecycle_v3.py`

### Acceptance checks

- `python -m pytest tests/hw/test_hardware_matrix_v3.py tests/hw/test_driver_lifecycle_v3.py tests/hw/test_firmware_resiliency_docs_v1.py -v`

### Done criteria for PR-1

- Tier claims and driver lifecycle obligations are explicit and test-referenced.
- Firmware/measured-boot policy is versioned and reviewable.

## PR-2: Suspend/Hotplug + Measured-Boot Evidence

### Objective

Expand matrix behavior coverage and emit measured-boot artifacts.

### Scope

- Add tests:
  - `tests/hw/test_suspend_resume_v1.py`
  - `tests/hw/test_hotplug_baseline_v1.py`
  - `tests/hw/test_measured_boot_attestation_v1.py`
  - `tests/hw/test_tpm_eventlog_schema_v1.py`
- Add tooling:
  - `tools/collect_hw_diagnostics_v3.py`
  - `tools/collect_measured_boot_report_v1.py`

### Primary files

- `tests/hw/test_suspend_resume_v1.py`
- `tests/hw/test_hotplug_baseline_v1.py`
- `tests/hw/test_measured_boot_attestation_v1.py`
- `tests/hw/test_tpm_eventlog_schema_v1.py`
- `tools/collect_hw_diagnostics_v3.py`
- `tools/collect_measured_boot_report_v1.py`

### Acceptance checks

- `python tools/collect_measured_boot_report_v1.py --out out/measured-boot-v1.json`
- `python -m pytest tests/hw/test_suspend_resume_v1.py tests/hw/test_hotplug_baseline_v1.py tests/hw/test_measured_boot_attestation_v1.py tests/hw/test_tpm_eventlog_schema_v1.py -v`

### Done criteria for PR-2

- Firmware resiliency and measured-boot evidence are deterministic and auditable.
- Matrix behavioral tests cover suspend/hotplug baseline paths.

## PR-3: Hardware v3 Gate + Firmware Sub-gate

### Objective

Make hardware matrix v3 and firmware-attestation checks release-blocking.

### Scope

- Add local gates:
  - `Makefile` target `test-hw-matrix-v3`
  - `Makefile` target `test-firmware-attestation-v1`
- Add CI steps:
  - `Hardware matrix v3 gate`
  - `Firmware attestation v1 gate`
- Add aggregate tests:
  - `tests/hw/test_hw_gate_v3.py`
  - `tests/hw/test_firmware_attestation_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/hw/test_hw_gate_v3.py`
- `tests/hw/test_firmware_attestation_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`

### Acceptance checks

- `make test-hw-matrix-v3`
- `make test-firmware-attestation-v1`

### Done criteria for PR-3

- Hardware and firmware-attestation gates are required in local and CI lanes.
- M23 can be marked done with matrix and attestation artifact evidence.

## Non-goals for M23 backlog

- Broad unsupported hardware-family claims beyond declared matrix tiers.
- Full firmware ecosystem parity across all vendor stacks.

