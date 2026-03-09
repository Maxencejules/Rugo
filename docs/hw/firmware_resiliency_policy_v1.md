# Firmware Resiliency Policy v1

Date: 2026-03-09  
Milestone: M23  
Status: active sub-gate policy  
Version: v1

## Scope

Define protect/detect/recover obligations for firmware and boot-path integrity
across hardware matrix v3 tiers.

## Policy pillars

- Protect:
  - Signed firmware update path only.
  - Immutable trust-anchor handling with explicit key-ownership records.
  - Firmware rollback prevention for release candidates.
- Detect:
  - Measured boot evidence must be captured for each release candidate.
  - TPM event log export is mandatory and machine-readable.
  - Required PCR set `0,2,4,7` must be attested unless profile exception is
    documented and approved.
- Recover:
  - Recovery workflow must be documented and drillable.
  - Recovery drill must produce an auditable artifact with pass/fail verdict.
  - Failed attestation must block release promotion until remediation evidence
    is attached.

## Required release evidence

- Artifact schema: `rugo.measured_boot_report.v1`
- Measured-boot contract: `docs/security/measured_boot_attestation_v1.md`
- Local gate: `make test-firmware-attestation-v1`
- CI step: `Firmware attestation v1 gate`

## Policy conformance notes

- This policy is required for matrix v3 claim promotion in
  `docs/hw/support_matrix_v3.md`.
- Policy exceptions must be explicit, time-bounded, and reviewed.
