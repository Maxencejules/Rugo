# M51 Execution Backlog (GUI Runtime + Toolkit Bridge v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: done

Archive note: this is a historical execution record. The current repo intro is
architecture-first and now lives in `README.md` plus `docs/roadmap/README.md`.

## Goal

Deliver a bounded GUI runtime that lets declared in-tree apps and toolkit
profiles launch, render, and process input through the implemented graphical
stack.

M51 source of truth remains `docs/M48_M52_GUI_IMPLEMENTATION_ROADMAP.md`,
`MILESTONES.md`, and this backlog.

## Current State Summary

- GUI runtime, toolkit profile, and font/text contracts now define the bounded
  in-tree app/runtime/toolkit boundary.
- Deterministic GUI runtime and toolkit compatibility tooling now produce
  machine-readable launch, render, text, and event-loop evidence.
- Live GUI runtime behavior is now wired into local and CI release gates.

## Execution plan

- PR-1: GUI runtime contract freeze
- PR-2: runtime/toolkit implementation + app campaigns
- PR-3: release gate wiring + closure

## Execution Result

- PR-1: complete (2026-03-11)
- PR-2: complete (2026-03-11)
- PR-3: complete (2026-03-11)

## PR-1: GUI Runtime Contract Freeze

### Objective

Define the GUI runtime, toolkit profile, and text/rendering boundaries before
app-facing implementation begins.

### Scope

- Add docs:
  - `docs/desktop/gui_runtime_contract_v1.md`
  - `docs/desktop/toolkit_profile_v1.md`
  - `docs/desktop/font_text_rendering_policy_v1.md`
- Add tests:
  - `tests/desktop/test_gui_runtime_docs_v1.py`

### Primary files

- `docs/desktop/gui_runtime_contract_v1.md`
- `docs/desktop/toolkit_profile_v1.md`
- `docs/desktop/font_text_rendering_policy_v1.md`
- `tests/desktop/test_gui_runtime_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/desktop/test_gui_runtime_docs_v1.py -v`

### Done criteria for PR-1

- GUI runtime and toolkit boundaries are explicit and versioned.
- Font/text/event-loop expectations are reviewable before runtime code lands.

### PR-1 completion summary

- Added contract docs:
  - `docs/desktop/gui_runtime_contract_v1.md`
  - `docs/desktop/toolkit_profile_v1.md`
  - `docs/desktop/font_text_rendering_policy_v1.md`
- Added executable doc checks:
  - `tests/desktop/test_gui_runtime_docs_v1.py`

## PR-2: GUI Runtime + Toolkit Compatibility Campaigns

### Objective

Implement live app runtime behavior and collect deterministic evidence for the
declared toolkit and render/event profile.

### Scope

- Add tooling:
  - `tools/run_gui_runtime_v1.py`
  - `tools/run_toolkit_compat_v1.py`
- Add tests:
  - `tests/desktop/test_gui_app_launch_render_v1.py`
  - `tests/desktop/test_font_text_rendering_v1.py`
  - `tests/desktop/test_toolkit_event_loop_v1.py`
  - `tests/desktop/test_gui_runtime_negative_v1.py`

### Primary files

- `tools/run_gui_runtime_v1.py`
- `tools/run_toolkit_compat_v1.py`
- `tests/desktop/test_gui_app_launch_render_v1.py`
- `tests/desktop/test_font_text_rendering_v1.py`
- `tests/desktop/test_toolkit_event_loop_v1.py`
- `tests/desktop/test_gui_runtime_negative_v1.py`

### Acceptance checks

- `python tools/run_gui_runtime_v1.py --out out/gui-runtime-v1.json`
- `python tools/run_toolkit_compat_v1.py --out out/toolkit-compat-v1.json`
- `python -m pytest tests/desktop/test_gui_app_launch_render_v1.py tests/desktop/test_font_text_rendering_v1.py tests/desktop/test_toolkit_event_loop_v1.py tests/desktop/test_gui_runtime_negative_v1.py -v`

### Done criteria for PR-2

- GUI runtime artifacts are deterministic and machine-readable.
- Declared apps can render and receive input through a live runtime/toolkit path.

### PR-2 completion summary

- Added deterministic GUI runtime tooling:
  - `tools/run_gui_runtime_v1.py`
  - `tools/run_toolkit_compat_v1.py`
- Added executable runtime and toolkit checks:
  - `tests/desktop/test_gui_app_launch_render_v1.py`
  - `tests/desktop/test_font_text_rendering_v1.py`
  - `tests/desktop/test_toolkit_event_loop_v1.py`
  - `tests/desktop/test_gui_runtime_negative_v1.py`

## PR-3: GUI Runtime Gate + Toolkit Sub-gate

### Objective

Make live GUI runtime behavior release-blocking for the declared bounded app
profiles.

### Scope

- Add local gates:
  - `Makefile` target `test-gui-runtime-v1`
  - `Makefile` target `test-toolkit-compat-v1`
- Add CI steps:
  - `GUI runtime v1 gate`
  - `Toolkit compatibility v1 gate`
- Add aggregate tests:
  - `tests/desktop/test_gui_runtime_gate_v1.py`
  - `tests/desktop/test_toolkit_compat_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/desktop/test_gui_runtime_gate_v1.py`
- `tests/desktop/test_toolkit_compat_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-gui-runtime-v1`
- `make test-toolkit-compat-v1`

### Done criteria for PR-3

- GUI runtime and toolkit regressions are blocked in local and CI lanes.
- Declared app-profile claims are tied to live runtime evidence instead of
  simulated matrices alone.

### PR-3 completion summary

- Added local gates:
  - `make test-gui-runtime-v1`
  - `make test-toolkit-compat-v1`
- Added aggregate gate tests:
  - `tests/desktop/test_gui_runtime_gate_v1.py`
  - `tests/desktop/test_toolkit_compat_gate_v1.py`
- Updated repo-level closure documents:
  - `MILESTONES.md`
  - `docs/STATUS.md`
  - `README.md`

## Non-goals for M51 backlog

- universal third-party toolkit parity
- broad browser, video-acceleration, or game-engine support claims
- text shaping, internationalization, or font breadth beyond the declared v1
  profile
