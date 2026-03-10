# USB Input + Removable Contract v1

Date: 2026-03-10  
Milestone: M46 Bare-Metal I/O Baseline v1  
Lane: Rugo (Rust kernel + Go user space)  
Status: active qualification sub-gate

## Goal

Bind USB input and removable-media evidence to the desktop input and recovery
contracts instead of treating controller discovery as sufficient qualification.

## Contract identifiers

- Contract ID: `rugo.usb_input_removable_contract.v1`
- Parent profile ID: `rugo.baremetal_io_profile.v1`
- Desktop input contract ID: `rugo.input_stack_contract.v1`
- Recovery workflow ID: `rugo.recovery_workflow.v3`
- Baseline schema: `rugo.baremetal_io_baseline.v1`

## Required device classes

- `xhci`
- `usb-hid`
- `usb-storage`

## Required checks

- `xhci_enumeration`
  - controller enumeration success ratio must be `>= 1.0`
- `usb_keyboard_latency`
  - keyboard latency p95 must be `<= 12 ms`
- `usb_pointer_latency`
  - pointer latency p95 must be `<= 14 ms`
- `usb_focus_delivery`
  - focused-window delivery ratio must be `>= 0.995`
- `usb_repeat_consistency`
  - dropped USB input events must be `<= 2`
- `usb_storage_enumeration`
  - removable media must enumerate successfully exactly once per attach cycle
- `usb_storage_mount`
  - mount readiness latency must be `<= 400 ms`
- `recovery_media_bootstrap`
  - removable media must pass recovery entry and post-audit workflow checks

## Determinism and report binding

- Qualification reports must expose `input_class`, `removable_media_class`,
  `desktop_input_checks`, and `install_recovery_checks`.
- `desktop_input_checks` must bind USB qualification to named checks instead of
  a generic "input ready" marker.
- `install_recovery_checks` must be sourced from the recovery workflow and
  include `recovery_entry_validation` and `post_recovery_audit`.
- Negative-path markers must remain explicit:
  - `USB: hid not found`
  - `USBSTOR: not found`

## Gate binding

- Tooling runner: `tools/run_baremetal_io_baseline_v1.py`
- Local sub-gate: `make test-usb-input-removable-v1`
- CI sub-gate: `USB input removable v1 gate`

