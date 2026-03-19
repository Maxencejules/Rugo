# M47 Execution Backlog (Hardware Claim Promotion Program v1)

Date: 2026-03-10  
Lane: Rugo (Rust kernel + Go user space)  
Status: done

## Goal

Make hardware support claims auditable by policy, promotion history, and tier
assignment rather than inferred from ad hoc evidence or test presence.

M47 source of truth remains:

- `docs/M45_M47_HARDWARE_EXPANSION_ROADMAP.md`
- `docs/hw/support_matrix_v6_plan.md`
- this backlog

## Current State Summary

- Support-tier claims are now explicit via a dedicated claim policy,
  bare-metal promotion policy revision, and support-tier audit contract.
- Deterministic claim-promotion and tier-audit artifacts now cover the modern
  VirtIO and bare-metal I/O classes introduced in M45 and M46.
- Local and CI release lanes now reject undocumented tier drift, missing
  promotion history, and unsupported claim broadening.

## Execution Result

- PR-1: complete (2026-03-10)
- PR-2: complete (2026-03-10)
- PR-3: complete (2026-03-10)

## Historical Rugo implementation summary

### Historical Rust kernel surface

- M47 was mostly policy and gate wiring, not a large new runtime mechanism.
- Rust-side impact was indirect: support-tier claims were tied back to the
  implemented hardware paths in `kernel_rs/src/`, `arch/`, and `boot/`.

### Historical Go user space surface

- `services/go/`: minimal direct ownership. This milestone audited and bounded
  existing support claims more than it added new userspace behavior.
- `services/go_std/`: not the primary path for this milestone.

### Historical Language-Native Verification

- `make kernel`
- `make userspace`
- `make image-demo`
- `make smoke-demo`
- `make test-hw-claim-promotion-v1`
- `make test-hw-support-tier-audit-v1`

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

### PR-1 completion summary

- Added claim and audit policy docs:
  - `docs/hw/support_claim_policy_v1.md`
  - `docs/hw/bare_metal_promotion_policy_v2.md`
  - `docs/hw/support_tier_audit_v1.md`
- Added executable contract checks:
  - `tests/hw/test_support_claim_docs_v1.py`

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

### PR-2 completion summary

- Added deterministic claim-promotion and audit tooling:
  - `tools/run_hw_claim_promotion_v1.py`
  - `tools/run_hw_support_tier_audit_v1.py`
- Added executable claim and negative-path checks:
  - `tests/hw/test_hw_claim_promotion_v1.py`
  - `tests/hw/test_hw_support_tier_audit_v1.py`
  - `tests/hw/test_hw_promotion_regression_v1.py`
  - `tests/hw/test_hw_support_claim_negative_v1.py`

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

### PR-3 completion summary

- Added aggregate gate checks:
  - `tests/hw/test_hw_claim_promotion_gate_v1.py`
  - `tests/hw/test_hw_support_tier_gate_v1.py`
- Added local gates:
  - `make test-hw-claim-promotion-v1`
  - `make test-hw-support-tier-audit-v1`
- Added CI qualification gates and artifacts:
  - `Hardware claim promotion v1 gate`
  - `Hardware support tier audit v1 gate`
- Updated closure docs:
  - `MILESTONES.md`
  - `docs/STATUS.md`
  - `README.md`

## X2 runtime-backed closure addendum (2026-03-18)

- This backlog is now governed by the shared x2 hardware runtime qualification lane.
- Claim promotion and support-tier audit now consume the shared x2 hardware
  runtime qualification evidence rather than auditing a separate claim-only
  lane.
- `make test-x2-hardware-runtime-v1` keeps promoted classes tied to explicit
  target qualification, device registry, and bind evidence.
- Future claim rows must remain subordinate to the same runtime-backed target
  evidence instead of policy-only promotion.

## Non-goals for M47 backlog

- implementing new hardware classes unrelated to support-tier policy
- broadening claims beyond what M45 and M46 evidence justify
- replacing deterministic unsupported behavior with vague “experimental” claims
