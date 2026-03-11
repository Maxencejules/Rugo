# M55 Execution Backlog (GPU Driver Baseline + Acceleration v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Add a bounded GPU driver lane with explicit kernel/userspace boundaries,
acceleration semantics, and deterministic fallback behavior for the desktop
stack.

M55 source of truth remains:

- `docs/POST_G2_EXTENDED_MILESTONES.md`
- `docs/M53_EXECUTION_BACKLOG.md`
- `docs/M48_EXECUTION_BACKLOG.md`
- this backlog

## Current State Summary

- M48-M52 established scanout, input, composition, GUI runtime, and shell
  workflows on fallback-friendly display paths.
- M53 defines the generic native-driver and firmware boundary needed for GPU
  work.
- There is no explicit v1 contract yet for GPU userspace ABI, acceleration
  semantics, or native GPU fallback policy revision.
- M55 must close those gaps before multi-monitor and productivity milestones
  depend on native GPU claims.

## Execution plan

- PR-1: GPU contract freeze
- PR-2: modeset and acceleration campaign baseline
- PR-3: GPU gate wiring + closure

## Execution status

- PR-1: pending
- PR-2: pending
- PR-3: pending

## PR-1: GPU Contract Freeze

### Objective

Define the kernel/userspace GPU boundary and fallback rules before broadening
desktop hardware claims.

### Scope

- Add docs:
  - `docs/hw/gpu_driver_contract_v1.md`
  - `docs/desktop/gpu_userspace_abi_v1.md`
  - `docs/desktop/gpu_fallback_policy_v2.md`
- Add tests:
  - `tests/hw/test_gpu_driver_docs_v1.py`
  - `tests/desktop/test_gpu_userspace_abi_v1.py`

### Primary files

- `docs/hw/gpu_driver_contract_v1.md`
- `docs/desktop/gpu_userspace_abi_v1.md`
- `docs/desktop/gpu_fallback_policy_v2.md`
- `tests/hw/test_gpu_driver_docs_v1.py`
- `tests/desktop/test_gpu_userspace_abi_v1.py`

### Acceptance checks

- `python -m pytest tests/hw/test_gpu_driver_docs_v1.py tests/desktop/test_gpu_userspace_abi_v1.py -v`

### Done criteria for PR-1

- GPU queue, fence, modeset, and fallback behavior are versioned and explicit.
- Desktop components can reference a stable GPU ABI rather than implicit
  framebuffer behavior.

## PR-2: Modeset and Acceleration Campaign Baseline

### Objective

Implement deterministic GPU evidence collection for probe, modeset, scanout,
and fallback semantics.

### Scope

- Add tooling:
  - `tools/run_gpu_accel_campaign_v1.py`
  - `tools/run_display_scanout_capture_v2.py`
- Add tests:
  - `tests/hw/test_gpu_probe_v1.py`
  - `tests/desktop/test_gpu_modeset_v1.py`
  - `tests/desktop/test_gpu_scanout_accel_v1.py`
  - `tests/desktop/test_gpu_fallback_v2.py`
  - `tests/hw/test_gpu_firmware_policy_v1.py`

### Primary files

- `tools/run_gpu_accel_campaign_v1.py`
- `tools/run_display_scanout_capture_v2.py`
- `tests/hw/test_gpu_probe_v1.py`
- `tests/desktop/test_gpu_modeset_v1.py`
- `tests/desktop/test_gpu_scanout_accel_v1.py`
- `tests/desktop/test_gpu_fallback_v2.py`
- `tests/hw/test_gpu_firmware_policy_v1.py`

### Acceptance checks

- `python tools/run_gpu_accel_campaign_v1.py --out out/gpu-accel-v1.json`
- `python tools/run_display_scanout_capture_v2.py --out out/gpu-scanout-v2.json`
- `python -m pytest tests/hw/test_gpu_probe_v1.py tests/desktop/test_gpu_modeset_v1.py tests/desktop/test_gpu_scanout_accel_v1.py tests/desktop/test_gpu_fallback_v2.py tests/hw/test_gpu_firmware_policy_v1.py -v`

### Done criteria for PR-2

- GPU evidence artifacts are deterministic and machine-readable.
- `GPU: probe ok`, `GPU: modeset ok`, and fallback markers stay stable across
  supported profiles.
- Firmware-bearing GPU paths remain policy-bounded and auditable.

## PR-3: GPU Gate + Fallback Sub-gate

### Objective

Make the GPU baseline enforceable before later desktop milestones depend on it.

### Scope

- Add local gates:
  - `Makefile` target `test-gpu-accel-v1`
  - `Makefile` target `test-gpu-fallback-v2`
- Add CI steps:
  - `GPU accel v1 gate`
  - `GPU fallback v2 gate`
- Add aggregate tests:
  - `tests/hw/test_gpu_gate_v1.py`
  - `tests/desktop/test_gpu_fallback_gate_v2.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/hw/test_gpu_gate_v1.py`
- `tests/desktop/test_gpu_fallback_gate_v2.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-gpu-accel-v1`
- `make test-gpu-fallback-v2`

### Done criteria for PR-3

- GPU and fallback sub-gates are required in local and CI release lanes.
- M55 can be marked done only with deterministic native GPU evidence and
  release-gated fallback behavior.

## Non-goals for M55 backlog

- full media/video acceleration breadth
- Wi-Fi or power-management implementation owned elsewhere
- claiming universal desktop GPU parity outside the declared contract scope
