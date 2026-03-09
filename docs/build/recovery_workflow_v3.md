# Recovery Workflow v3

Date: 2026-03-09  
Milestone: M30 Installer/Upgrade/Recovery UX v3  
Status: active release gate

## Purpose

Define deterministic recovery and rollback workflow behavior for day-to-day
operator use.

## Workflow identifiers

- Recovery workflow ID: `rugo.recovery_workflow.v3`
- Recovery drill schema: `rugo.recovery_drill.v3`
- Rollback safety schema: `rugo.rollback_safety_report.v3`
- Triage bundle schema: `rugo.postmortem_triage_bundle.v1`

## Deterministic recovery stages

1. `recovery_entry_validation`
2. `rollback_snapshot_mount`
3. `state_reconciliation`
4. `service_restore_validation`
5. `post_recovery_audit`

## Rollback safety requirements

- `rollback_floor_enforced` must be `true`.
- `unsigned_artifact_rejected` must be `true`.
- `rollback_path_verified` must be `true`.
- Maximum allowed recovery failures: `0`.

## Operator flow

1. `python tools/run_upgrade_drill_v3.py --out out/upgrade-drill-v3.json`
2. `python tools/run_recovery_drill_v3.py --out out/recovery-drill-v3.json`
3. `make test-ops-ux-v3`

## Required gates

- Local gate: `make test-ops-ux-v3`
- CI gate: `Ops UX v3 gate`
