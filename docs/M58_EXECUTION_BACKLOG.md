# M58 Execution Backlog (Journaling Filesystem Baseline v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Introduce a journaled filesystem baseline with explicit replay, checksum, and
quota semantics that strengthen crash consistency without weakening
reproducibility.

M58 source of truth remains:

- `docs/POST_G2_EXTENDED_MILESTONES.md`
- `docs/storage/fs_v1.md`
- `docs/M54_EXECUTION_BACKLOG.md`
- this backlog

## Current State Summary

- Storage reliability and feature expansion exist, but journaling remains out
  of scope.
- Native storage work in M54 introduces new device classes that need stronger
  crash-consistency semantics than a purely virtio-shaped baseline.
- There is no dedicated contract yet for journal layout, replay, or quota
  hooks.
- M58 must freeze those semantics before encryption, RAID, or CoW milestones
  build on them.

## Execution plan

- PR-1: journaling and quota contract freeze
- PR-2: replay and power-fail campaign baseline
- PR-3: journaling gate wiring + closure

## Execution status

- PR-1: pending
- PR-2: pending
- PR-3: pending

## PR-1: Journaling and Quota Contract Freeze

### Objective

Define journal transaction, replay, checksum, and quota semantics before
implementation broadens storage guarantees.

### Scope

- Add docs:
  - `docs/storage/fs_journal_contract_v1.md`
  - `docs/storage/journal_replay_policy_v1.md`
  - `docs/abi/fs_quota_contract_v1.md`
- Add tests:
  - `tests/storage/test_fs_journal_docs_v1.py`
  - `tests/storage/test_quota_contract_v1.py`

### Primary files

- `docs/storage/fs_journal_contract_v1.md`
- `docs/storage/journal_replay_policy_v1.md`
- `docs/abi/fs_quota_contract_v1.md`
- `tests/storage/test_fs_journal_docs_v1.py`
- `tests/storage/test_quota_contract_v1.py`

### Acceptance checks

- `python -m pytest tests/storage/test_fs_journal_docs_v1.py tests/storage/test_quota_contract_v1.py -v`

### Done criteria for PR-1

- Journaled durability and quota behavior are explicit and versioned.
- Power-loss and partial-write behavior is described as deterministic policy.

## PR-2: Replay and Power-fail Campaign Baseline

### Objective

Implement deterministic replay and fault-injection evidence for the journaled
filesystem baseline.

### Scope

- Add tooling:
  - `tools/storage_journal_replay_v1.py`
  - `tools/run_journal_fault_campaign_v1.py`
- Add tests:
  - `tests/storage/test_journal_replay_v1.py`
  - `tests/storage/test_journal_checksum_v1.py`
  - `tests/storage/test_quota_contract_v1.py`
  - `tests/storage/test_powerfail_matrix_v1.py`

### Primary files

- `tools/storage_journal_replay_v1.py`
- `tools/run_journal_fault_campaign_v1.py`
- `tests/storage/test_journal_replay_v1.py`
- `tests/storage/test_journal_checksum_v1.py`
- `tests/storage/test_quota_contract_v1.py`
- `tests/storage/test_powerfail_matrix_v1.py`

### Acceptance checks

- `python tools/storage_journal_replay_v1.py --check --out out/storage-journal-replay-v1.json`
- `python tools/run_journal_fault_campaign_v1.py --seed 20260311 --out out/journal-fault-campaign-v1.json`
- `python -m pytest tests/storage/test_journal_replay_v1.py tests/storage/test_journal_checksum_v1.py tests/storage/test_quota_contract_v1.py tests/storage/test_powerfail_matrix_v1.py -v`

### Done criteria for PR-2

- Journal replay and fault-campaign artifacts are deterministic and
  machine-readable.
- `FSJ: commit ok` and `FSJ: replay ok` style markers remain stable under
  seeded power-fail campaigns.

## PR-3: Storage Reliability v3 Gate + Journal Sub-gate

### Objective

Make the journaled baseline release-blocking for declared storage profiles.

### Scope

- Add local gates:
  - `Makefile` target `test-storage-reliability-v3`
  - `Makefile` target `test-fs-journal-contract-v1`
- Add CI steps:
  - `Storage reliability v3 gate`
  - `FS journal contract v1 gate`
- Add aggregate tests:
  - `tests/storage/test_storage_reliability_gate_v3.py`
  - `tests/storage/test_fs_journal_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/storage/test_storage_reliability_gate_v3.py`
- `tests/storage/test_fs_journal_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-storage-reliability-v3`
- `make test-fs-journal-contract-v1`

### Done criteria for PR-3

- Journaling and journal-contract sub-gates are required in local and CI lanes.
- M58 can be marked done only with release-gated replay evidence and
  deterministic quota policy outcomes.

## Non-goals for M58 backlog

- full encryption/key management owned by M59
- multi-device RAID behavior owned by M60
- CoW snapshot and integrity repair owned by M61
