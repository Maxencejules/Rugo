# M46 Execution Backlog (Bare-Metal I/O Baseline v1)

Date: 2026-03-10  
Lane: Rugo (Rust kernel + Go user space)  
Status: done

## Goal

Add a practical bare-metal floor around common wired NICs, USB input, and
removable-media paths needed for real desktop interaction and recovery flows.

M46 source of truth remains:

- `docs/M45_M47_HARDWARE_EXPANSION_ROADMAP.md`
- `docs/hw/support_matrix_v6_plan.md`
- this backlog

## Current State Summary

- Bare-metal I/O scope is now explicit through a dedicated profile and USB
  input/removable-media child contract.
- Deterministic evidence now exists for `e1000e`, `rtl8169`, `xhci` +
  `usb-hid`, and `usb-storage`.
- USB input evidence is now bound to desktop input checks, and removable-media
  evidence is now bound to recovery workflow checks.

## Execution Result

- PR-1: complete (2026-03-10)
- PR-2: complete (2026-03-10)
- PR-3: complete (2026-03-10)

## Historical Rugo implementation summary

### Historical Rust kernel surface

- `kernel_rs/src/`: bare-metal NIC, USB input, and removable-media driver
  behavior that established the first practical non-virtio I/O floor.
- `arch/` and `boot/`: interrupt, bus-init, and device-discovery plumbing
  needed to make wired NIC, USB HID, and removable-media paths deterministic.

### Historical Go user space surface

- `services/go/`: desktop-facing input and recovery workflows that consumed the
  new bare-metal I/O floor once it existed.
- `services/go_std/`: not the primary path for this milestone.

### Historical Language-Native Verification

- `make kernel`
- `make userspace`
- `make image-demo`
- `make smoke-demo`
- `make test-baremetal-io-baseline-v1`
- `make test-usb-input-removable-v1`

## PR-1: Bare-Metal I/O Contract Freeze

### Objective

Define the bare-metal I/O floor for wired NICs, USB input, and removable media.

### Scope

- Add docs:
  - `docs/hw/baremetal_io_profile_v1.md`
  - `docs/hw/usb_input_removable_contract_v1.md`
- Extend docs:
  - `docs/hw/driver_lifecycle_contract_v6.md`
  - `docs/desktop/input_stack_contract_v1.md`
- Add tests:
  - `tests/hw/test_baremetal_io_profile_v1.py`
  - `tests/hw/test_usb_input_removable_docs_v1.py`

### Primary files

- `docs/hw/baremetal_io_profile_v1.md`
- `docs/hw/usb_input_removable_contract_v1.md`
- `docs/hw/driver_lifecycle_contract_v6.md`
- `docs/desktop/input_stack_contract_v1.md`
- `tests/hw/test_baremetal_io_profile_v1.py`
- `tests/hw/test_usb_input_removable_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/hw/test_baremetal_io_profile_v1.py tests/hw/test_usb_input_removable_docs_v1.py -v`

### Done criteria for PR-1

- Bare-metal I/O scope is explicit and versioned.
- Wired-NIC, USB input, and removable-media claims are bounded to specific
  device classes and thresholds.

### PR-1 completion summary

- Added contract docs:
  - `docs/hw/baremetal_io_profile_v1.md`
  - `docs/hw/usb_input_removable_contract_v1.md`
- Extended shared contracts:
  - `docs/hw/driver_lifecycle_contract_v6.md`
  - `docs/desktop/input_stack_contract_v1.md`
- Added executable doc checks:
  - `tests/hw/test_baremetal_io_profile_v1.py`
  - `tests/hw/test_usb_input_removable_docs_v1.py`

## PR-2: Bare-Metal I/O Campaigns

### Objective

Implement deterministic campaigns for wired-NIC, USB-input, and removable-media
classes and connect them to desktop/recovery evidence.

### Scope

- Add tooling:
  - `tools/run_baremetal_io_baseline_v1.py`
  - `tools/collect_hw_promotion_evidence_v2.py`
- Add tests:
  - `tests/hw/test_e1000e_baseline_v1.py`
  - `tests/hw/test_rtl8169_baseline_v1.py`
  - `tests/hw/test_xhci_usb_hid_v1.py`
  - `tests/hw/test_usb_storage_v1.py`
  - `tests/hw/test_baremetal_io_recovery_v1.py`
  - `tests/desktop/test_usb_input_focus_delivery_v1.py`

### Primary files

