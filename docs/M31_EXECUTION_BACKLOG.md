# M31 Execution Backlog (Release Engineering + Support Lifecycle v2)

Date: 2026-03-06  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Formalize distribution-grade release lifecycle governance with support windows
and supply-chain revalidation.

M31 source of truth remains `docs/M21_M34_MATURITY_PARITY_ROADMAP.md`,
`MILESTONES.md`, and this backlog.

## Current State Summary

- Release engineering baseline exists from prior milestones.
- M31 advances branch/support governance to v2 and adds revalidation sub-gate.
- SBOM/provenance/signature drift checks are not yet enforced per candidate.

## Execution Result

- PR-1: planned
- PR-2: planned
- PR-3: planned

## PR-1: Release Lifecycle + Revalidation Contracts

### Objective

Freeze release lifecycle policy and supply-chain revalidation requirements.

### Scope

- Add docs:
  - `docs/build/release_policy_v2.md`
  - `docs/build/support_lifecycle_policy_v1.md`
  - `docs/build/supply_chain_revalidation_policy_v1.md`
  - `docs/build/release_attestation_policy_v1.md`
- Add tests:
  - `tests/build/test_release_policy_v2_docs.py`
  - `tests/build/test_supply_chain_revalidation_docs_v1.py`

### Primary files

- `docs/build/release_policy_v2.md`
- `docs/build/support_lifecycle_policy_v1.md`
- `docs/build/supply_chain_revalidation_policy_v1.md`
- `docs/build/release_attestation_policy_v1.md`
- `tests/build/test_release_policy_v2_docs.py`
- `tests/build/test_supply_chain_revalidation_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/build/test_release_policy_v2_docs.py tests/build/test_supply_chain_revalidation_docs_v1.py -v`

### Done criteria for PR-1

- Release/support/revalidation policies are explicit and test-referenced.

## PR-2: Audit + Revalidation Tooling

### Objective

Operationalize release branch/support audits and attestation drift detection.

### Scope

- Add tooling:
  - `tools/release_branch_audit_v2.py`
  - `tools/support_window_audit_v1.py`
  - `tools/verify_sbom_provenance_v2.py`
  - `tools/verify_release_attestations_v1.py`
- Add tests:
  - `tests/build/test_release_branch_policy_v2.py`
  - `tests/build/test_support_window_policy_v1.py`
  - `tests/build/test_sbom_revalidation_v1.py`
  - `tests/build/test_provenance_verification_v1.py`
  - `tests/build/test_attestation_drift_v1.py`

### Primary files

- `tools/release_branch_audit_v2.py`
- `tools/support_window_audit_v1.py`
- `tools/verify_sbom_provenance_v2.py`
- `tools/verify_release_attestations_v1.py`
- `tests/build/test_release_branch_policy_v2.py`
- `tests/build/test_support_window_policy_v1.py`
- `tests/build/test_sbom_revalidation_v1.py`
- `tests/build/test_provenance_verification_v1.py`
- `tests/build/test_attestation_drift_v1.py`

### Acceptance checks

- `python -m pytest tests/build/test_release_branch_policy_v2.py tests/build/test_support_window_policy_v1.py tests/build/test_sbom_revalidation_v1.py tests/build/test_provenance_verification_v1.py tests/build/test_attestation_drift_v1.py -v`

### Done criteria for PR-2

- Branch/support/revalidation audits are deterministic and machine-readable.
- Attestation drift is detectable with explicit policy outcome.

## PR-3: Release Lifecycle v2 Gate + Revalidation Sub-gate

### Objective

Make release lifecycle and supply-chain revalidation release-blocking.

### Scope

- Add local gates:
  - `Makefile` target `test-release-lifecycle-v2`
  - `Makefile` target `test-supply-chain-revalidation-v1`
- Add CI steps:
  - `Release lifecycle v2 gate`
  - `Supply-chain revalidation v1 gate`
- Add aggregate tests:
  - `tests/build/test_release_lifecycle_gate_v2.py`
  - `tests/build/test_supply_chain_revalidation_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/build/test_release_lifecycle_gate_v2.py`
- `tests/build/test_supply_chain_revalidation_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`

### Acceptance checks

- `make test-release-lifecycle-v2`
- `make test-supply-chain-revalidation-v1`

### Done criteria for PR-3

- Lifecycle and revalidation gates are required in local and CI lanes.
- M31 can be marked done with audit and support-window evidence.

## Non-goals for M31 backlog

- Multi-year process exception handling beyond documented policy.
- Optional-only supply-chain checks in release candidate lanes.

