# M70 Execution Backlog (Desktop Productivity Workflow v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Add bounded productivity workflows on top of the post-M52 desktop so the shell
feels like a usable daily environment without claiming full workstation parity.

M70 source of truth remains:

- `docs/POST_G2_EXTENDED_MILESTONES.md`
- `docs/M67_EXECUTION_BACKLOG.md`
- `docs/M69_EXECUTION_BACKLOG.md`
- this backlog

## Current State Summary

- Accessibility, file management, configuration, and multi-monitor work define
  the prerequisites for richer desktop workflows.
- Search, clipboard, drag-drop, and session restore are not yet explicit,
  release-gated contract surfaces.
- Productivity claims remain bounded to shell launcher workflows from M52.
- M70 must define those semantics before package, app-bundle, and later
  desktop ecosystem work depends on them.

## Execution plan

- PR-1: productivity workflow contract freeze
- PR-2: search, clipboard, and restore campaign baseline
- PR-3: productivity gate wiring + closure

## Execution status

- PR-1: pending
- PR-2: pending
- PR-3: pending

## PR-1: Productivity Workflow Contract Freeze

### Objective

Define search, clipboard, drag-drop, and session-restore semantics before
implementation broadens desktop workflow claims.

### Scope

- Add docs:
  - `docs/desktop/productivity_workflow_profile_v1.md`
  - `docs/desktop/clipboard_dragdrop_contract_v1.md`
  - `docs/desktop/session_restore_policy_v1.md`
- Add tests:
  - `tests/desktop/test_productivity_docs_v1.py`

### Primary files

- `docs/desktop/productivity_workflow_profile_v1.md`
- `docs/desktop/clipboard_dragdrop_contract_v1.md`
- `docs/desktop/session_restore_policy_v1.md`
- `tests/desktop/test_productivity_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/desktop/test_productivity_docs_v1.py -v`

### Done criteria for PR-1

- Productivity workflow semantics are explicit and versioned.
- Search, clipboard, and restore boundaries are reviewable before
  implementation lands.

## PR-2: Search, Clipboard, and Restore Campaign Baseline

### Objective

Implement deterministic evidence for app search, drag-drop, clipboard, and
session restore behavior.

### Scope

- Add tooling:
  - `tools/run_desktop_productivity_v1.py`
  - `tools/run_session_restore_v1.py`
- Add tests:
  - `tests/desktop/test_app_search_launch_v1.py`
  - `tests/desktop/test_clipboard_dragdrop_v1.py`
  - `tests/desktop/test_session_restore_v1.py`
  - `tests/desktop/test_productivity_negative_v1.py`

### Primary files

- `tools/run_desktop_productivity_v1.py`
- `tools/run_session_restore_v1.py`
- `tests/desktop/test_app_search_launch_v1.py`
- `tests/desktop/test_clipboard_dragdrop_v1.py`
- `tests/desktop/test_session_restore_v1.py`
- `tests/desktop/test_productivity_negative_v1.py`

### Acceptance checks

- `python tools/run_desktop_productivity_v1.py --out out/desktop-productivity-v1.json`
- `python tools/run_session_restore_v1.py --out out/session-restore-v1.json`
- `python -m pytest tests/desktop/test_app_search_launch_v1.py tests/desktop/test_clipboard_dragdrop_v1.py tests/desktop/test_session_restore_v1.py tests/desktop/test_productivity_negative_v1.py -v`

### Done criteria for PR-2

- Productivity artifacts are deterministic and machine-readable.
- `DESKTOP: launch ok` and restore markers remain stable.
- Later packaging and app-bundle work can depend on one explicit desktop
  workflow profile.

## PR-3: Productivity Gate + Session Restore Sub-gate

### Objective

Make the desktop productivity baseline release-blocking for the declared shell
profile.

### Scope

- Add local gates:
  - `Makefile` target `test-desktop-productivity-v1`
  - `Makefile` target `test-session-restore-v1`
- Add CI steps:
  - `Desktop productivity v1 gate`
  - `Session restore v1 gate`
- Add aggregate tests:
  - `tests/desktop/test_desktop_productivity_gate_v1.py`
  - `tests/desktop/test_session_restore_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/desktop/test_desktop_productivity_gate_v1.py`
- `tests/desktop/test_session_restore_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-desktop-productivity-v1`
- `make test-session-restore-v1`

### Done criteria for PR-3

- Productivity and session-restore sub-gates are required in local and CI
  release lanes.
- M70 can be marked done only with deterministic desktop productivity evidence
  for the declared workflow profile.

## Non-goals for M70 backlog

- full office/browser ecosystem parity
- package solver and app-bundle work owned by M71-M72
- enterprise desktop policy breadth outside the declared productivity scope
