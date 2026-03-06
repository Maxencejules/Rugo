# Storage Recovery Playbook v2

Date: 2026-03-06  
Milestone: M18 Storage Reliability v2

## Goal

Provide a deterministic workflow for storage integrity checks, journal recovery
classification, and release-gate evidence collection.

## Recovery workflow

1. Run recovery checker:
   - `python tools/storage_recover_v2.py --check --out out/storage-recovery-v2.json`
2. Validate required fields:
   - `schema == rugo.storage_recovery_report.v2`
   - `mountable == true`
   - `total_issues == 0`
3. If check fails:
   - inspect failed checks and superblock bounds,
   - classify `journal_state` and `recovery_action`,
   - block release until deterministic repro and fix are recorded.

## Power-fault workflow

1. Run campaign tool:
   - `python tools/run_storage_powerfail_campaign_v2.py --seed 20260304 --out out/storage-powerfail-v2.json`
2. Validate thresholds:
   - `schema == rugo.storage_powerfail_campaign_report.v2`
   - `total_failures <= max_failures`
3. Preserve report artifacts for CI upload and incident triage.

## Escalation policy

- Recovery-check regressions are release-blocking for M18.
- Power-fail threshold breaches are release-blocking for M18.
- Fixes require updated negative-path tests and refreshed artifacts.
