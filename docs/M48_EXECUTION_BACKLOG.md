# M48 Execution Backlog (Display Runtime + Scanout v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: proposed

## Goal

Implement the first real graphical output path for Rugo by turning display
qualification from contract/report evidence into live scanout, present timing,
and frame-capture behavior on declared devices.

M48 source of truth remains `docs/M48_M52_GUI_IMPLEMENTATION_ROADMAP.md`,
`MILESTONES.md`, and this backlog.

## Current State Summary

- Bootable image generation and QEMU bring-up are already stable.
- Desktop display contracts and hardware display-class evidence exist from M35,
  M44, and M45.
- The repo still lacks an in-tree display runtime that produces real graphical
  frames and capture artifacts.

## Execution plan

- PR-1: display runtime contract freeze
- PR-2: scanout/runtime implementation + capture campaigns
- PR-3: release gate wiring + closure

## Execution Result

- PR-1: not started
- PR-2: not started
- PR-3: not started

## PR-1: Display Runtime Contract Freeze

### Objective

Define the real scanout/runtime contract, buffer ownership rules, and fallback
policy required before any display implementation work lands.

### Scope

- Add docs:
  - `docs/desktop/display_runtime_contract_v1.md`
  - `docs/desktop/scanout_buffer_contract_v1.md`
  - `docs/desktop/gpu_fallback_policy_v1.md`
- Add tests:
  - `tests/desktop/test_display_runtime_docs_v1.py`

### Primary files

- `docs/desktop/display_runtime_contract_v1.md`
- `docs/desktop/scanout_buffer_contract_v1.md`
- `docs/desktop/gpu_fallback_policy_v1.md`
- `tests/desktop/test_display_runtime_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/desktop/test_display_runtime_docs_v1.py -v`

### Done criteria for PR-1

- Real display-runtime boundaries are explicit and versioned.
- Scanout buffer ownership, timing, and fallback rules are reviewable before
  implementation starts.

## PR-2: Scanout Runtime + Capture Campaigns

### Objective

Implement the live display path and produce deterministic frame/timing evidence
for declared graphical devices.

### Scope

- Add tooling:
  - `tools/run_display_runtime_v1.py`
  - `tools/capture_display_frame_v1.py`
- Add tests:
  - `tests/desktop/test_virtio_gpu_scanout_v1.py`
  - `tests/desktop/test_efifb_fallback_v1.py`
  - `tests/desktop/test_display_present_timing_v1.py`
  - `tests/desktop/test_display_frame_capture_v1.py`

### Primary files

- `tools/run_display_runtime_v1.py`
- `tools/capture_display_frame_v1.py`
- `tests/desktop/test_virtio_gpu_scanout_v1.py`
- `tests/desktop/test_efifb_fallback_v1.py`
- `tests/desktop/test_display_present_timing_v1.py`
- `tests/desktop/test_display_frame_capture_v1.py`

### Acceptance checks

- `python tools/run_display_runtime_v1.py --out out/display-runtime-v1.json`
- `python tools/capture_display_frame_v1.py --out out/display-frame-v1.png`
- `python -m pytest tests/desktop/test_virtio_gpu_scanout_v1.py tests/desktop/test_efifb_fallback_v1.py tests/desktop/test_display_present_timing_v1.py tests/desktop/test_display_frame_capture_v1.py -v`

### Done criteria for PR-2

- Real display runtime artifacts are deterministic and machine-readable.
- Declared graphical devices can present frames and export capture evidence.

## PR-3: Display Gate + Scanout Sub-gate

### Objective

Make live display runtime behavior release-blocking and replace serial-only
graphical evidence in the declared path.

### Scope

- Add local gates:
  - `Makefile` target `test-display-runtime-v1`
  - `Makefile` target `test-scanout-path-v1`
- Add CI steps:
  - `Display runtime v1 gate`
  - `Scanout path v1 gate`
- Add aggregate tests:
  - `tests/desktop/test_display_runtime_gate_v1.py`
  - `tests/desktop/test_scanout_path_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/desktop/test_display_runtime_gate_v1.py`
- `tests/desktop/test_scanout_path_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-display-runtime-v1`
- `make test-scanout-path-v1`

### Done criteria for PR-3

- Real display runtime and scanout behavior are release-blocking in local and CI
  lanes.
- Frame output evidence is tied to explicit runtime gates and artifacts.

## Non-goals for M48 backlog

- accelerated 3D, GPU compute, or universal graphics driver breadth
- multi-monitor support, HDR, or advanced color-management policy
- broad desktop-shell or app-runtime behavior beyond first-frame output
