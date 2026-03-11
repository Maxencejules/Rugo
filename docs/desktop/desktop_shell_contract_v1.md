# Desktop Shell Contract v1

Date: 2026-03-11  
Milestone: M52 Desktop Shell + Workflow Baseline v1  
Status: active release gate

## Objective

Define the bounded graphical shell that sits above the GUI runtime and below
the declared session workflows for the first usable desktop path.

## Contract identifiers

- Desktop shell contract ID: `rugo.desktop_shell_contract.v1`.
- Parent GUI runtime contract ID: `rugo.gui_runtime_contract.v1`.
- Parent desktop profile ID: `rugo.desktop_profile.v2`.
- Session workflow profile ID: `rugo.session_workflow_profile.v1`.
- Shell workflow report schema: `rugo.desktop_shell_workflow_report.v1`.
- Graphical installer smoke schema: `rugo.graphical_installer_smoke_report.v1`.

## Declared shell surfaces and components

- Primary shell surface:
  - `desktop.shell.workspace`
- Shell interaction surfaces:
  - `desktop.shell.launcher`
  - `desktop.shell.taskbar`
  - `desktop.shell.status_bar`
  - `desktop.shell.power_menu`
- Declared workflow windows:
  - `files.panel`
  - `settings.panel`

## Required shell checks

- `launcher_open_budget`:
  - launcher open latency p95 must remain `<= 60 ms`.
- `launcher_activation_integrity`:
  - launcher activation mismatches must remain `= 0`.
- `app_switch_latency_budget`:
  - launcher-to-app switch latency p95 must remain `<= 42 ms`.
- `shell_focus_restore_integrity`:
  - focus-restore violations must remain `= 0`.
- `file_picker_roundtrip_budget`:
  - declared file picker round-trip latency p95 must remain `<= 80 ms`.
- `file_save_commit_budget`:
  - file save commit latency p95 must remain `<= 95 ms`.
- `settings_apply_budget`:
  - settings apply latency p95 must remain `<= 85 ms`.
- `settings_persist_integrity`:
  - settings persistence violations must remain `= 0`.
- `shutdown_request_budget`:
  - shutdown request latency p95 must remain `<= 55 ms`.
- `shutdown_surface_cleanup_integrity`:
  - shell shutdown cleanup violations must remain `= 0`.

## Reporting requirements

- `out/desktop-shell-v1.json` must expose:
  - `shell_components`
  - `workflows`
  - `session_state`
  - `source_reports`
  - `digest`
- Every workflow record must include:
  - `workflow_id`
  - `category`
  - `start_focus`
  - `end_focus`
  - `steps`
  - `checks_pass`

## Release gating

- Local gate: `make test-desktop-shell-v1`.
- Local sub-gate: `make test-desktop-workflows-v1`.
- CI gate: `Desktop shell v1 gate`.
- CI sub-gate: `Desktop workflows v1 gate`.
