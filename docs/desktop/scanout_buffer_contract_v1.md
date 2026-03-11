# Scanout Buffer Contract v1

Date: 2026-03-11  
Milestone: M48 Display Runtime + Scanout v1  
Status: active release gate

## Objective

Define scanout buffer ownership, rotation, and capture-read semantics for the
first graphical runtime path.

## Contract identifiers

- Buffer contract ID: `rugo.scanout_buffer_contract.v1`.
- Parent runtime contract ID: `rugo.display_runtime_contract.v1`.
- Runtime report schema: `rugo.display_runtime_report.v1`.
- Frame capture schema: `rugo.display_frame_capture.v1`.

## Buffer model

- Minimum scanout buffers: `3`.
- Optional capture shadow buffers: `1`.
- Required pixel format: `xrgb8888`.
- Required memory layout fields:
  - `width`
  - `height`
  - `stride_bytes`
  - `buffer_bytes`
  - `scanout_buffer_count`
  - `capture_shadow_count`

## Ownership states

- `runtime_owned`: writable by the compositor/runtime only.
- `scanout_pending`: queued for the next present boundary; writes are forbidden.
- `display_owned`: currently visible on the active scanout path.
- `capture_read_only`: readable by frame export only; writes are forbidden.

## Ownership rules

- A buffer may not be both `runtime_owned` and `display_owned`.
- Capture reads must occur only from `capture_read_only`.
- Ownership handoff must complete within `1` frame interval.
- Partial writes visible to scanout are release-blocking.
- `buffer_ownership_integrity` and `scanout_buffer_depth` are required
  release-gate checks.

## Runtime reporting requirements

- `out/display-runtime-v1.json` must expose:
  - `buffer_strategy`
  - `scanout_buffer_count`
  - `capture_shadow_count`
  - `total_buffers`
  - `ownership_states`
  - `state_counts`
  - `integrity_pass`
- The capture manifest must record the runtime digest and active buffer
  contract ID before it can claim `capture_pass=true`.
