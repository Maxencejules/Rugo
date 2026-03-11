# Surface Lifecycle Contract v1

Date: 2026-03-11  
Milestone: M50 Window System + Composition v1  
Status: active release gate

## Objective

Define deterministic surface ownership and lifecycle semantics for the first
live in-tree window system.

## Contract identifiers

- Surface lifecycle contract ID: `rugo.surface_lifecycle_contract.v1`.
- Parent display runtime contract ID: `rugo.display_runtime_contract.v1`.
- Parent seat contract ID: `rugo.seat_input_contract.v1`.
- Runtime report schema: `rugo.window_system_runtime_report.v1`.
- Compositor damage schema: `rugo.compositor_damage_report.v1`.
- Window manager contract ID: `rugo.window_manager_contract.v2`.
- Compositor damage policy ID: `rugo.compositor_damage_policy.v1`.

## Declared lifecycle states

- Required states:
  - `created`
  - `mapped`
  - `visible`
  - `occluded`
  - `focused`
  - `unmapped`
  - `destroyed`
- Required transition order:
  - `created -> mapped -> visible`
  - `visible -> focused` for the active top-level window
  - `visible|focused -> unmapped -> destroyed` for a retiring surface
- Surface state transitions must remain single-owner and monotonic within one
  runtime report digest.

## Required checks

- `surface_create_budget`:
  - surface creation latency must remain `<= 65 ms`.
- `surface_map_budget`:
  - surface map latency must remain `<= 45 ms`.
- `surface_visibility_integrity`:
  - lifecycle state violations must remain `= 0`.
- `surface_activate_budget`:
  - focus/activation latency must remain `<= 35 ms`.
- `surface_unmap_budget`:
  - surface unmap latency must remain `<= 28 ms`.
- `surface_release_budget`:
  - surface release latency must remain `<= 28 ms`.

## Runtime reporting requirements

- `out/window-system-v1.json` must expose:
  - `seat`
  - `output`
  - `surfaces`
  - `retired_surfaces`
  - `lifecycle_log`
  - `geometry_mutations`
  - `source_reports`
  - `digest`
- Every lifecycle event must record:
  - `surface_id`
  - `window_id`
  - `phase`
  - `state_after`
  - `latency_ms`
- Runtime closure requires the focused top-level surface and every destroyed
  surface to be auditable from the lifecycle log alone.

## Release gating

- Local gate: `make test-window-system-v1`.
- Local sub-gate: `make test-compositor-damage-v1`.
- CI gate: `Window system v1 gate`.
- CI sub-gate: `Compositor damage v1 gate`.
