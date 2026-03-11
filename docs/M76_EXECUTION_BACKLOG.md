# M76 Execution Backlog (Compliance Profiles + Audit Evidence v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Add explicit compliance profiles and audit-evidence semantics so security and
release claims can be mapped to machine-auditable controls.

M76 source of truth remains:

- `docs/POST_G2_EXTENDED_MILESTONES.md`
- `docs/security/rights_capability_model_v1.md`
- `docs/M75_EXECUTION_BACKLOG.md`
- this backlog

## Current State Summary

- Security hardening and update orchestration exist, but there is no
  compliance-profile or audit-evidence contract in the post-M75 plan.
- Runtime evidence and release policy work provide building blocks for audit
  output, but not formalized control mapping.
- Compliance claims remain a roadmap-level concept rather than a gated
  contract.
- M76 must define those semantics before attestation and community release
  operations depend on them.

## Execution plan

- PR-1: compliance and audit contract freeze
- PR-2: control-mapping and redaction campaign baseline
- PR-3: compliance gate wiring + closure

## Execution status

- PR-1: pending
- PR-2: pending
- PR-3: pending

## PR-1: Compliance and Audit Contract Freeze

### Objective

Define compliance-profile, audit-evidence, and release-checklist semantics
before implementation broadens audit claims.

### Scope

- Add docs:
  - `docs/security/compliance_profile_v1.md`
  - `docs/security/audit_evidence_contract_v1.md`
  - `docs/build/compliance_release_checklist_v1.md`
- Add tests:
  - `tests/security/test_compliance_docs_v1.py`
  - `tests/build/test_compliance_release_docs_v1.py`

### Primary files

- `docs/security/compliance_profile_v1.md`
- `docs/security/audit_evidence_contract_v1.md`
- `docs/build/compliance_release_checklist_v1.md`
- `tests/security/test_compliance_docs_v1.py`
- `tests/build/test_compliance_release_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/security/test_compliance_docs_v1.py tests/build/test_compliance_release_docs_v1.py -v`

### Done criteria for PR-1

- Compliance and audit-evidence semantics are explicit and versioned.
- Redaction, retention, and unsupported-control paths remain deterministic.

## PR-2: Control-mapping and Redaction Campaign Baseline

### Objective

Implement deterministic evidence for control mapping, audit bundle generation,
and redaction policy.

### Scope

- Add tooling:
  - `tools/run_compliance_audit_v1.py`
  - `tools/run_audit_bundle_redaction_v1.py`
- Add tests:
  - `tests/security/test_compliance_control_mapping_v1.py`
  - `tests/security/test_audit_bundle_redaction_v1.py`
  - `tests/build/test_compliance_release_gate_v1.py`
  - `tests/security/test_compliance_negative_v1.py`

### Primary files

- `tools/run_compliance_audit_v1.py`
- `tools/run_audit_bundle_redaction_v1.py`
- `tests/security/test_compliance_control_mapping_v1.py`
- `tests/security/test_audit_bundle_redaction_v1.py`
- `tests/build/test_compliance_release_gate_v1.py`
- `tests/security/test_compliance_negative_v1.py`

### Acceptance checks

- `python tools/run_compliance_audit_v1.py --out out/compliance-audit-v1.json`
- `python tools/run_audit_bundle_redaction_v1.py --out out/audit-redaction-v1.json`
- `python -m pytest tests/security/test_compliance_control_mapping_v1.py tests/security/test_audit_bundle_redaction_v1.py tests/build/test_compliance_release_gate_v1.py tests/security/test_compliance_negative_v1.py -v`

### Done criteria for PR-2

- Compliance artifacts are deterministic and machine-readable.
- `COMP: profile ok` and redaction markers are stable.
- Audit bundles can be validated against explicit control and checklist IDs.

## PR-3: Compliance Gate + Audit Evidence Sub-gate

### Objective

Make compliance and audit-evidence behavior release-blocking for declared
security profiles.

### Scope

- Add local gates:
  - `Makefile` target `test-compliance-profile-v1`
  - `Makefile` target `test-audit-evidence-v1`
- Add CI steps:
  - `Compliance profile v1 gate`
  - `Audit evidence v1 gate`
- Add aggregate tests:
  - `tests/security/test_compliance_gate_v1.py`
  - `tests/security/test_audit_evidence_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/security/test_compliance_gate_v1.py`
- `tests/security/test_audit_evidence_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-compliance-profile-v1`
- `make test-audit-evidence-v1`

### Done criteria for PR-3

- Compliance and audit-evidence sub-gates are required in local and CI release
  lanes.
- M76 can be marked done only with deterministic control-mapping and audit
  bundle evidence for declared security profiles.

## Non-goals for M76 backlog

- attested fleet admission work owned by M77
- community release-train operations owned by M84
- best-effort-only audit output outside declared policy
