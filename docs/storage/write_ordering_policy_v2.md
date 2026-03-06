# Write Ordering and Barrier Policy v2

Date: 2026-03-06  
Milestone: M18 Storage Reliability v2

## Purpose

Define required ordering between data, journal records, barriers, and metadata
so recovery outcomes are deterministic under power faults.

## Ordering rules

For a durability-checked commit unit:

1. Write data payload blocks.
2. Issue data durability barrier.
3. Append journal descriptor/intent record.
4. Write metadata updates (size, extent, allocation state).
5. Issue metadata durability barrier.
6. Persist journal commit marker.
7. Update checkpoint/clean marker.

## Forbidden ordering

- Metadata commit before data barrier.
- Journal commit marker before metadata durability barrier.
- Clean marker update before journal commit marker.
- Silent reordering that changes crash outcomes across identical inputs.

## Failure behavior

- Any ordering violation marks the unit as failed or dirty journal state.
- Recovery may replay or roll back, but the selected path and final state must
  be deterministic and reportable.

## Evidence

- `tests/storage/test_journal_recovery_v2.py`
- `tests/storage/test_powerfail_campaign_v2.py`
- `tests/storage/test_metadata_integrity_v2.py`
