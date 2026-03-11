# M75 Execution Backlog (Trusted Update Orchestrator v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Add a trusted update orchestrator with explicit staged rollout, rollback, and
reboot coordination semantics for managed systems.

M75 source of truth remains:

- `docs/POST_G2_EXTENDED_MILESTONES.md`
- `docs/security/rights_capability_model_v1.md`
- `docs/M61_EXECUTION_BACKLOG.md`
- this backlog

## Current State Summary

- Release and update trust work exists, but there is no first-class
  orchestrator contract for staged system updates.
- Snapshot and storage rollback semantics in M61 provide the durability
  building blocks update orchestration will depend on.
- Unattended or policy-driven update behavior remains outside current
  release-gated claims.
- M75 must define those semantics before compliance and fleet admission work
  builds on them.

## Execution plan

- PR-1: update orchestrator contract freeze
- PR-2: rollout and rollback campaign baseline
- PR-3: update-orchestrator gate wiring + closure

## Execution status

- PR-1: pending
- PR-2: pending
- PR-3: pending

## PR-1: Update Orchestrator Contract Freeze

### Objective

Define staged rollout, rollback guardrails, and reboot coordination semantics
before implementation broadens managed-update claims.

### Scope

- Add docs:
  - `docs/security/update_orchestrator_policy_v1.md`
  - `docs/pkg/update_rollout_contract_v1.md`
  - `docs/runtime/reboot_coordinator_contract_v1.md`
- Add tests:
  - `tests/security/test_update_orchestrator_docs_v1.py`
  - `tests/pkg/test_update_rollout_docs_v1.py`

### Primary files

- `docs/security/update_orchestrator_policy_v1.md`
- `docs/pkg/update_rollout_contract_v1.md`
- `docs/runtime/reboot_coordinator_contract_v1.md`
- `tests/security/test_update_orchestrator_docs_v1.py`
- `tests/pkg/test_update_rollout_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/security/test_update_orchestrator_docs_v1.py tests/pkg/test_update_rollout_docs_v1.py -v`

### Done criteria for PR-1

- Update orchestration, rollout, and reboot semantics are explicit and
  versioned.
- Rollback guardrails and denied-policy paths are deterministic and reviewable.

## PR-2: Rollout and Rollback Campaign Baseline

### Objective

Implement deterministic evidence for staged rollout, rollback guardrails, and
reboot coordination behavior.

### Scope

- Add tooling:
  - `tools/run_update_orchestrator_v1.py`
  - `tools/run_rollback_guardrail_drill_v1.py`
- Add tests:
  - `tests/security/test_update_orchestrator_v1.py`
  - `tests/pkg/test_staged_rollout_policy_v1.py`
  - `tests/security/test_rollback_guardrails_v1.py`
  - `tests/security/test_update_orchestrator_negative_v1.py`

### Primary files

- `tools/run_update_orchestrator_v1.py`
- `tools/run_rollback_guardrail_drill_v1.py`
- `tests/security/test_update_orchestrator_v1.py`
- `tests/pkg/test_staged_rollout_policy_v1.py`
- `tests/security/test_rollback_guardrails_v1.py`
- `tests/security/test_update_orchestrator_negative_v1.py`

### Acceptance checks

- `python tools/run_update_orchestrator_v1.py --out out/update-orchestrator-v1.json`
- `python tools/run_rollback_guardrail_drill_v1.py --out out/update-rollback-v1.json`
- `python -m pytest tests/security/test_update_orchestrator_v1.py tests/pkg/test_staged_rollout_policy_v1.py tests/security/test_rollback_guardrails_v1.py tests/security/test_update_orchestrator_negative_v1.py -v`

### Done criteria for PR-2

- Update-orchestration artifacts are deterministic and machine-readable.
- `UPDATE: stage ok` and rollback-block markers remain stable.
- Managed update flows can reference explicit reboot and rollback policy IDs.

## PR-3: Security Hardening v4 Gate + Update Orchestrator Sub-gate

### Objective

Make trusted update behavior release-blocking for declared managed-system
profiles.

### Scope

- Add local gates:
  - `Makefile` target `test-security-hardening-v4`
  - `Makefile` target `test-update-orchestrator-v1`
- Add CI steps:
  - `Security hardening v4 gate`
  - `Update orchestrator v1 gate`
- Add aggregate tests:
  - `tests/security/test_security_hardening_gate_v4.py`
  - `tests/security/test_update_orchestrator_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/security/test_security_hardening_gate_v4.py`
- `tests/security/test_update_orchestrator_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-security-hardening-v4`
- `make test-update-orchestrator-v1`

### Done criteria for PR-3

- Update orchestration and hardening sub-gates are required in local and CI
  release lanes.
- M75 can be marked done only with deterministic staged-update and rollback
  evidence for declared managed-system profiles.

## Non-goals for M75 backlog

- formal compliance evidence mapping owned by M76
- fleet admission and attestation work owned by M77
- broad enterprise fleet-management control plane breadth
