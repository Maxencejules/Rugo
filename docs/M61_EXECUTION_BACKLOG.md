# M61 Execution Backlog (CoW Snapshots + Integrity Repair v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Add bounded copy-on-write snapshots and integrity-repair semantics so Rugo can
offer rollback and scrub-based repair without claiming unlimited filesystem
parity.

M61 source of truth remains:

- `docs/POST_G2_EXTENDED_MILESTONES.md`
- `docs/M58_EXECUTION_BACKLOG.md`
- `docs/M60_EXECUTION_BACKLOG.md`
- this backlog

## Current State Summary

- Journaling, encryption, and RAID baselines define the durability and device
  model required for snapshot and repair work.
- Existing snapshot-style features from earlier expansion work are not yet a
  declared CoW and integrity-repair contract.
- There is no versioned snapshot-management ABI or integrity scrub policy in
  the post-M58 storage plan.
- M61 must define those semantics before security, update, and rollback flows
  depend on them.

## Execution plan

- PR-1: CoW and integrity contract freeze
- PR-2: snapshot and repair campaign baseline
- PR-3: integrity gate wiring + closure

## Execution status

- PR-1: pending
- PR-2: pending
- PR-3: pending

## PR-1: CoW and Integrity Contract Freeze

### Objective

Define snapshot, subvolume, scrub, and repair semantics before implementation
lands.

### Scope

- Add docs:
  - `docs/storage/cow_snapshot_contract_v1.md`
  - `docs/storage/integrity_scrub_policy_v1.md`
  - `docs/abi/snapshot_management_contract_v1.md`
- Add tests:
  - `tests/storage/test_cow_snapshot_docs_v1.py`

### Primary files

- `docs/storage/cow_snapshot_contract_v1.md`
- `docs/storage/integrity_scrub_policy_v1.md`
- `docs/abi/snapshot_management_contract_v1.md`
- `tests/storage/test_cow_snapshot_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/storage/test_cow_snapshot_docs_v1.py -v`

### Done criteria for PR-1

- Snapshot and integrity-repair behavior are explicit and versioned.
- Rollback, send/receive, and self-heal boundaries are reviewable before
  broader rollback claims appear elsewhere.

## PR-2: Snapshot and Repair Campaign Baseline

### Objective

Implement deterministic evidence for snapshot creation, scrub, repair, and
send/receive semantics.

### Scope

- Add tooling:
  - `tools/run_snapshot_integrity_campaign_v1.py`
  - `tools/run_scrub_repair_v1.py`
- Add tests:
  - `tests/storage/test_cow_snapshot_semantics_v1.py`
  - `tests/storage/test_scrub_selfheal_v1.py`
  - `tests/storage/test_snapshot_send_receive_v1.py`
  - `tests/storage/test_snapshot_negative_v1.py`

### Primary files

- `tools/run_snapshot_integrity_campaign_v1.py`
- `tools/run_scrub_repair_v1.py`
- `tests/storage/test_cow_snapshot_semantics_v1.py`
- `tests/storage/test_scrub_selfheal_v1.py`
- `tests/storage/test_snapshot_send_receive_v1.py`
- `tests/storage/test_snapshot_negative_v1.py`

### Acceptance checks

- `python tools/run_snapshot_integrity_campaign_v1.py --out out/snapshot-integrity-v1.json`
- `python tools/run_scrub_repair_v1.py --out out/scrub-repair-v1.json`
- `python -m pytest tests/storage/test_cow_snapshot_semantics_v1.py tests/storage/test_scrub_selfheal_v1.py tests/storage/test_snapshot_send_receive_v1.py tests/storage/test_snapshot_negative_v1.py -v`

### Done criteria for PR-2

- Snapshot and repair artifacts are deterministic and machine-readable.
- `SNAP: create ok` and `SCRUB: repaired` markers are stable and auditable.
- Rollback-oriented consumers can reference explicit snapshot contract IDs.

## PR-3: Storage Integrity Gate + Snapshot Management Sub-gate

### Objective

Make CoW and integrity-repair semantics release-blocking for declared storage
profiles.

### Scope

- Add local gates:
  - `Makefile` target `test-storage-integrity-v1`
  - `Makefile` target `test-snapshot-management-v1`
- Add CI steps:
  - `Storage integrity v1 gate`
  - `Snapshot management v1 gate`
- Add aggregate tests:
  - `tests/storage/test_storage_integrity_gate_v1.py`
  - `tests/storage/test_snapshot_management_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/storage/test_storage_integrity_gate_v1.py`
- `tests/storage/test_snapshot_management_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-storage-integrity-v1`
- `make test-snapshot-management-v1`

### Done criteria for PR-3

- Integrity and snapshot-management sub-gates are required in local and CI
  release lanes.
- M61 can be marked done only with deterministic rollback, scrub, and repair
  evidence linked to the declared contract surfaces.

## Non-goals for M61 backlog

- unlimited filesystem feature parity with mature CoW filesystems
- hidden rollback behavior that bypasses update or security policy
- distributed replication beyond the declared snapshot-management scope
