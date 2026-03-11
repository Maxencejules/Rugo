# Graphical Installer UX v1

Date: 2026-03-11  
Milestone: M52 Desktop Shell + Workflow Baseline v1  
Status: active release gate

## Objective

Define the bounded graphical installer path that is allowed to claim shell and
first-boot usability for the M52 desktop workflow baseline.

## Contract identifiers

- Graphical installer UX ID: `rugo.graphical_installer_ux.v1`.
- Parent installer UX contract ID: `rugo.installer_ux_contract.v3`.
- Parent desktop shell contract ID: `rugo.desktop_shell_contract.v1`.
- Recovery workflow ID: `rugo.recovery_workflow.v3`.
- Installer contract schema: `rugo.installer_contract.v2`.
- Graphical installer smoke schema: `rugo.graphical_installer_smoke_report.v1`.

## Declared installer stages

- `shell_bootstrap`
- `device_scan`
- `target_selection`
- `layout_review`
- `install_commit`
- `first_boot_handoff`

## Required graphical installer checks

- `shell_entry_budget` must remain `<= 90 ms`.
- `device_discovery_budget` must remain `<= 140 ms`.
- `target_selection_integrity` must remain `= 0` violations.
- `layout_validation_integrity` must remain `= 0` violations.
- `install_commit_budget` must remain `<= 180 ms`.
- `first_boot_handoff_integrity` must remain `= 0` violations.
- `recovery_entry_visible` must remain `>= 1.0`.
- Maximum allowed graphical installer failures: `0`.

## Required artifacts

- `tools/run_graphical_installer_smoke_v1.py`
- `out/graphical-installer-v1.json`
- `out/desktop-shell-v1.json`
- `out/recovery-drill-v3.json`

## Gate anchors

- Local gate: `make test-desktop-shell-v1`.
- Local sub-gate: `make test-desktop-workflows-v1`.
- CI gate: `Desktop shell v1 gate`.
- CI sub-gate: `Desktop workflows v1 gate`.
