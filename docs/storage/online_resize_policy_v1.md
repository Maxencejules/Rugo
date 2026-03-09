# Online Resize Policy v1

Date: 2026-03-09  
Milestone: M38 Storage + Platform Feature Expansion v1  
Lane: Rugo (Rust kernel + Go user space)  
Status: active policy

## Purpose

Define deterministic online filesystem resize expectations for grow/shrink
controls under M38.

## Policy identity

- Policy identifier: `rugo.online_resize_policy.v1`.
- Parent storage feature contract: `rugo.storage_feature_contract.v1`.
- Feature campaign schema: `rugo.storage_feature_campaign_report.v1`.

## Required operations

- `online_resize_grow`
- `online_resize_shrink_guard`
- `post_resize_fsck`
- `capacity_reconcile`

## Deterministic thresholds

- Online grow completion latency: `<= 120 ms`.
- Capacity reconcile mismatch count: `0`.
- Shrink guard enforcement ratio: `>= 1.0`.
- Post-resize fsck error count: `0`.

## Required semantic rules

- Grow path must preserve metadata integrity and deterministic completion state.
- Unsafe shrink requests must be rejected deterministically.
- Safe shrink eligibility remains policy-bounded and never implicit.
- Post-resize fsck must report zero structural errors.

## Gate wiring

- Campaign runner: `tools/run_storage_feature_campaign_v1.py`.
- Local gate: `make test-storage-platform-v1`.
- Local sub-gate: `make test-storage-feature-contract-v1`.
- CI gate: `Storage platform v1 gate`.
- CI sub-gate: `Storage feature contract v1 gate`.

## Failure handling

- Any resize threshold violation fails release gates.
- Any nondeterministic shrink behavior is a release blocker.
- Resize support claims are bounded to this policy version.
