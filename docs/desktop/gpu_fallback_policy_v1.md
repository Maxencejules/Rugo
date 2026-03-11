# GPU Fallback Policy v1

Date: 2026-03-11  
Milestone: M48 Display Runtime + Scanout v1  
Status: active release gate

## Objective

Define when the display runtime may leave the primary `virtio-gpu-pci` path and
how it must preserve scanout and frame-capture evidence when falling back to
`framebuffer-console`.

## Policy identifiers

- Fallback policy ID: `rugo.gpu_fallback_policy.v1`.
- Parent runtime contract ID: `rugo.display_runtime_contract.v1`.
- Runtime report schema: `rugo.display_runtime_report.v1`.

## Ordered path selection

1. Prefer `virtio-gpu-pci` with runtime driver `virtio_gpu_scanout`.
2. Permit `force_fallback` to select `framebuffer-console` with `efifb`.
3. Permit automatic fallback only when the primary path cannot satisfy the
   runtime contract.
4. Never claim accelerated 3D, universal GPU coverage, or multi-monitor parity.

## Fallback obligations

- `efifb_fallback_activation` must remain `<= 80 ms`.
- `efifb_fallback_scanout` must remain `<= 0.01` frame-drop ratio.
- Frame capture export remains required after fallback.
- `policy_decision` must be one of:
  - `primary`
  - `forced_fallback`
  - `auto_fallback`
- Automatic fallback does not waive primary-path regressions in the release
  gate; declared `virtio-gpu-pci` support must still pass.

## Audit and gate anchors

- The runtime report must expose `active_runtime_path`, `fallback_ready`, and
  `fallback_activation_ms`.
- Local gate: `make test-display-runtime-v1`.
- Local sub-gate: `make test-scanout-path-v1`.
- CI gate: `Display runtime v1 gate`.
- CI sub-gate: `Scanout path v1 gate`.
