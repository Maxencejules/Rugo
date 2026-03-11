# M84 Execution Backlog (Community Release Train + Support Channels v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Add an explicit public release-train and support-channel baseline so community
operations become predictable, auditable, and sustainable.

M84 source of truth remains:

- `docs/POST_G2_EXTENDED_MILESTONES.md`
- `docs/M31_EXECUTION_BACKLOG.md`
- `docs/M83_EXECUTION_BACKLOG.md`
- this backlog

## Current State Summary

- Release lifecycle, governance, and docs-quality work provide the foundations
  for public release operations.
- There is no versioned community release-train, support-channel SLA, or public
  advisory workflow contract in the post-M83 plan.
- Public release and support promises remain mostly implicit.
- M84 must define those semantics before broader community operations are
  treated as part of the platform contract.

## Execution plan

- PR-1: community release-train contract freeze
- PR-2: release calendar and support audit baseline
- PR-3: release-community gate wiring + closure

## Execution status

- PR-1: pending
- PR-2: pending
- PR-3: pending

## PR-1: Community Release-train Contract Freeze

### Objective

Define release calendar, support-channel, and public-advisory semantics before
implementation broadens public commitments.

### Scope

- Add docs:
  - `docs/build/community_release_train_v1.md`
  - `docs/community/support_channel_sla_v1.md`
  - `docs/security/public_advisory_workflow_v1.md`
- Add tests:
  - `tests/build/test_release_train_docs_v1.py`
  - `tests/security/test_public_advisory_docs_v1.py`

### Primary files

- `docs/build/community_release_train_v1.md`
- `docs/community/support_channel_sla_v1.md`
- `docs/security/public_advisory_workflow_v1.md`
- `tests/build/test_release_train_docs_v1.py`
- `tests/security/test_public_advisory_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/build/test_release_train_docs_v1.py tests/security/test_public_advisory_docs_v1.py -v`

### Done criteria for PR-1

- Release-train, support-channel, and public-advisory semantics are explicit
  and versioned.
- Escalation and SLA-exception paths remain deterministic and reviewable.

## PR-2: Release Calendar and Support Audit Baseline

### Objective

Implement deterministic evidence for release calendar, support-channel SLA, and
public advisory drills.

### Scope

- Add tooling:
  - `tools/run_release_calendar_audit_v1.py`
  - `tools/run_support_sla_audit_v1.py`
  - `tools/run_public_advisory_drill_v1.py`
- Add tests:
  - `tests/build/test_release_calendar_contract_v1.py`
  - `tests/docs/test_support_sla_docs_v1.py`
  - `tests/security/test_public_advisory_workflow_v1.py`
  - `tests/build/test_release_train_negative_v1.py`

### Primary files

- `tools/run_release_calendar_audit_v1.py`
- `tools/run_support_sla_audit_v1.py`
- `tools/run_public_advisory_drill_v1.py`
- `tests/build/test_release_calendar_contract_v1.py`
- `tests/docs/test_support_sla_docs_v1.py`
- `tests/security/test_public_advisory_workflow_v1.py`
- `tests/build/test_release_train_negative_v1.py`

### Acceptance checks

- `python tools/run_release_calendar_audit_v1.py --out out/release-calendar-v1.json`
- `python tools/run_support_sla_audit_v1.py --out out/support-sla-v1.json`
- `python tools/run_public_advisory_drill_v1.py --out out/public-advisory-v1.json`
- `python -m pytest tests/build/test_release_calendar_contract_v1.py tests/docs/test_support_sla_docs_v1.py tests/security/test_public_advisory_workflow_v1.py tests/build/test_release_train_negative_v1.py -v`

### Done criteria for PR-2

- Release-community artifacts are deterministic and machine-readable.
- Calendar, SLA, and advisory drill markers remain stable.
- Public commitments can be audited against explicit contract IDs.

## PR-3: Release Community Gate + Support SLA Sub-gate

### Objective

Make public release-train and support behavior release-blocking for community
operations.

### Scope

- Add local gates:
  - `Makefile` target `test-release-community-v1`
  - `Makefile` target `test-support-sla-v1`
- Add CI steps:
  - `Release community v1 gate`
  - `Support sla v1 gate`
- Add aggregate tests:
  - `tests/build/test_release_community_gate_v1.py`
  - `tests/docs/test_support_sla_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/build/test_release_community_gate_v1.py`
- `tests/docs/test_support_sla_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-release-community-v1`
- `make test-support-sla-v1`

### Done criteria for PR-3

- Release-community and support-SLA sub-gates are required in local and CI
  release lanes.
- M84 can be marked done only with deterministic release calendar, support, and
  advisory evidence for public operations.

## Non-goals for M84 backlog

- replacing technical release gates with community process alone
- indefinite support commitments outside declared SLA policy
- broad ecosystem or fleet scope beyond the public release/support baseline
