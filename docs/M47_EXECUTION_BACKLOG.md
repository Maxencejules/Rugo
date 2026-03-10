# M47 Execution Backlog (Hardware Claim Promotion Program v1)

Date: 2026-03-10  
Lane: Rugo (Rust kernel + Go user space)  
Status: proposed

## Goal

Make hardware support claims auditable by policy, promotion history, and tier
assignment rather than inferred from ad hoc evidence or test presence.

M47 source of truth remains:

- `docs/M45_M47_HARDWARE_EXPANSION_ROADMAP.md`
- `docs/hw/support_matrix_v6_plan.md`
- this backlog

## Current State Summary

- The project already has strong matrix evidence discipline, but support-tier
  promotion remains lighter than the newer release/security/evidence programs.
- As hardware breadth grows, the project needs a formal support-claim program,
  not just more tests.
- Hardware classes introduced in M45 and M46 should remain non-claiming until a
  promotion/audit milestone closes.

## Execution Plan

- PR-1: contract freeze
- PR-2: promotion tooling + audits
- PR-3: release-gate wiring + support-tier enforcement

## PR-1: Support Claim Policy Freeze

### Objective

Define support-tier, promotion, and audit policy for claimable hardware
classes.

### Scope

- Add docs:
  - `docs/hw/support_claim_policy_v1.md`
  - `docs/hw/bare_metal_promotion_policy_v2.md`
  - `docs/hw/support_tier_audit_v1.md`
- Add tests:
  - `tests/hw/test_support_claim_docs_v1.py`

### Primary files

- `docs/hw/support_claim_policy_v1.md`
- `docs/hw/bare_metal_promotion_policy_v2.md`
- `docs/hw/support_tier_audit_v1.md`
- `tests/hw/test_support_claim_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/hw/test_support_claim_docs_v1.py -v`

### Done criteria for PR-1

- Claimable support tiers and promotion obligations are explicit and versioned.
- Any change to support status can be audited against declared policy.

## PR-2: Promotion and Support-Tier Audits

### Objective

Implement deterministic promotion and audit tooling for hardware support tiers.

### Scope

- Add tooling:
  - `tools/run_hw_claim_promotion_v1.py`
  - `tools/run_hw_support_tier_audit_v1.py`
- Add tests:
  - `tests/hw/test_hw_claim_promotion_v1.py`
  - `tests/hw/test_hw_support_tier_audit_v1.py`
  - `tests/hw/test_hw_promotion_regression_v1.py`
  - `tests/hw/test_hw_support_claim_negative_v1.py`

### Primary files

- `tools/run_hw_claim_promotion_v1.py`
- `tools/run_hw_support_tier_audit_v1.py`
- `tests/hw/test_hw_claim_promotion_v1.py`
- `tests/hw/test_hw_support_tier_audit_v1.py`
- `tests/hw/test_hw_promotion_regression_v1.py`
- `tests/hw/test_hw_support_claim_negative_v1.py`

### Acceptance checks

- `python tools/run_hw_claim_promotion_v1.py --out out/hw-claim-promotion-v1.json`
- `python tools/run_hw_support_tier_audit_v1.py --out out/hw-support-tier-audit-v1.json`
- `python -m pytest tests/hw/test_hw_claim_promotion_v1.py tests/hw/test_hw_support_tier_audit_v1.py tests/hw/test_hw_promotion_regression_v1.py tests/hw/test_hw_support_claim_negative_v1.py -v`

### Done criteria for PR-2

- Promotion and tier-audit artifacts are deterministic and machine-readable.
- Regressions, unsupported promotions, and undocumented tier drift are rejected.

## PR-3: Claim Promotion Gate + Tier Audit Sub-gate

### Objective

Make promotion policy enforceable in release lanes and tie claimed hardware
changes to auditable gate results.

### Scope

- Add local gates:
  - `Makefile` target `test-hw-claim-promotion-v1`
  - `Makefile` target `test-hw-support-tier-audit-v1`
- Add CI steps:
  - `Hardware claim promotion v1 gate`
  - `Hardware support tier audit v1 gate`
- Add aggregate tests:
  - `tests/hw/test_hw_claim_promotion_gate_v1.py`
  - `tests/hw/test_hw_support_tier_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/hw/test_hw_claim_promotion_gate_v1.py`
- `tests/hw/test_hw_support_tier_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-hw-claim-promotion-v1`
- `make test-hw-support-tier-audit-v1`

### Done criteria for PR-3

- Support-tier changes are blocked unless promotion and audit gates pass.
- Claimed classes are traceable to explicit promotion history and policy IDs.
- Unsupported classes remain explicit and machine-auditable.

## Non-goals for M47 backlog

- implementing new hardware classes unrelated to support-tier policy
- broadening claims beyond what M45 and M46 evidence justify
- replacing deterministic unsupported behavior with vague “experimental” claims

