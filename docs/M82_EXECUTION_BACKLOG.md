# M82 Execution Backlog (Contributor Portal + Governance v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Establish explicit contributor and governance workflows so roadmap execution can
scale beyond a small maintainer group.

M82 source of truth remains:

- `docs/POST_G2_EXTENDED_MILESTONES.md`
- `docs/M31_EXECUTION_BACKLOG.md`
- `docs/M74_EXECUTION_BACKLOG.md`
- this backlog

## Current State Summary

- Technical milestone and release policy work exists, but community process
  remains mostly implicit.
- Ecosystem federation in M74 increases the need for explicit contributor and
  governance rules.
- There is no versioned contribution policy, RFC process, or conduct-enforcement
  contract in-tree.
- M82 must define those semantics before localization and public support-channel
  work depends on them.

## Execution plan

- PR-1: contributor and governance contract freeze
- PR-2: portal and governance validation baseline
- PR-3: governance gate wiring + closure

## Execution status

- PR-1: pending
- PR-2: pending
- PR-3: pending

## PR-1: Contributor and Governance Contract Freeze

### Objective

Define contributor policy, RFC flow, and conduct-enforcement semantics before
community operations broaden.

### Scope

- Add docs:
  - `docs/community/contribution_policy_v1.md`
  - `docs/community/rfc_process_v1.md`
  - `docs/community/code_of_conduct_enforcement_v1.md`
- Add tests:
  - `tests/docs/test_contribution_docs_v1.py`
  - `tests/docs/test_rfc_process_v1.py`

### Primary files

- `docs/community/contribution_policy_v1.md`
- `docs/community/rfc_process_v1.md`
- `docs/community/code_of_conduct_enforcement_v1.md`
- `tests/docs/test_contribution_docs_v1.py`
- `tests/docs/test_rfc_process_v1.py`

### Acceptance checks

- `python -m pytest tests/docs/test_contribution_docs_v1.py tests/docs/test_rfc_process_v1.py -v`

### Done criteria for PR-1

- Contribution, RFC, and conduct-enforcement semantics are explicit and
  versioned.
- Governance exceptions and escalation paths are reviewable before broader
  community operations land.

## PR-2: Portal and Governance Validation Baseline

### Objective

Implement deterministic validation for contributor portal and governance index
behavior.

### Scope

- Add tooling:
  - `tools/validate_contributor_portal_v1.py`
  - `tools/validate_governance_index_v1.py`
- Add tests:
  - `tests/docs/test_contributor_portal_v1.py`
  - `tests/docs/test_governance_index_v1.py`

### Primary files

- `tools/validate_contributor_portal_v1.py`
- `tools/validate_governance_index_v1.py`
- `tests/docs/test_contributor_portal_v1.py`
- `tests/docs/test_governance_index_v1.py`

### Acceptance checks

- `python tools/validate_contributor_portal_v1.py --out out/contributor-portal-v1.json`
- `python tools/validate_governance_index_v1.py --out out/governance-index-v1.json`
- `python -m pytest tests/docs/test_contributor_portal_v1.py tests/docs/test_governance_index_v1.py -v`

### Done criteria for PR-2

- Contributor-portal artifacts are deterministic and machine-readable.
- Governance and contributor paths can be validated without manual-only review.

## PR-3: Docs Governance Gate + Contributor Portal Sub-gate

### Objective

Make contributor and governance behavior release-blocking for public
contribution flows.

### Scope

- Add local gates:
  - `Makefile` target `test-docs-governance-v1`
  - `Makefile` target `test-contributor-portal-v1`
- Add CI steps:
  - `Docs governance v1 gate`
  - `Contributor portal v1 gate`
- Add aggregate tests:
  - `tests/docs/test_docs_governance_gate_v1.py`
  - `tests/docs/test_contributor_portal_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/docs/test_docs_governance_gate_v1.py`
- `tests/docs/test_contributor_portal_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-docs-governance-v1`
- `make test-contributor-portal-v1`

### Done criteria for PR-3

- Governance and contributor-portal sub-gates are required in local and CI
  release lanes.
- M82 can be marked done only with deterministic policy and portal evidence for
  public contribution flows.

## Non-goals for M82 backlog

- localization and docs-quality work owned by M83
- public support-channel operations owned by M84
- replacing technical review with governance process alone
