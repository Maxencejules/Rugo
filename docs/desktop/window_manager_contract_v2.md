# Window Manager Contract v2

Date: 2026-03-11  
Milestone: M50 Window System + Composition v1  
Status: active release gate

## Objective

Define z-order, focus ownership, move/resize, and composition behavior for the
first live bounded window manager.

## Contract identifiers

- Window manager contract ID: `rugo.window_manager_contract.v2`.
- Supersedes window manager contract ID: `rugo.window_manager_contract.v1`.
- Parent desktop profile ID: `rugo.desktop_profile.v2`.
- Parent seat contract ID: `rugo.seat_input_contract.v1`.
- Runtime report schema: `rugo.window_system_runtime_report.v1`.
- Compositor damage schema: `rugo.compositor_damage_report.v1`.
- Surface lifecycle contract ID: `rugo.surface_lifecycle_contract.v1`.
- Compositor damage policy ID: `rugo.compositor_damage_policy.v1`.

## Declared stacking model

- Top-level stacking is ordered by `stacking_layer` then `z_index`.
- Required layers:
  - `background`
  - `normal`
  - `overlay`
- Only the topmost visible `normal` window may own focus.
- Opaque top-level windows must clip lower `normal` windows before damage is
  committed to the output.

## Required checks

- `z_order_integrity` must remain `0` ordering violations.
- `focus_z_order_alignment` must remain `0` focus/z-order alignment violations.
- `occlusion_clip_integrity` must remain `0` clipping violations.
- `window_move_budget` must remain `<= 24 ms`.
- `window_resize_budget` must remain `<= 32 ms`.
- `compositor_frame_budget` must remain `<= 16.667 ms`.

## Runtime reporting requirements

- `out/window-system-v1.json` must expose:
  - `z_order`
  - `composition`
  - `geometry_mutations`
  - `artifact_refs`
- The runtime report must record:
  - `focus_owner`
  - `topmost_focusable_window`
  - `render_order`
  - `ordering_violations`
  - `occlusion_clip_violations`
- Move and resize operations must preserve deterministic final geometry for the
  active focused window.

## Release gating

- Local gate: `make test-window-system-v1`.
- Local sub-gate: `make test-compositor-damage-v1`.
- CI gate: `Window system v1 gate`.
- CI sub-gate: `Compositor damage v1 gate`.
