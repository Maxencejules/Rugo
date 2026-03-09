# Platform Feature Profile v1

Date: 2026-03-09  
Milestone: M38 Storage + Platform Feature Expansion v1  
Lane: Rugo (Rust kernel + Go user space)  
Status: active release gate

## Objective

Define deterministic platform profile requirements for advanced storage feature
availability in M38.

## Contract identifiers

- Platform conformance policy ID: `rugo.platform_feature_profile.v1`
- Platform conformance report schema: `rugo.platform_feature_conformance_report.v1`
- Profile requirement schema: `rugo.platform_feature_requirement_set.v1`
- Parent storage feature contract ID: `rugo.storage_feature_contract.v1`
- Feature campaign schema: `rugo.storage_feature_campaign_report.v1`

## Profile set

- `server_storage_dense_v1`
- `edge_resilient_v1`
- `dev_workstation_v1`

## Required profile checks

- `server_storage_dense_v1`
  - `snapshot_create_ms` must be `<= 90`.
  - `online_resize_grow_ms` must be `<= 130`.
  - `reflink_success_ratio` must be `>= 1.0`.
- `edge_resilient_v1`
  - `snapshot_restore_integrity_ratio` must be `>= 1.0`.
  - `resize_shrink_guard_ratio` must be `>= 1.0`.
  - `post_resize_fsck_errors` must be `<= 0`.
- `dev_workstation_v1`
  - `xattr_roundtrip_ms` must be `<= 10`.
  - `copy_file_range_ms` must be `<= 16`.
  - `fallocate_ms` must be `<= 15`.

## Tooling and gate wiring

- Platform conformance tool: `tools/run_platform_feature_conformance_v1.py`
- Storage campaign tool: `tools/run_storage_feature_campaign_v1.py`
- Local gate: `make test-storage-platform-v1`
- Local sub-gate: `make test-storage-feature-contract-v1`
- CI gate: `Storage platform v1 gate`
- CI sub-gate: `Storage feature contract v1 gate`
- Release artifact: `out/platform-feature-v1.json`

## Policy notes

- Profile conformance is release-blocking for declared M38 profiles only.
- Additional platform profiles require explicit contract/version updates before
  they can be claimed.
