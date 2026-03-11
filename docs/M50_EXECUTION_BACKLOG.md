# M50 Execution Backlog (Window System + Composition v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: proposed

## Goal

Implement an in-tree window system and compositor that own surface lifecycle,
visibility, and damage semantics for the declared desktop profile.

M50 source of truth remains `docs/M48_M52_GUI_IMPLEMENTATION_ROADMAP.md`,
`MILESTONES.md`, and this backlog.

## Current State Summary

- Window-manager contracts and bounded desktop qualifications already exist.
- The repo does not yet have a live window server/compositor/surface runtime.
- Input and display milestones can only support a real desktop once window and
  composition semantics are executable in-tree.

## Execution plan

- PR-1: window/compositor contract freeze
- PR-2: window runtime implementation + composition campaigns
- PR-3: release gate wiring + closure

## Execution Result

- PR-1: not started
- PR-2: not started
- PR-3: not started

## PR-1: Window/Compositor Contract Freeze

### Objective

Define surface lifecycle, composition, and damage semantics before implementing
the first real window system.

### Scope

- Add docs:
  - `docs/desktop/surface_lifecycle_contract_v1.md`
  - `docs/desktop/compositor_damage_policy_v1.md`
  - `docs/desktop/window_manager_contract_v2.md`
- Add tests:
  - `tests/desktop/test_window_system_docs_v1.py`

### Primary files

- `docs/desktop/surface_lifecycle_contract_v1.md`
- `docs/desktop/compositor_damage_policy_v1.md`
- `docs/desktop/window_manager_contract_v2.md`
- `tests/desktop/test_window_system_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/desktop/test_window_system_docs_v1.py -v`

### Done criteria for PR-1

- Surface lifecycle and composition semantics are explicit and versioned.
- Damage and visibility behavior are reviewable before runtime code lands.

## PR-2: Window Runtime + Composition Campaigns

### Objective

Implement live surface/window/compositor behavior and collect deterministic
composition evidence.

### Scope

- Add tooling:
  - `tools/run_window_system_runtime_v1.py`
  - `tools/run_compositor_damage_v1.py`
- Add tests:
  - `tests/desktop/test_surface_lifecycle_v1.py`
  - `tests/desktop/test_window_zorder_v1.py`
  - `tests/desktop/test_compositor_damage_regions_v1.py`
  - `tests/desktop/test_window_resize_move_v1.py`

### Primary files

- `tools/run_window_system_runtime_v1.py`
- `tools/run_compositor_damage_v1.py`
- `tests/desktop/test_surface_lifecycle_v1.py`
- `tests/desktop/test_window_zorder_v1.py`
- `tests/desktop/test_compositor_damage_regions_v1.py`
- `tests/desktop/test_window_resize_move_v1.py`

### Acceptance checks

- `python tools/run_window_system_runtime_v1.py --out out/window-system-v1.json`
- `python tools/run_compositor_damage_v1.py --out out/compositor-damage-v1.json`
- `python -m pytest tests/desktop/test_surface_lifecycle_v1.py tests/desktop/test_window_zorder_v1.py tests/desktop/test_compositor_damage_regions_v1.py tests/desktop/test_window_resize_move_v1.py -v`

### Done criteria for PR-2

- Window-system artifacts are deterministic and machine-readable.
- Real composition behavior is exercised through live surfaces and windows.

## PR-3: Window System Gate + Compositor Sub-gate

### Objective

Make live window/compositor behavior release-blocking for the declared desktop
profile.

### Scope

- Add local gates:
  - `Makefile` target `test-window-system-v1`
  - `Makefile` target `test-compositor-damage-v1`
- Add CI steps:
  - `Window system v1 gate`
  - `Compositor damage v1 gate`
- Add aggregate tests:
  - `tests/desktop/test_window_system_gate_v1.py`
  - `tests/desktop/test_compositor_damage_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/desktop/test_window_system_gate_v1.py`
- `tests/desktop/test_compositor_damage_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-window-system-v1`
- `make test-compositor-damage-v1`

### Done criteria for PR-3

- Window-system and compositor regressions are blocked in local and CI lanes.
- Damage, z-order, and lifecycle behavior are tied to explicit runtime artifacts.

## Non-goals for M50 backlog

- X11 or Wayland protocol compatibility claims
- advanced effects, animation systems, or multi-monitor composition
- clipboard, drag-and-drop, or accessibility breadth beyond the first bounded
  workflow set
