# M52 Execution Backlog (Desktop Shell + Workflow Baseline v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: proposed

## Goal

Deliver a minimal usable graphical shell and a bounded set of end-user
workflows on top of the implemented display, input, window, and GUI runtime
stack.

M52 source of truth remains `docs/M48_M52_GUI_IMPLEMENTATION_ROADMAP.md`,
`MILESTONES.md`, and this backlog.

## Current State Summary

- Installer/recovery UX and desktop profile docs already exist.
- The repo does not yet provide an in-tree graphical shell or daily-use
  graphical workflows.
- A real GUI phase is only complete once users can reach and operate a bounded
  desktop workflow path rather than just pass lower-layer runtime tests.

## Execution plan

- PR-1: shell/workflow contract freeze
- PR-2: shell implementation + workflow campaigns
- PR-3: release gate wiring + closure

## Execution Result

- PR-1: not started
- PR-2: not started
- PR-3: not started

## PR-1: Shell/Workflow Contract Freeze

### Objective

Define the minimal shell, session workflow, and graphical installer boundaries
for the first usable desktop path.

### Scope

- Add docs:
  - `docs/desktop/desktop_shell_contract_v1.md`
  - `docs/desktop/session_workflow_profile_v1.md`
  - `docs/build/graphical_installer_ux_v1.md`
- Add tests:
  - `tests/desktop/test_desktop_shell_docs_v1.py`

### Primary files

- `docs/desktop/desktop_shell_contract_v1.md`
- `docs/desktop/session_workflow_profile_v1.md`
- `docs/build/graphical_installer_ux_v1.md`
- `tests/desktop/test_desktop_shell_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/desktop/test_desktop_shell_docs_v1.py -v`

### Done criteria for PR-1

- Shell and workflow boundaries are explicit and versioned.
- First-use and installer expectations are reviewable before implementation.

## PR-2: Shell Runtime + Workflow Campaigns

### Objective

Implement the first usable shell/workflow path and collect deterministic
artifacts for declared daily-use flows.

### Scope

- Add tooling:
  - `tools/run_desktop_shell_workflows_v1.py`
  - `tools/run_graphical_installer_smoke_v1.py`
- Add tests:
  - `tests/desktop/test_shell_launcher_workflow_v1.py`
  - `tests/desktop/test_file_open_save_workflow_v1.py`
  - `tests/desktop/test_settings_workflow_v1.py`
  - `tests/build/test_graphical_installer_smoke_v1.py`

### Primary files

- `tools/run_desktop_shell_workflows_v1.py`
- `tools/run_graphical_installer_smoke_v1.py`
- `tests/desktop/test_shell_launcher_workflow_v1.py`
- `tests/desktop/test_file_open_save_workflow_v1.py`
- `tests/desktop/test_settings_workflow_v1.py`
- `tests/build/test_graphical_installer_smoke_v1.py`

### Acceptance checks

- `python tools/run_desktop_shell_workflows_v1.py --out out/desktop-shell-v1.json`
- `python tools/run_graphical_installer_smoke_v1.py --out out/graphical-installer-v1.json`
- `python -m pytest tests/desktop/test_shell_launcher_workflow_v1.py tests/desktop/test_file_open_save_workflow_v1.py tests/desktop/test_settings_workflow_v1.py tests/build/test_graphical_installer_smoke_v1.py -v`

### Done criteria for PR-2

- Shell and workflow artifacts are deterministic and machine-readable.
- Declared user workflows execute through a live graphical shell path.

## PR-3: Desktop Shell Gate + Workflow Sub-gate

### Objective

Make the bounded graphical shell path release-blocking once the underlying GUI
stack is in place.

### Scope

- Add local gates:
  - `Makefile` target `test-desktop-shell-v1`
  - `Makefile` target `test-desktop-workflows-v1`
- Add CI steps:
  - `Desktop shell v1 gate`
  - `Desktop workflows v1 gate`
- Add aggregate tests:
  - `tests/desktop/test_desktop_shell_gate_v1.py`
  - `tests/desktop/test_desktop_workflows_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/desktop/test_desktop_shell_gate_v1.py`
- `tests/desktop/test_desktop_workflows_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-desktop-shell-v1`
- `make test-desktop-workflows-v1`

### Done criteria for PR-3

- The bounded graphical shell path is release-blocking in local and CI lanes.
- Real desktop workflows, not just lower-layer runtime checks, determine GUI
  readiness for the declared profile.

## Non-goals for M52 backlog

- broad office/media/browser ecosystem parity
- multi-user login-manager, accessibility-suite, or enterprise policy breadth
- claiming universal desktop usability outside the declared workflow profile
