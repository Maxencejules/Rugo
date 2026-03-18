# M28 Execution Backlog (Security Hardening Program v3)

Date: 2026-03-06  
Lane: Rugo (Rust kernel + Go user space)  
Status: done

## Goal

Evolve from baseline security controls to mature hardening and operational
vulnerability-response posture.

M28 source of truth remains `docs/M21_M34_MATURITY_PARITY_ROADMAP.md`,
`MILESTONES.md`, and this backlog.

## Current State Summary

- Hardening profile v3 and threat model v2 are now explicit and versioned.
- Attack-suite evidence now binds to booted runtime capture from the shipped lane.
- Fuzz, advisory, and embargo artifacts remain deterministic and release-auditable.
- Sub-gate `test-vuln-response-v1` is release-blocking in local and CI lanes.

## Execution Result

- PR-1: complete (2026-03-09)
- PR-2: complete (2026-03-09)
- PR-3: complete (2026-03-09)

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

### PR-1 completion summary

- Added hardening contract docs:
  - `docs/security/hardening_profile_v3.md`
  - `docs/security/threat_model_v2.md`
- Upgraded vulnerability-response/advisory policy contracts:
  - `docs/security/vulnerability_response_policy_v1.md`
  - `docs/security/security_advisory_policy_v1.md`
- Added executable PR-1 doc contract checks:
  - `tests/security/test_hardening_docs_v3.py`
  - `tests/security/test_vuln_response_docs_v1.py`

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

- Hardening outcomes are bound to booted runtime evidence from the shipped image.
- Fuzz outcomes and advisory workflow drills are deterministic.
- Triage/embargo/advisory artifacts are machine-readable and auditable.

### PR-2 completion summary

- Added runtime-backed hardening enforcement tooling:
  - `tools/run_security_attack_suite_v3.py`
- Added boot capture dependency for hardening evidence:
  - `tools/collect_booted_runtime_v1.py`
- `tools/run_security_fuzz_v2.py`
- Hardened vuln-response tooling contracts:
  - `tools/security_advisory_lint_v1.py`
  - `tools/security_embargo_drill_v1.py`
- Added executable PR-2 enforcement checks:
  - `tests/security/test_attack_suite_v3.py`
  - `tests/security/test_fuzz_gate_v2.py`
  - `tests/security/test_policy_enforcement_v3.py`
  - `tests/security/test_vuln_triage_sla_v1.py`
  - `tests/security/test_embargo_workflow_v1.py`
  - `tests/security/test_advisory_schema_v1.py`

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

### PR-3 completion summary

- Added aggregate gate tests:
  - `tests/security/test_security_hardening_gate_v3.py`
  - `tests/security/test_vuln_response_gate_v1.py`
- Added local gates:
  - `make test-security-hardening-v3`
  - `make test-vuln-response-v1`
  - JUnit outputs:
    - `out/pytest-security-hardening-v3.xml`
    - `out/pytest-vuln-response-v1.xml`
- Added CI gates + artifact uploads:
  - step: `Security hardening v3 gate`
  - artifact: `security-hardening-v3-artifacts`
  - step: `Vulnerability response v1 gate`
  - artifact: `vuln-response-v1-artifacts`
- Updated closure docs:
  - `MILESTONES.md`
  - `docs/STATUS.md`
  - `README.md`

## Non-goals for M28 backlog

- Unscoped expansion of security controls without contract/test ownership.
- Best-effort-only vulnerability handling outside defined SLA workflow.
