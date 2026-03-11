# M77 Execution Backlog (Secrets, Attestation, and Fleet Admission v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Add bounded secret sealing, attested identity, and fleet admission semantics
for managed systems without broadening the release floor beyond audited
hardware.

M77 source of truth remains:

- `docs/POST_G2_EXTENDED_MILESTONES.md`
- `docs/M23_EXECUTION_BACKLOG.md`
- `docs/M76_EXECUTION_BACKLOG.md`
- this backlog

## Current State Summary

- Measured boot and fleet-style operations exist as earlier building blocks,
  and M76 adds compliance evidence semantics.
- There is no versioned attested identity, secret sealing, or fleet admission
  contract in the post-M75 security plan.
- Managed fleet trust remains outside current release-gated claims.
- M77 must define those semantics before community release operations or later
  fleet policies build on them.

## Execution plan

- PR-1: attestation and secret-sealing contract freeze
- PR-2: sealing and admission campaign baseline
- PR-3: fleet-admission gate wiring + closure

## Execution status

- PR-1: pending
- PR-2: pending
- PR-3: pending

## PR-1: Attestation and Secret-sealing Contract Freeze

### Objective

Define attested identity, secret sealing, and fleet admission semantics before
implementation broadens managed-fleet claims.

### Scope

- Add docs:
  - `docs/security/attested_identity_contract_v1.md`
  - `docs/security/secret_sealing_policy_v1.md`
  - `docs/pkg/fleet_admission_contract_v1.md`
- Add tests:
  - `tests/security/test_attested_identity_docs_v1.py`

### Primary files

- `docs/security/attested_identity_contract_v1.md`
- `docs/security/secret_sealing_policy_v1.md`
- `docs/pkg/fleet_admission_contract_v1.md`
- `tests/security/test_attested_identity_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/security/test_attested_identity_docs_v1.py -v`

### Done criteria for PR-1

- Attested identity, sealing, and admission semantics are explicit and
  versioned.
- Simulation-only and unaudited-hardware paths are clearly bounded.

## PR-2: Sealing and Admission Campaign Baseline

### Objective

Implement deterministic evidence for secret sealing, attestation admission, and
fleet denial behavior.

### Scope

- Add tooling:
  - `tools/run_secret_sealing_v1.py`
  - `tools/run_attestation_admission_v1.py`
- Add tests:
  - `tests/security/test_secret_sealing_v1.py`
  - `tests/security/test_attestation_admission_v1.py`
  - `tests/pkg/test_fleet_policy_denial_v1.py`
  - `tests/security/test_attestation_negative_v1.py`

### Primary files

- `tools/run_secret_sealing_v1.py`
- `tools/run_attestation_admission_v1.py`
- `tests/security/test_secret_sealing_v1.py`
- `tests/security/test_attestation_admission_v1.py`
- `tests/pkg/test_fleet_policy_denial_v1.py`
- `tests/security/test_attestation_negative_v1.py`

### Acceptance checks

- `python tools/run_secret_sealing_v1.py --out out/secret-sealing-v1.json`
- `python tools/run_attestation_admission_v1.py --out out/attestation-admission-v1.json`
- `python -m pytest tests/security/test_secret_sealing_v1.py tests/security/test_attestation_admission_v1.py tests/pkg/test_fleet_policy_denial_v1.py tests/security/test_attestation_negative_v1.py -v`

### Done criteria for PR-2

- Attestation artifacts are deterministic and machine-readable.
- `ATTEST: admit ok` and `SECRET: unseal ok` markers are stable.
- Fleet denial and unaudited-hardware behavior remains explicit and auditable.

## PR-3: Fleet Admission Gate + Secret Sealing Sub-gate

### Objective

Make attested admission behavior release-blocking for declared managed-fleet
profiles.

### Scope

- Add local gates:
  - `Makefile` target `test-fleet-admission-v1`
  - `Makefile` target `test-secret-sealing-v1`
- Add CI steps:
  - `Fleet admission v1 gate`
  - `Secret sealing v1 gate`
- Add aggregate tests:
  - `tests/security/test_fleet_admission_gate_v1.py`
  - `tests/security/test_secret_sealing_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/security/test_fleet_admission_gate_v1.py`
- `tests/security/test_secret_sealing_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-fleet-admission-v1`
- `make test-secret-sealing-v1`

### Done criteria for PR-3

- Fleet-admission and secret-sealing sub-gates are required in local and CI
  release lanes.
- M77 can be marked done only with deterministic attestation and denial-path
  evidence on audited profiles.

## Non-goals for M77 backlog

- broad unaudited hardware admission claims
- community governance and support work owned by M82-M84
- replacing measured-boot and fleet policy with ad hoc trust shortcuts
