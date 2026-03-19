# M50 Execution Backlog (Window System + Composition v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: done

## Goal

Implement an in-tree window system and compositor that own surface lifecycle,
visibility, and damage semantics for the declared desktop profile.

M50 source of truth remains `docs/M48_M52_GUI_IMPLEMENTATION_ROADMAP.md`,
`MILESTONES.md`, and this backlog.

## Current State Summary

- Surface lifecycle, compositor damage, and window-manager v2 contracts now
  define the first live in-tree window system.
- Deterministic runtime and damage tooling now produce machine-readable surface,
  z-order, move/resize, and composition artifacts.
- Live window/compositor behavior is now wired into local and CI release gates.

## Execution plan

- PR-1: window/compositor contract freeze
- PR-2: window runtime implementation + composition campaigns
- PR-3: release gate wiring + closure

## Execution Result

- PR-1: complete (2026-03-11)
- PR-2: complete (2026-03-11)
- PR-3: complete (2026-03-11)

## Historical Rugo implementation summary

### Historical Rust kernel surface

- `kernel_rs/src/`: stable display, memory, and event-delivery primitives that
  the first window and composition runtime depended on.
- `arch/` and `boot/`: low-level device and timing behavior that kept window
  presentation and compositor evidence deterministic.

### Historical Go user space surface

- `services/go/`: this milestone was primarily userspace-facing, covering
  surface lifecycle, z-order, damage handling, and compositor behavior.
- `services/go_std/`: not the primary path for this milestone.

### Historical Language-Native Verification

- `make kernel`
- `make userspace`
- `make image-demo`
- `make smoke-demo`
- `make test-window-system-v1`
- `make test-compositor-damage-v1`

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

### PR-1 completion summary

- Added contract docs:
  - `docs/desktop/surface_lifecycle_contract_v1.md`
  - `docs/desktop/compositor_damage_policy_v1.md`
  - `docs/desktop/window_manager_contract_v2.md`
- Added executable doc checks:
  - `tests/desktop/test_window_system_docs_v1.py`

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

### PR-2 completion summary

- Added deterministic window-system tooling:
  - `tools/run_window_system_runtime_v1.py`
  - `tools/run_compositor_damage_v1.py`
- Added executable runtime and compositor checks:
  - `tests/desktop/test_surface_lifecycle_v1.py`
  - `tests/desktop/test_window_zorder_v1.py`
  - `tests/desktop/test_compositor_damage_regions_v1.py`
  - `tests/desktop/test_window_resize_move_v1.py`

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

### PR-3 completion summary

- Added aggregate gate checks:
  - `tests/desktop/test_window_system_gate_v1.py`
  - `tests/desktop/test_compositor_damage_gate_v1.py`
- Added local gates:
  - `make test-window-system-v1`
  - `make test-compositor-damage-v1`
- Added CI qualification gates and artifacts:
  - `Window system v1 gate`
  - `Compositor damage v1 gate`
- Updated closure docs:
  - `MILESTONES.md`
  - `docs/STATUS.md`
  - `README.md`

## X4 runtime-backed closure addendum (2026-03-18)

- This backlog is now governed by the shared desktop profile runtime
  qualification lane.
- The historical surface/compositor reports now feed the boot-backed desktop
  image and the shared X4 aggregate gate.
- Future window/compositor breadth must land through the same desktop-profile
  runtime qualification path before it is treated as an expanded X4 claim.

## Non-goals for M50 backlog

- X11 or Wayland protocol compatibility claims
- advanced effects, animation systems, or multi-monitor composition
- clipboard, drag-and-drop, or accessibility breadth beyond the first bounded
  workflow set
