# Measured Boot Attestation v1

Date: 2026-03-09  
Milestone: M23  
Status: active sub-gate contract  
Version: v1

## Objective

Define measured-boot attestation contract for release evidence and firmware
resiliency policy enforcement.

## Contract

- TPM event log is exported and machine-readable.
- PCR set must include `0,2,4,7` unless profile documents an approved exception.
- Policy verdict must be explicit: pass/fail with reason list.
- Event log entries must include `pcr`, `type`, `component`, and SHA-256
  `digest`.

## Artifact schema

Schema identifier: `rugo.measured_boot_report.v1`

Required top-level fields:

- `schema`
- `created_utc`
- `firmware_policy`
- `attestation_contract_id`
- `platform`
- `policy_profile`
- `pcr_bank`
- `expected_pcrs`
- `pcrs`
- `tpm_event_log`
- `event_count`
- `policy_pass`
- `failures`
- `attestation_verdict`

## Required policy bindings

- Firmware policy: `docs/hw/firmware_resiliency_policy_v1.md`
- Local gate: `make test-firmware-attestation-v1`
- CI gate: `.github/workflows/ci.yml` step `Firmware attestation v1 gate`

## Evidence

- Tool: `tools/collect_measured_boot_report_v1.py`
- Tests:
  - `tests/hw/test_measured_boot_attestation_v1.py`
  - `tests/hw/test_tpm_eventlog_schema_v1.py`
  - `tests/hw/test_firmware_attestation_gate_v1.py`
