# Display Runtime Contract v1

Date: 2026-03-11  
Milestone: M48 Display Runtime + Scanout v1  
Status: active release gate

## Objective

Define the first real pixel-producing display runtime for the declared graphical
path, including present timing, frame capture, and required fallback behavior.

## Contract identifiers

- Display runtime contract ID: `rugo.display_runtime_contract.v1`.
- Parent display stack contract ID: `rugo.display_stack_contract.v1`.
- Runtime report schema: `rugo.display_runtime_report.v1`.
- Frame capture schema: `rugo.display_frame_capture.v1`.
- Scanout buffer contract ID: `rugo.scanout_buffer_contract.v1`.
- Fallback policy ID: `rugo.gpu_fallback_policy.v1`.

## Declared runtime paths

- Primary runtime path:
  - display class: `virtio-gpu-pci`
  - runtime driver: `virtio_gpu_scanout`
  - declared-support source schema: `rugo.hw_matrix_evidence.v6`
- Fallback runtime path:
  - display class: `framebuffer-console`
  - fallback driver: `efifb`
  - declared-support source schema: `rugo.baremetal_io_baseline.v1`

## Required checks

- `virtio_gpu_scanout`:
  - the primary runtime must present frames without runtime errors.
  - threshold: runtime errors `= 0`.
- `virtio_present_cadence`:
  - the primary runtime must sustain bounded scanout stability.
  - threshold: frame-drop ratio `<= 0.005`.
- `buffer_ownership_integrity`:
  - scanout buffers must not be visible while partially written.
  - threshold: partial-write visibility ratio `= 0.0`.
- `scanout_buffer_depth`:
  - the runtime must expose at least triple-buffer scanout depth.
  - threshold: scanout buffer depth `>= 3`.
- `present_timing_budget`:
  - present timing must stay within the declared first-frame/runtime budget.
  - threshold: present latency p95 `<= 16.667 ms`.
- `present_jitter_budget`:
  - vblank jitter must remain bounded for deterministic cadence evidence.
  - threshold: vblank jitter p95 `<= 1.5 ms`.
- `frame_capture_ready`:
  - the active runtime path must export a deterministic frame capture.
  - threshold: capture export ratio `>= 1.0`.
- `efifb_fallback_activation`:
  - the fallback path must activate within a bounded handoff window.
  - threshold: fallback activation latency `<= 80 ms`.
- `efifb_fallback_scanout`:
  - the fallback path must still present stable frames once selected.
  - threshold: fallback frame-drop ratio `<= 0.01`.

## Determinism rules

- All checks must be represented as machine-readable entries in
  `out/display-runtime-v1.json`.
- The runtime report must record the selected `active_runtime_path`,
  `policy_decision`, `digest`, and artifact references.
- Frame capture output must be exported to `out/display-frame-v1.png` and a
  machine-readable manifest adjacent to the PNG.
- Capture evidence must remain tied to the runtime report digest and active
  runtime path.

## Release gating

- Local gate: `make test-display-runtime-v1`.
- Local sub-gate: `make test-scanout-path-v1`.
- CI gate: `Display runtime v1 gate`.
- CI sub-gate: `Scanout path v1 gate`.
