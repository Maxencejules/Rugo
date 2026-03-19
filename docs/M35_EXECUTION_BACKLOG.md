# M35 Execution Backlog (Desktop + Interactive UX Baseline v1)

Date: 2026-03-09  
Lane: Rugo (Rust kernel + Go user space)  
Status: done

## Goal

Promote desktop and interactive UX from explicit out-of-scope status to a
bounded, deterministic baseline with release-gated evidence.

M35 source of truth remains `docs/M35_M39_GENERAL_PURPOSE_EXPANSION_ROADMAP.md`,
`MILESTONES.md`, and this backlog.

## Current State Summary

- Compatibility profiles still mark desktop/GUI compatibility as unsupported.
- Operability and installer UX are mature for operator workflows, not full
  desktop session workflows.
- M35 introduces desktop/session/input contracts and deterministic GUI baseline
  gates.

## Execution Result

- PR-1: complete (2026-03-09)
- PR-2: complete (2026-03-09)
- PR-3: complete (2026-03-09)

## PR-1: Desktop Contract Freeze

### Objective

Define desktop/session/input baseline expectations as executable contracts.

### Scope

- Add docs:
  - `docs/desktop/display_stack_contract_v1.md`
  - `docs/desktop/window_manager_contract_v1.md`
  - `docs/desktop/input_stack_contract_v1.md`
  - `docs/desktop/desktop_profile_v1.md`
- Add tests:
  - `tests/desktop/test_desktop_docs_v1.py`

### Primary files

- `docs/desktop/display_stack_contract_v1.md`
- `docs/desktop/window_manager_contract_v1.md`
- `docs/desktop/input_stack_contract_v1.md`
- `docs/desktop/desktop_profile_v1.md`
- `tests/desktop/test_desktop_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/desktop/test_desktop_docs_v1.py -v`

### Done criteria for PR-1

- Desktop/session/input contracts are explicit, versioned, and test-referenced.

### PR-1 completion summary

- Added desktop contract docs:
  - `docs/desktop/display_stack_contract_v1.md`
  - `docs/desktop/window_manager_contract_v1.md`
  - `docs/desktop/input_stack_contract_v1.md`
  - `docs/desktop/desktop_profile_v1.md`
- Added executable contract checks:
  - `tests/desktop/test_desktop_docs_v1.py`

## PR-2: Desktop Baseline Tooling + Tests

### Objective

Implement deterministic desktop smoke and GUI app-compat baseline behavior.

### Scope

- Add tooling:
  - `tools/run_desktop_smoke_v1.py`
  - `tools/run_gui_app_matrix_v1.py`
- Add tests:
  - `tests/desktop/test_display_session_v1.py`
  - `tests/desktop/test_input_baseline_v1.py`
  - `tests/desktop/test_window_lifecycle_v1.py`
  - `tests/desktop/test_gui_app_compat_v1.py`

### Primary files

- `tools/run_desktop_smoke_v1.py`
- `tools/run_gui_app_matrix_v1.py`
- `tests/desktop/test_display_session_v1.py`
- `tests/desktop/test_input_baseline_v1.py`
- `tests/desktop/test_window_lifecycle_v1.py`
- `tests/desktop/test_gui_app_compat_v1.py`

### Acceptance checks

- `python tools/run_desktop_smoke_v1.py --out out/desktop-smoke-v1.json`
- `python tools/run_gui_app_matrix_v1.py --out out/gui-app-matrix-v1.json`
- `python -m pytest tests/desktop/test_display_session_v1.py tests/desktop/test_input_baseline_v1.py tests/desktop/test_window_lifecycle_v1.py tests/desktop/test_gui_app_compat_v1.py -v`

### Done criteria for PR-2

- Desktop and GUI artifacts are deterministic and machine-readable.
- Session/input/window lifecycle behavior is executable and auditable.

### PR-2 completion summary

- Added deterministic desktop and GUI tooling:
  - `tools/run_desktop_smoke_v1.py`
  - `tools/run_gui_app_matrix_v1.py`
- Added executable baseline checks:
  - `tests/desktop/test_display_session_v1.py`
  - `tests/desktop/test_input_baseline_v1.py`
  - `tests/desktop/test_window_lifecycle_v1.py`
  - `tests/desktop/test_gui_app_compat_v1.py`

## PR-3: Desktop Gate + GUI Sub-gate

### Objective

Make desktop baseline and GUI app compatibility release-blocking.

### Scope

- Add local gates:
  - `Makefile` target `test-desktop-stack-v1`
  - `Makefile` target `test-gui-app-compat-v1`
- Add CI steps:
  - `Desktop stack v1 gate`
  - `GUI app compatibility v1 gate`
- Add aggregate tests:
  - `tests/desktop/test_desktop_gate_v1.py`
  - `tests/desktop/test_gui_app_compat_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/desktop/test_desktop_gate_v1.py`
- `tests/desktop/test_gui_app_compat_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-desktop-stack-v1`
- `make test-gui-app-compat-v1`

### Done criteria for PR-3

- Desktop and GUI sub-gates are required in local and CI release lanes.
- M35 can be marked done with deterministic artifact evidence.

### PR-3 completion summary

- Added aggregate gate checks:
  - `tests/desktop/test_desktop_gate_v1.py`
  - `tests/desktop/test_gui_app_compat_gate_v1.py`
- Added local gates:
  - `make test-desktop-stack-v1`
  - `make test-gui-app-compat-v1`
- Added CI gates and artifacts:
  - `Desktop stack v1 gate`
  - `GUI app compatibility v1 gate`
- Updated closure docs:
  - `MILESTONES.md`
  - `docs/STATUS.md`
  - `README.md`

## X4 runtime-backed closure addendum (2026-03-18)

- This backlog is now governed by the shared desktop profile runtime
  qualification lane.
- The historical desktop/session baseline now feeds the boot-backed
  `os-go-desktop.iso` image through `make test-desktop-profile-runtime-v1`
  instead of remaining a standalone deterministic-only lane.
- Future desktop baseline breadth must land through the same desktop profile
  runtime qualification path before it is treated as an expanded X4 claim.

## Non-goals for M35 backlog

- Universal desktop compatibility claims beyond declared profile.
- Full compositor/graphics parity with mature desktop OS stacks.
