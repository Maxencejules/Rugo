# M28 Execution Backlog (Security Hardening Program v3)

Date: 2026-03-06  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Evolve from baseline security controls to mature hardening and operational
vulnerability-response posture.

M28 source of truth remains `docs/M21_M34_MATURITY_PARITY_ROADMAP.md`,
`MILESTONES.md`, and this backlog.

## Current State Summary

- Security baseline and fuzzing workflows exist from earlier milestones.
- M28 adds hardened policy enforcement at v3 plus response-process maturity.
- New sub-gate `test-vuln-response-v1` must be release-blocking.

## Execution Result

- PR-1: planned
- PR-2: planned
- PR-3: planned

## PR-1: Hardening + Vulnerability Response Contracts

### Objective

Freeze hardening profile and vulnerability/advisory policy contracts.

### Scope

- Add docs:
  - `docs/security/hardening_profile_v3.md`
  - `docs/security/threat_model_v2.md`
  - `docs/security/vulnerability_response_policy_v1.md`
  - `docs/security/security_advisory_policy_v1.md`
- Add tests:
  - `tests/security/test_hardening_docs_v3.py`
  - `tests/security/test_vuln_response_docs_v1.py`

### Primary files

- `docs/security/hardening_profile_v3.md`
- `docs/security/threat_model_v2.md`
- `docs/security/vulnerability_response_policy_v1.md`
- `docs/security/security_advisory_policy_v1.md`
- `tests/security/test_hardening_docs_v3.py`
- `tests/security/test_vuln_response_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/security/test_hardening_docs_v3.py tests/security/test_vuln_response_docs_v1.py -v`

### Done criteria for PR-1

- Hardening and vuln-response contracts are versioned and test-referenced.

## PR-2: Enforcement, Fuzzing, Advisory, and Embargo Drills

### Objective

Operationalize hardening controls and vulnerability-response workflows.

### Scope

- Add tooling:
  - `tools/run_security_attack_suite_v3.py`
  - `tools/run_security_fuzz_v2.py`
  - `tools/security_advisory_lint_v1.py`
  - `tools/security_embargo_drill_v1.py`
- Add tests:
  - `tests/security/test_attack_suite_v3.py`
  - `tests/security/test_fuzz_gate_v2.py`
  - `tests/security/test_policy_enforcement_v3.py`
  - `tests/security/test_vuln_triage_sla_v1.py`
  - `tests/security/test_embargo_workflow_v1.py`
  - `tests/security/test_advisory_schema_v1.py`

### Primary files

- `tools/run_security_attack_suite_v3.py`
- `tools/run_security_fuzz_v2.py`
- `tools/security_advisory_lint_v1.py`
- `tools/security_embargo_drill_v1.py`
- `tests/security/test_attack_suite_v3.py`
- `tests/security/test_fuzz_gate_v2.py`
- `tests/security/test_policy_enforcement_v3.py`
- `tests/security/test_vuln_triage_sla_v1.py`
- `tests/security/test_embargo_workflow_v1.py`
- `tests/security/test_advisory_schema_v1.py`

### Acceptance checks

- `python -m pytest tests/security/test_attack_suite_v3.py tests/security/test_fuzz_gate_v2.py tests/security/test_policy_enforcement_v3.py tests/security/test_vuln_triage_sla_v1.py tests/security/test_embargo_workflow_v1.py tests/security/test_advisory_schema_v1.py -v`

### Done criteria for PR-2

- Hardening/fuzz outcomes and advisory workflow drills are deterministic.
- Triage/embargo/advisory artifacts are machine-readable and auditable.

## PR-3: Security v3 Gate + Vulnerability Response Sub-gate

### Objective

Make hardening and vulnerability-response checks release-blocking.

### Scope

- Add local gates:
  - `Makefile` target `test-security-hardening-v3`
  - `Makefile` target `test-vuln-response-v1`
- Add CI steps:
  - `Security hardening v3 gate`
  - `Vulnerability response v1 gate`
- Add aggregate tests:
  - `tests/security/test_security_hardening_gate_v3.py`
  - `tests/security/test_vuln_response_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/security/test_security_hardening_gate_v3.py`
- `tests/security/test_vuln_response_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`

### Acceptance checks

- `make test-security-hardening-v3`
- `make test-vuln-response-v1`

### Done criteria for PR-3

- Security and vulnerability-response gates are required in local and CI lanes.
- M28 can be marked done with hardening and advisory evidence pointers.

## Non-goals for M28 backlog

- Unscoped expansion of security controls without contract/test ownership.
- Best-effort-only vulnerability handling outside defined SLA workflow.

