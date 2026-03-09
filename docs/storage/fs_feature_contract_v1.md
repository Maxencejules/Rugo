# Filesystem Feature Contract v1

Date: 2026-03-09  
Milestone: M38 Storage + Platform Feature Expansion v1  
Lane: Rugo (Rust kernel + Go user space)  
Status: active release gate

## Goal

Define bounded, deterministic semantics for advanced storage features introduced
in M38 while preserving M18 durability and recovery guarantees.

## Contract identity

- Storage feature contract ID: `rugo.storage_feature_contract.v1`.
- Snapshot policy ID: `rugo.snapshot_policy.v1`.
- Online resize policy ID: `rugo.online_resize_policy.v1`.
- Platform profile ID: `rugo.platform_feature_profile.v1`.
- Feature campaign schema: `rugo.storage_feature_campaign_report.v1`.
- Platform conformance schema: `rugo.platform_feature_conformance_report.v1`.

## Feature classes and required semantics

| Class | Required checks | Contract source | Gate behavior |
|---|---|---|---|
| Snapshot semantics | create, restore, retention, deterministic metadata integrity | `docs/storage/snapshot_policy_v1.md` | release-blocking |
| Online resize | grow path, shrink guard, post-resize fsck cleanliness | `docs/storage/online_resize_policy_v1.md` | release-blocking |
| Advanced filesystem ops | reflink/copy_file_range, fallocate, xattr roundtrip | this contract | release-blocking |
| Platform profile conformance | server/edge/dev profile feature coverage | `docs/runtime/platform_feature_profile_v1.md` | release-blocking |

## Required advanced fs operation surface

- `reflink`/clone-like copy path must be deterministic and regression-gated.
- `fallocate` preallocation path must preserve deterministic bounds.
- `copy_file_range` equivalent path must preserve deterministic completion
  status.
- xattr set/get roundtrip must preserve deterministic key/value integrity.

## Gate requirements

- Storage feature campaign command:
  - `python tools/run_storage_feature_campaign_v1.py --out out/storage-feature-v1.json`
- Platform feature conformance command:
  - `python tools/run_platform_feature_conformance_v1.py --out out/platform-feature-v1.json`
- Local gate: `make test-storage-platform-v1`.
- Local sub-gate: `make test-storage-feature-contract-v1`.
- CI gate: `Storage platform v1 gate`.
- CI sub-gate: `Storage feature contract v1 gate`.

Gate pass requires:

- feature campaign `gate_pass = true`
- platform conformance `gate_pass = true`
- no snapshot/resize/advanced-fs policy violations

## Artifact schema anchors

Required artifacts:

- `out/storage-feature-v1.json`
- `out/platform-feature-v1.json`
- `out/pytest-storage-platform-v1.xml`
- `out/pytest-storage-feature-contract-v1.xml`

## Claims boundary

- M38 does not claim broad storage feature parity beyond this contract.
- Features outside this contract remain non-claiming until explicitly versioned
  and gate-backed.
- Durability/recovery guarantees from storage reliability lanes are unchanged
  and remain required.
