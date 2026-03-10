# M45 Execution Backlog (Modern Virtual Platform Parity v1)

Date: 2026-03-10  
Lane: Rugo (Rust kernel + Go user space)  
Status: proposed

## Goal

Close the gap between the current v5 transitional-VirtIO baseline and modern
virtual-platform defaults while binding display device evidence to desktop
qualification.

M45 source of truth remains:

- `docs/M45_M47_HARDWARE_EXPANSION_ROADMAP.md`
- `docs/hw/support_matrix_v6_plan.md`
- `docs/hw/support_matrix_v5.md`
- this backlog

## Current State Summary

- v5 is stable and release-blocking, but it is still centered around
  transitional VirtIO assumptions.
- Modern VirtIO classes and `virtio-gpu-pci` are not yet first-class matrix
  surfaces.
- Desktop qualification exists, but display-class evidence is not yet tied into
  hardware support claims.

## Execution Plan

- PR-1: contract freeze
- PR-2: tooling + deterministic campaigns
- PR-3: shadow-gate rollout + promotion criteria

## PR-1: Matrix v6 / Modern VirtIO Contract Freeze

### Objective

Define the contract for modern virtual-platform parity and explicit display
device evidence.

### Scope

- Add docs:
  - `docs/hw/support_matrix_v6.md`
  - `docs/hw/driver_lifecycle_contract_v6.md`
  - `docs/hw/virtio_platform_profile_v1.md`
- Extend docs:
  - `docs/desktop/display_stack_contract_v1.md`
- Add tests:
  - `tests/hw/test_hw_matrix_docs_v6.py`
  - `tests/hw/test_virtio_platform_profile_v1.py`

### Primary files

- `docs/hw/support_matrix_v6.md`
- `docs/hw/driver_lifecycle_contract_v6.md`
- `docs/hw/virtio_platform_profile_v1.md`
- `docs/desktop/display_stack_contract_v1.md`
- `tests/hw/test_hw_matrix_docs_v6.py`
- `tests/hw/test_virtio_platform_profile_v1.py`

### Acceptance checks

- `python -m pytest tests/hw/test_hw_matrix_docs_v6.py tests/hw/test_virtio_platform_profile_v1.py -v`

### Done criteria for PR-1

- Matrix v6 target classes and tier expectations are explicit and versioned.
- Modern VirtIO classes and `virtio-gpu-pci` are defined as contract surfaces,
  not implied support.
- Display-class requirements are tied to desktop evidence instead of a
  standalone hardware marker.

## PR-2: Modern VirtIO Campaigns + Desktop Display Bridge

### Objective

Implement deterministic campaigns for modern storage/network/CSI/GPU classes
and connect GPU/display evidence to the desktop smoke model.

### Scope

- Add tooling:
  - `tools/run_hw_matrix_v6.py`
- Extend tooling:
  - `tools/run_desktop_smoke_v1.py`
- Add tests:
  - `tests/hw/test_virtio_modern_storage_v1.py`
  - `tests/hw/test_virtio_modern_net_v1.py`
  - `tests/hw/test_virtio_scsi_v1.py`
  - `tests/hw/test_virtio_gpu_framebuffer_v1.py`
  - `tests/hw/test_driver_lifecycle_v6.py`
  - `tests/hw/test_hw_negative_paths_v5.py`
  - `tests/desktop/test_display_device_bridge_v1.py`

### Primary files

- `tools/run_hw_matrix_v6.py`
- `tools/run_desktop_smoke_v1.py`
- `tests/hw/test_virtio_modern_storage_v1.py`
- `tests/hw/test_virtio_modern_net_v1.py`
- `tests/hw/test_virtio_scsi_v1.py`
- `tests/hw/test_virtio_gpu_framebuffer_v1.py`
- `tests/hw/test_driver_lifecycle_v6.py`
- `tests/hw/test_hw_negative_paths_v5.py`
- `tests/desktop/test_display_device_bridge_v1.py`

### Acceptance checks

- `python tools/run_hw_matrix_v6.py --out out/hw-matrix-v6.json`
- `python tools/run_desktop_smoke_v1.py --out out/desktop-smoke-v1.json`
- `python -m pytest tests/hw/test_virtio_modern_storage_v1.py tests/hw/test_virtio_modern_net_v1.py tests/hw/test_virtio_scsi_v1.py tests/hw/test_virtio_gpu_framebuffer_v1.py tests/hw/test_driver_lifecycle_v6.py tests/hw/test_hw_negative_paths_v5.py tests/desktop/test_display_device_bridge_v1.py -v`

### Done criteria for PR-2

- Modern VirtIO storage/network parity is machine-readable and deterministic.
- `virtio-scsi-pci` and `virtio-gpu-pci` have explicit probe/init/runtime
  evidence.
- Negative-path behavior remains deterministic for missing/unsupported classes.
- Desktop smoke evidence can name the display device class used during
  qualification.

## PR-3: v6 Shadow Gate + Adoption Criteria

### Objective

Run v6 in parallel with v5, prove sustained parity, and define the criteria for
promoting v6 to the primary hardware gate.

### Scope

- Add local gates:
  - `Makefile` target `test-hw-matrix-v6`
  - `Makefile` target `test-virtio-platform-v1`
- Add CI steps:
  - `Hardware matrix v6 shadow gate`
  - `Virtio platform v1 shadow gate`
- Add aggregate tests:
  - `tests/hw/test_hw_gate_v6.py`
  - `tests/hw/test_virtio_platform_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/hw/test_hw_gate_v6.py`
- `tests/hw/test_virtio_platform_gate_v1.py`
- `README.md`

### Acceptance checks

- `make test-hw-matrix-v6`
- `make test-virtio-platform-v1`

### Done criteria for PR-3

- v6 runs in local and CI lanes as a shadow gate beside v5.
- Promotion from shadow gate to primary gate has explicit criteria:
  - minimum `14` consecutive green shadow runs,
  - zero v5 regressions,
  - zero fatal lifecycle errors,
  - desktop display bridge checks remain green,
  - both transitional and modern VirtIO profiles remain reproducible.
- No support claim broadens until the promotion criteria are satisfied.

## Non-goals for M45 backlog

- bare-metal NIC expansion (`e1000e`, `rtl8169`)
- USB input or removable-media work
- Wi-Fi, Bluetooth, audio, webcam, or power-management breadth
- replacing v5 before sustained v6 parity evidence exists

