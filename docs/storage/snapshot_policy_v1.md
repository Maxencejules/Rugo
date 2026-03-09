# Snapshot Policy v1

Date: 2026-03-09  
Milestone: M38 Storage + Platform Feature Expansion v1  
Lane: Rugo (Rust kernel + Go user space)  
Status: active policy

## Purpose

Define deterministic snapshot semantics and thresholds that are release-blocking
for M38.

## Policy identity

- Policy identifier: `rugo.snapshot_policy.v1`.
- Parent storage feature contract: `rugo.storage_feature_contract.v1`.
- Feature campaign schema: `rugo.storage_feature_campaign_report.v1`.

## Required operations

- `snapshot_create`
- `snapshot_list`
- `snapshot_restore`
- `snapshot_prune`
- `snapshot_gc`

## Deterministic thresholds

- Snapshot create latency: `<= 80 ms`.
- Snapshot restore integrity ratio: `>= 1.0`.
- Snapshot retention policy violations: `0`.
- Snapshot orphaned metadata count: `0`.

## Required semantic rules

- Snapshot create must produce a stable metadata digest for identical seed and
  state.
- Snapshot restore must preserve deterministic file checksum integrity.
- Snapshot prune must not remove pinned snapshots.
- Snapshot garbage collection must leave zero orphan references.

## Gate wiring

- Campaign runner: `tools/run_storage_feature_campaign_v1.py`.
- Local gate: `make test-storage-platform-v1`.
- Local sub-gate: `make test-storage-feature-contract-v1`.
- CI gate: `Storage platform v1 gate`.
- CI sub-gate: `Storage feature contract v1 gate`.

## Failure handling

- Any threshold violation fails the storage platform gate.
- Any deterministic-integrity violation blocks release promotion.
- Snapshot support claims are bounded to this policy version.
