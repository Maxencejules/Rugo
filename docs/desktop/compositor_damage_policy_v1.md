# Compositor Damage Policy v1

Date: 2026-03-11  
Milestone: M50 Window System + Composition v1  
Status: active release gate

## Objective

Define deterministic damage tracking, clipping, and present-region behavior for
the first bounded compositor runtime.

## Policy identifiers

- Compositor damage policy ID: `rugo.compositor_damage_policy.v1`.
- Parent surface lifecycle contract ID: `rugo.surface_lifecycle_contract.v1`.
- Parent display runtime contract ID: `rugo.display_runtime_contract.v1`.
- Runtime report schema: `rugo.window_system_runtime_report.v1`.
- Compositor damage schema: `rugo.compositor_damage_report.v1`.

## Declared composition policy

- Damage union policy: `bounding_union_per_output`.
- Opaque clip policy: `front_to_back_opaque_clip`.
- Full-scene reset policy: `full_output_on_scene_init`.
- Declared output count: `1`.
- Declared output identifier: `display-0`.

## Required checks

- `damage_region_union`:
  - bounding-union mismatches must remain `= 0`.
- `occlusion_clip_integrity`:
  - occlusion clip violations must remain `= 0`.
- `present_region_budget`:
  - present-region latency p95 must remain `<= 16.667 ms`.
- `retained_region_reuse`:
  - retained region reuse ratio must remain `>= 0.75`.
- `fullscreen_damage_reset`:
  - scene-init full-output reset coverage ratio must remain `>= 1.0`.

## Reporting requirements

- `out/compositor-damage-v1.json` must expose:
  - `policy`
  - `output`
  - `phases`
  - `clip_snapshots`
  - `present`
  - `source_reports`
  - `digest`
- Every damage phase must record:
  - `phase`
  - `target_window_id`
  - `damage_rects`
  - `union_rect`
  - `clipped_damage_rects`
- Damage evidence must remain tied to the corresponding
  `rugo.window_system_runtime_report.v1` digest before it can claim
  `gate_pass=true`.

## Release gating

- Local gate: `make test-window-system-v1`.
- Local sub-gate: `make test-compositor-damage-v1`.
- CI gate: `Window system v1 gate`.
- CI sub-gate: `Compositor damage v1 gate`.
