# Installer UX v3 Contract

Date: 2026-03-09  
Milestone: M30 Installer/Upgrade/Recovery UX v3  
Status: active release gate

## Purpose

Define deterministic installer and upgrade UX expectations that are executable
in local and CI release lanes.

## Contract identifiers

- Installer UX contract ID: `rugo.installer_ux_contract.v3`
- Upgrade drill schema: `rugo.upgrade_drill.v3`
- Recovery drill schema: `rugo.recovery_drill.v3`
- Rollback safety schema: `rugo.rollback_safety_report.v3`

## Installer UX baseline

- Deterministic seed: `20260309`.
- Upgrade candidate must not violate rollback floor policy.
- Rollback floor policy key: `trusted_floor_sequence`.
- Upgrade drill required stages:
  - `upgrade_plan_validate`
  - `upgrade_apply`
  - `post_upgrade_health_check`
  - `rollback_safety_check`
- Maximum allowed upgrade failures: `0`.

## Required artifacts

- `tools/run_upgrade_drill_v3.py`
- `tools/run_recovery_drill_v3.py`
- `out/upgrade-drill-v3.json`
- `out/recovery-drill-v3.json`

## Required gates

- Local gate: `make test-ops-ux-v3`
- CI gate: `Ops UX v3 gate`
