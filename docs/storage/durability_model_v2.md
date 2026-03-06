# Durability Model v2

Date: 2026-03-06  
Milestone: M18 Storage Reliability v2

## Purpose

Specify v2 durability semantics for journaled commit units and deterministic
crash outcomes under power-fault scenarios.

## Durability classes

- `volatile`
  - write may be visible before crash and may be lost after crash.
- `fdatasync`
  - file payload blocks are durable after completion.
  - metadata durability is limited to required data reachability.
- `fsync`
  - data and metadata for the active commit unit are durable after completion.
  - required lookup metadata for parent directory paths is durable.

## Journaled commit-unit phases

1. data payload blocks written,
2. data durability barrier completed,
3. journal descriptor appended,
4. metadata updates written,
5. metadata durability barrier completed,
6. journal commit marker persisted,
7. checkpoint/clean marker updated.

## Crash-point model

Crash points are modeled at every phase boundary listed above.

Expected behavior:

- phases 1-3: pre-commit state remains mountable and consistent.
- phases 4-6: recovery replays or rolls back deterministically.
- phase 7: clean committed state mounts without replay.

## Deterministic error requirements

- Partial commit must be detected and classified as `dirty_journal`.
- Recovery outcome for identical image+seed inputs must be stable.
- Unsupported durability operations must fail explicitly, never silently
  succeed.

## Evidence

- `tests/storage/test_journal_recovery_v2.py`
- `tests/storage/test_metadata_integrity_v2.py`
- `tests/storage/test_powerfail_campaign_v2.py`
