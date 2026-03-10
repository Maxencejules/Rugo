# Bare-Metal I/O Profile v1

Date: 2026-03-10  
Milestone: M46 Bare-Metal I/O Baseline v1  
Lane: Rugo (Rust kernel + Go user space)  
Status: active qualification gate

## Goal

Define the minimum bare-metal I/O floor for common wired NICs, USB input, and
removable-media workflows that are required for practical desktop interaction
and recovery.

## Profile identifiers

- Profile identifier: `rugo.baremetal_io_profile.v1`
- Report schema: `rugo.baremetal_io_baseline.v1`
- Driver lifecycle contract ID: `rugo.driver_lifecycle_report.v6`
- USB/removable contract ID: `rugo.usb_input_removable_contract.v1`
- Desktop input contract ID: `rugo.input_stack_contract.v1`
- Recovery workflow ID: `rugo.recovery_workflow.v3`

## Tier policy

| Tier | Target class | Minimum pass criteria | Gate policy |
|---|---|---|---|
| Tier 2 | Bare-metal qualification boards | One declared wired NIC class plus `xhci`, `usb-hid`, and `usb-storage` all pass without manual exception handling | Eligible for M46 qualification |
| Tier 3 | Bare-metal breadth candidates | Deterministic evidence only; unsupported classes stay explicit | Never claimable by this profile alone |

### Declared M46 device-class floor

| Device class | Domain | Requirement |
|---|---|---|
| `e1000e` | wired NIC | required evidence class |
| `rtl8169` | wired NIC | required evidence class |
| `xhci` | USB host controller | required evidence class |
| `usb-hid` keyboard | desktop input | required evidence class |
| `usb-hid` mouse | desktop input | required evidence class |
| `usb-storage` | removable media | required evidence class |

## Required report fields

The `rugo.baremetal_io_baseline.v1` report must include:

- `schema`
- `created_utc`
- `profile_id`
- `driver_contract_id`
- `usb_input_removable_contract_id`
- `input_contract_id`
- `recovery_workflow_id`
- `seed`
- `gate`
- `checks`
- `summary`
- `tier2_profiles`
- `device_class_coverage`
- `driver_lifecycle`
- `wired_nic`
- `usb_input`
- `removable_media`
- `desktop_input_checks`
- `install_recovery_checks`
- `negative_paths`
- `artifact_refs`
- `total_failures`
- `gate_pass`
- `digest`

## Qualification rules

- Baseline pass requires `total_failures = 0`.
- `desktop_input_checks` must name `input_class = usb-hid` for M46-bound runs.
- `install_recovery_checks` must be sourced from the recovery workflow and stay
  green for removable-media qualification.
- At least one Tier 2 profile must report `manual_exception_required = false`
  and `status = pass`.
- Unsupported or unstable hardware classes remain explicit and non-claiming.

## Reference Tier 2 profiles

- `intel_q470_e1000e_xhci`
- `amd_b550_rtl8169_xhci`

These profiles are qualification examples only. M46 does not claim universal PC
compatibility.

## Gate binding

- Local gate: `make test-baremetal-io-baseline-v1`.
- Local sub-gate: `make test-usb-input-removable-v1`.
- CI gate: `Bare-metal io baseline v1 gate`.
- CI sub-gate: `USB input removable v1 gate`.

