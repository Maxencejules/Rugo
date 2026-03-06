# M18 Execution Backlog (Storage Reliability v2)

Date: 2026-03-06  
Lane: Rugo (Rust kernel + Go user space)  
Status: done

## Goal

Raise storage durability from v1 baseline to stronger crash consistency and
power-fault resilience for multi-purpose operation.

M18 source of truth remains `docs/M15_M20_MULTIPURPOSE_PLAN.md`,
`MILESTONES.md`, and this backlog.

## Current State Summary

- Storage reliability v1 milestone is complete and gate-wired.
- Recovery and fault-campaign tooling patterns are already established.
- M18 closes v2 contract and reliability thresholds.

## Execution Result

- PR-1: complete (2026-03-06)
- PR-2: complete (2026-03-06)
- PR-3: complete (2026-03-06)

## PR-1: Storage Contract + Journaling Policy v2

### Objective

Freeze v2 durability and write-ordering semantics before campaign updates.

### Scope

- Add docs:
  - `docs/storage/fs_v2.md`
  - `docs/storage/durability_model_v2.md`
  - `docs/storage/write_ordering_policy_v2.md`
- Add tests:
  - `tests/storage/test_journal_recovery_v2.py`
  - `tests/storage/test_metadata_integrity_v2.py`

### Primary files

- `docs/storage/fs_v2.md`
- `docs/storage/durability_model_v2.md`
- `docs/storage/write_ordering_policy_v2.md`
- `tests/storage/test_journal_recovery_v2.py`
- `tests/storage/test_metadata_integrity_v2.py`

### Acceptance checks

- `python -m pytest tests/storage/test_journal_recovery_v2.py tests/storage/test_metadata_integrity_v2.py -v`

### Done criteria for PR-1

- Storage v2 contracts are explicit, versioned, and test-referenced.
- Crash-consistency guarantees have no unowned placeholders.

### PR-1 completion summary

- Added storage v2 contract docs for:
  - journaling phases,
  - durability classes,
  - deterministic recovery/failure semantics.
- Added executable checks for:
  - journal recovery report schema + mountability invariants,
  - metadata corruption and bounds-integrity detection,
  - v2 doc contract tokens.

## PR-2: Recovery + Power-Fail Campaign v2

### Objective

Validate deterministic recovery behavior under power-fault scenarios.

### Scope

- Add tooling:
  - `tools/storage_recover_v2.py`
  - `tools/run_storage_powerfail_campaign_v2.py`
- Add tests:
  - `tests/storage/test_powerfail_campaign_v2.py`
- Add docs:
  - `docs/storage/recovery_playbook_v2.md`
  - `docs/storage/fault_campaign_v2.md`

### Primary files

- `tools/storage_recover_v2.py`
- `tools/run_storage_powerfail_campaign_v2.py`
- `tests/storage/test_powerfail_campaign_v2.py`
- `docs/storage/recovery_playbook_v2.md`
- `docs/storage/fault_campaign_v2.md`

### Acceptance checks

- `python tools/storage_recover_v2.py --check --out out/storage-recovery-v2.json`
- `python tools/run_storage_powerfail_campaign_v2.py --seed 20260304 --out out/storage-powerfail-v2.json`
- `python -m pytest tests/storage/test_powerfail_campaign_v2.py -v`

### Done criteria for PR-2

- Recovery and power-fail artifacts are machine-readable and deterministic.
- Campaign thresholds are explicit and regression-trackable.

### PR-2 completion summary

- Added deterministic tooling:
  - `tools/storage_recover_v2.py`
  - `tools/run_storage_powerfail_campaign_v2.py`
- Added deterministic artifact schemas:
  - `rugo.storage_recovery_report.v2`
  - `rugo.storage_powerfail_campaign_report.v2`
- Added campaign and playbook docs:
  - `docs/storage/recovery_playbook_v2.md`
  - `docs/storage/fault_campaign_v2.md`

## PR-3: Storage v2 Gate + Milestone Closure

### Objective

Promote storage reliability v2 checks to release-blocking status.

### Scope

- Add aggregate test:
  - `tests/storage/test_storage_gate_v2.py`
- Add local gate:
  - `Makefile` target `test-storage-reliability-v2`
- Add CI gate:
  - `.github/workflows/ci.yml` step `Storage reliability v2 gate`

### Primary files

- `tests/storage/test_storage_gate_v2.py`
- `Makefile`
- `.github/workflows/ci.yml`
- `MILESTONES.md`
- `docs/STATUS.md`

### Acceptance checks

- `make test-storage-reliability-v2`

### Done criteria for PR-3

- Storage v2 gate is required in local and CI release lanes.
- M18 status can be marked done with linked evidence.

### PR-3 completion summary

- Added aggregate gate test:
  - `tests/storage/test_storage_gate_v2.py`
- Added local gate:
  - `make test-storage-reliability-v2`
  - JUnit output: `out/pytest-storage-reliability-v2.xml`
- Added CI gate + artifact upload:
  - step: `Storage reliability v2 gate`
  - artifact: `storage-reliability-v2-artifacts`
- Updated milestone/status documents to mark M18 done with evidence links.

## Non-goals for M18 backlog

- Full snapshot/online-resize filesystem feature parity.
- Broad storage hardware expansion beyond scoped matrix targets.
