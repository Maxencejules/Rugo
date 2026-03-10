# M46 Execution Backlog (Bare-Metal I/O Baseline v1)

Date: 2026-03-10  
Lane: Rugo (Rust kernel + Go user space)  
Status: proposed

## Goal

Add a practical bare-metal floor around common wired NICs, USB input, and
removable-media paths needed for real desktop interaction and recovery flows.

M46 source of truth remains:

- `docs/M45_M47_HARDWARE_EXPANSION_ROADMAP.md`
- `docs/hw/support_matrix_v6_plan.md`
- this backlog

## Current State Summary

- Native hardware confidence exists for a limited set of storage/NIC classes,
  but common wired desktop NICs are still outside the main support path.
- Desktop baseline contracts exist, but they are not yet tied to explicit USB
  input device classes.
- Recovery and installer workflows are stronger on policy than on removable
  media hardware coverage.

## Execution Plan

- PR-1: contract freeze
- PR-2: tooling + deterministic campaigns
- PR-3: gate wiring + Tier 2 qualification baseline

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

## Non-goals for M46 backlog

- Wi-Fi, Bluetooth, or audio breadth
- discrete GPU acceleration
- broad laptop-specific peripheral support
- converting all Tier 2 profiles to release claims in this milestone

