# SimpleFS v2 - Journaling and Crash-Consistency Contract

Date: 2026-03-06  
Milestone: M18 Storage Reliability v2  
Status: active release gate

## Purpose

Define the storage correctness contract for v2 durability and recovery behavior.
This extends the v1 baseline with explicit journaling phases, bounded replay
rules, and deterministic power-fault outcomes.

## Baseline guarantees

- Mount safety:
  - superblock and metadata bounds are validated before mount success.
- Crash consistency:
  - commit units follow journaled write-ordering and barrier semantics.
- Recovery:
  - replay/check path classifies clean vs dirty journal state deterministically.
- Deterministic failure behavior:
  - malformed metadata and ordering violations fail with explicit evidence.

## On-disk v2 expectations

- Superblock keeps v1 core fields and must remain structurally valid.
- Journaled commit unit phases are explicit:
  - data write,
  - data durability barrier,
  - journal intent+descriptor append,
  - metadata commit,
  - journal commit marker,
  - checkpoint/clean transition.
- Recovery checks verify:
  - superblock magic and bounds,
  - data and metadata region limits,
  - journal ordering window consistency.

## Durability classes

- `volatile`
  - no post-crash survival guarantee.
- `fdatasync`
  - payload durability with metadata updates limited to reachability needs.
- `fsync`
  - full commit-unit durability for data and metadata.

Durability semantics are normatively defined in
`docs/storage/durability_model_v2.md`.

## Required tooling and tests

- Tooling:
  - `tools/storage_recover_v2.py`
  - `tools/run_storage_powerfail_campaign_v2.py`
- Tests:
  - `tests/storage/test_journal_recovery_v2.py`
  - `tests/storage/test_metadata_integrity_v2.py`
  - `tests/storage/test_powerfail_campaign_v2.py`
  - `tests/storage/test_storage_gate_v2.py`

## Required release gates

- Local: `make test-storage-reliability-v2`
- CI: `Storage reliability v2 gate`
- Artifacts:
  - `out/storage-recovery-v2.json`
  - `out/storage-powerfail-v2.json`
  - `out/pytest-storage-reliability-v2.xml`
