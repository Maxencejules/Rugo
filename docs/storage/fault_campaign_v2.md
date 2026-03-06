# Storage Power-Fail Campaign v2

Date: 2026-03-06  
Milestone: M18 Storage Reliability v2

## Purpose

Define deterministic power-fault campaign dimensions and thresholds used by the
M18 release gate.

## Campaign dimensions

- Fault classes:
  - loss before data barrier,
  - loss after data barrier,
  - loss before metadata barrier,
  - loss before checkpoint,
  - superblock partial write,
  - journal-tail torn write.
- Workload emphasis:
  - metadata-heavy updates,
  - many-small-file churn,
  - overwrite-heavy rewrite loops.
- Verification:
  - post-recovery mountability,
  - bounded failure thresholds,
  - machine-readable deterministic report output.

## Required report fields

- `schema: rugo.storage_powerfail_campaign_report.v2`
- `seed`
- `iterations`
- per-fault counters
- `injected_faults`
- `recovered_cases`
- `total_failures`
- `max_failures`
- `meets_target`

## Gate thresholds (v2)

- fixed-seed deterministic campaign in CI.
- default `max_failures = 0`.
- threshold changes require explicit milestone document update.

## Evidence

- `tools/run_storage_powerfail_campaign_v2.py`
- `tests/storage/test_powerfail_campaign_v2.py`
