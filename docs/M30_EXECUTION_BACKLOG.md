# M30 Execution Backlog (Installer/Upgrade/Recovery UX v3)

Date: 2026-03-06  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Raise install/upgrade/rollback/recovery workflows from engineering-capable to
robust day-to-day operator quality.

M30 source of truth remains `docs/M21_M34_MATURITY_PARITY_ROADMAP.md`,
`MILESTONES.md`, and this backlog.

## Current State Summary

- Prior release-operability milestones established baseline workflows.
- M30 focuses on v3 workflow determinism and operator UX quality.
- Existing runbooks need v3 contract and gate references.

## Execution Result

- PR-1: planned
- PR-2: planned
- PR-3: planned

## PR-1: Installer + Recovery Contract v3

### Objective

Freeze installer/recovery workflow contracts for v3 operator baseline.

### Scope

- Add docs:
  - `docs/build/installer_ux_v3.md`
  - `docs/build/recovery_workflow_v3.md`
- Add tests:
  - `tests/build/test_installer_ux_v3.py`

### Primary files

- `docs/build/installer_ux_v3.md`
- `docs/build/recovery_workflow_v3.md`
- `tests/build/test_installer_ux_v3.py`

### Acceptance checks

- `python -m pytest tests/build/test_installer_ux_v3.py -v`

### Done criteria for PR-1

- Installer/recovery UX contracts are explicit and test-referenced.

## PR-2: Upgrade + Recovery Drill Tooling

### Objective

Validate deterministic upgrade/recovery/rollback behavior at v3.

### Scope

- Add tooling:
  - `tools/run_upgrade_drill_v3.py`
  - `tools/run_recovery_drill_v3.py`
- Add tests:
  - `tests/build/test_upgrade_recovery_v3.py`
  - `tests/build/test_rollback_safety_v3.py`

### Primary files

- `tools/run_upgrade_drill_v3.py`
- `tools/run_recovery_drill_v3.py`
- `tests/build/test_upgrade_recovery_v3.py`
- `tests/build/test_rollback_safety_v3.py`

### Acceptance checks

- `python tools/run_upgrade_drill_v3.py --out out/upgrade-drill-v3.json`
- `python tools/run_recovery_drill_v3.py --out out/recovery-drill-v3.json`
- `python -m pytest tests/build/test_upgrade_recovery_v3.py tests/build/test_rollback_safety_v3.py -v`

### Done criteria for PR-2

- Upgrade/recovery drills are deterministic and machine-readable.
- Rollback safety behavior is executable and auditable.

## PR-3: Ops UX v3 Gate + Closure

### Objective

Make operational UX v3 release-blocking.

### Scope

- Add local gate:
  - `Makefile` target `test-ops-ux-v3`
- Add CI step:
  - `Ops UX v3 gate`
- Add aggregate test:
  - `tests/build/test_ops_ux_gate_v3.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/build/test_ops_ux_gate_v3.py`
- `MILESTONES.md`
- `docs/STATUS.md`

### Acceptance checks

- `make test-ops-ux-v3`

### Done criteria for PR-3

- Ops UX v3 gate is required in local and CI release lanes.
- M30 can be marked done with drill artifacts and runbook links.

## Non-goals for M30 backlog

- Broad desktop installer UI permutations.
- Fleet orchestration controls (owned by later milestones).

