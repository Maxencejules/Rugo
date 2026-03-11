# M51 Execution Backlog (GUI Runtime + Toolkit Bridge v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: proposed

## Goal

Deliver a bounded GUI runtime that lets declared in-tree apps and toolkit
profiles launch, render, and process input through the implemented graphical
stack.

M51 source of truth remains `docs/M48_M52_GUI_IMPLEMENTATION_ROADMAP.md`,
`MILESTONES.md`, and this backlog.

## Current State Summary

- GUI qualification tiers and runtime-backed workload artifacts already exist.
- Those artifacts still do not correspond to a real in-tree toolkit/runtime
  boundary or live graphical app execution path.
- After display, input, and windowing exist, the next missing layer is the
  actual GUI runtime and toolkit-facing contract.

## Execution plan

- PR-1: GUI runtime contract freeze
- PR-2: runtime/toolkit implementation + app campaigns
- PR-3: release gate wiring + closure

## Execution Result

- PR-1: not started
- PR-2: not started
- PR-3: not started

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

## Non-goals for M51 backlog

- universal third-party toolkit parity
- broad browser, video-acceleration, or game-engine support claims
- text shaping, internationalization, or font breadth beyond the declared v1
  profile