- `tools/run_baremetal_io_baseline_v1.py`
- `tools/collect_hw_promotion_evidence_v2.py`
- `tests/hw/test_e1000e_baseline_v1.py`
- `tests/hw/test_rtl8169_baseline_v1.py`
- `tests/hw/test_xhci_usb_hid_v1.py`
- `tests/hw/test_usb_storage_v1.py`
- `tests/hw/test_baremetal_io_recovery_v1.py`
- `tests/desktop/test_usb_input_focus_delivery_v1.py`

### Acceptance checks

- `python tools/run_baremetal_io_baseline_v1.py --out out/baremetal-io-v1.json`
- `python tools/collect_hw_promotion_evidence_v2.py --out out/hw-promotion-v2.json`
- `python -m pytest tests/hw/test_e1000e_baseline_v1.py tests/hw/test_rtl8169_baseline_v1.py tests/hw/test_xhci_usb_hid_v1.py tests/hw/test_usb_storage_v1.py tests/hw/test_baremetal_io_recovery_v1.py tests/desktop/test_usb_input_focus_delivery_v1.py -v`

### Done criteria for PR-2

- Common wired-NIC and USB/removable-media classes have deterministic evidence.
- Input classes satisfy declared desktop latency and reliability thresholds.
- Removable-media paths are validated against installer/recovery workflows.

### PR-2 completion summary

- Added deterministic bare-metal I/O tooling:
  - `tools/run_baremetal_io_baseline_v1.py`
  - `tools/collect_hw_promotion_evidence_v2.py`
- Extended desktop smoke evidence with named input-device bridge fields:
  - `tools/run_desktop_smoke_v1.py`
- Added executable class checks:
  - `tests/hw/test_e1000e_baseline_v1.py`
  - `tests/hw/test_rtl8169_baseline_v1.py`
  - `tests/hw/test_xhci_usb_hid_v1.py`
  - `tests/hw/test_usb_storage_v1.py`
  - `tests/hw/test_baremetal_io_recovery_v1.py`
  - `tests/desktop/test_usb_input_focus_delivery_v1.py`

## PR-3: Bare-Metal I/O Gate + USB/Removable Sub-gate

### Objective

Make the bare-metal I/O baseline executable as a qualification lane and define
its Tier 2 promotion floor.

### Scope

- Add local gates:
  - `Makefile` target `test-baremetal-io-baseline-v1`
  - `Makefile` target `test-usb-input-removable-v1`
- Add CI steps:
  - `Bare-metal io baseline v1 gate`
  - `USB input removable v1 gate`
- Add aggregate tests:
  - `tests/hw/test_baremetal_io_gate_v1.py`
  - `tests/hw/test_usb_input_removable_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/hw/test_baremetal_io_gate_v1.py`
- `tests/hw/test_usb_input_removable_gate_v1.py`
- `README.md`

### Acceptance checks

- `make test-baremetal-io-baseline-v1`
- `make test-usb-input-removable-v1`

### Done criteria for PR-3

- Bare-metal I/O and USB/removable-media sub-gates are available in local and
  CI qualification lanes.
- At least one Tier 2 board profile can satisfy the baseline without manual
  exception handling.
- Unsupported or unstable classes remain explicit and non-claiming.

### PR-3 completion summary

- Added aggregate gate checks:
  - `tests/hw/test_baremetal_io_gate_v1.py`
  - `tests/hw/test_usb_input_removable_gate_v1.py`
- Added local gates:
  - `make test-baremetal-io-baseline-v1`
  - `make test-usb-input-removable-v1`
- Added CI qualification gates and artifacts:
  - `Bare-metal io baseline v1 gate`
  - `USB input removable v1 gate`
- Updated closure docs:
  - `MILESTONES.md`
  - `docs/STATUS.md`
  - `README.md`

## X2 runtime-backed closure addendum (2026-03-18)

- This backlog is now governed by the shared x2 hardware runtime qualification lane.
- The bare-metal board profiles now feed the shared x2 hardware runtime
  qualification lane as named Tier 2 target classes.
- `make test-x2-hardware-runtime-v1` binds wired NIC, USB input, removable
  media, and recovery evidence to the same runtime-backed target report.
- Additional bare-metal classes must join the shared X2 target registry before
  they can be treated as supportable.

## Non-goals for M46 backlog

- Wi-Fi, Bluetooth, or audio breadth
- discrete GPU acceleration
- broad laptop-specific peripheral support
- converting all Tier 2 profiles to release claims in this milestone
