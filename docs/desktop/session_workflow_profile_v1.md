# Session Workflow Profile v1

Date: 2026-03-11  
Milestone: M52 Desktop Shell + Workflow Baseline v1  
Status: active release gate

## Objective

Define the bounded end-user workflow set that the first graphical shell must
complete before the desktop path is considered usable.

## Profile identifiers

- Session workflow profile ID: `rugo.session_workflow_profile.v1`.
- Parent desktop shell contract ID: `rugo.desktop_shell_contract.v1`.
- Parent GUI runtime contract ID: `rugo.gui_runtime_contract.v1`.
- Shell workflow report schema: `rugo.desktop_shell_workflow_report.v1`.
- Graphical installer smoke schema: `rugo.graphical_installer_smoke_report.v1`.

## Declared workflow set

- `launcher_open`
- `file_open_save`
- `settings_update`
- `shutdown_request`
- `graphical_installer_smoke`

## Workflow boundary

In scope:
- launcher-driven app activation
- bounded file open and save flow
- bounded settings mutation and persistence flow
- graphical shutdown request and confirmation path
- graphical installer bootstrap and first-boot handoff

Out of scope:
- arbitrary third-party desktop app workflows
- multi-user login/session switching
- browser, office-suite, or media-suite parity claims

## Workflow requirements

- Required shell workflows: `launcher_open`, `file_open_save`, `settings_update`,
  and `shutdown_request`.
- Required installer workflow: `graphical_installer_smoke`.
- Minimum required passing shell workflows: `4`.
- Minimum required passing installer workflows: `1`.
- `files.panel` and `settings.panel` must remain the declared workflow windows.
- `desktop.shell.launcher` must remain the declared activation entrypoint.
- The workflow report must record `steps`, `checks_pass`, and `resulting_window_id`
  for every declared shell workflow.

## Gate requirements

- Shell workflow command:
  - `python tools/run_desktop_shell_workflows_v1.py --out out/desktop-shell-v1.json`
- Graphical installer command:
  - `python tools/run_graphical_installer_smoke_v1.py --out out/graphical-installer-v1.json`
- Local gate: `make test-desktop-shell-v1`.
- Local sub-gate: `make test-desktop-workflows-v1`.
- CI gate: `Desktop shell v1 gate`.
- CI sub-gate: `Desktop workflows v1 gate`.

Gate pass requires:

- shell workflow report `total_failures = 0`
- shell workflow report `gate_pass = true`
- graphical installer smoke report `total_failures = 0`
- graphical installer smoke report `gate_pass = true`
