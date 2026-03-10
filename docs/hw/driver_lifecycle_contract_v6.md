# Driver Lifecycle Contract v6

Date: 2026-03-10  
Milestone: M45 Modern Virtual Platform Parity v1  
Lane: Rugo (Rust kernel + Go user space)  
Status: shadow gate contract

## Goal

Define deterministic v6 lifecycle semantics for transitional and modern VirtIO
profiles plus explicit SCSI and framebuffer-display coverage.

## Schema and report identity

Schema identifier: `rugo.driver_lifecycle_report.v6`

Required report fields:

- `driver`
- `device_class`
- `profile`
- `states_observed`
- `probe_attempts`
- `probe_successes`
- `init_failures`
- `runtime_errors`
- `irq_vector_bound`
- `irq_vector_retarget`
- `affinity_balance_events`
- `recoveries`
- `fatal_errors`
- `status`

## Deterministic lifecycle states

| State id | Meaning | Deterministic marker expectation |
|---|---|---|
| `probe_missing` | Device class not discovered | `BLK: not found`, `NET: not found`, `SCSI: not found`, or `GPU: not found` |
| `probe_found` | Device class discovered | device-specific ready marker emitted exactly once |
| `init_ready` | Driver init path complete | device-specific init marker emitted exactly once |
| `runtime_ok` | Runtime operation succeeds | `BLK: rw ok`, `NET: udp echo`, `SCSI: inquiry ok`, or `GPU: framebuffer ready` |
| `link_ready` | Wired NIC establishes the expected carrier state | `NET: link ready` |
| `irq_vector_bound` | IRQ vector bind completed | `IRQ: vector bound` |
| `irq_vector_retarget` | IRQ moved under policy | lifecycle report retarget counter increments |
| `cpu_affinity_balance` | IRQ affinity stays policy-balanced | `SMP: affinity balanced` |
| `reset_recover` | Reset and runtime restoration succeeds | recovery event count increments |
| `framebuffer_console_present` | GPU framebuffer can host the baseline console/shell surface | `GPU: framebuffer ready` |
| `display_scanout_ready` | Display scanout is stable enough for desktop qualification | `DISP: scanout stable` |
| `hid_ready` | USB HID devices enumerate and expose baseline keyboard/pointer paths | `USB: hid ready` |
| `focus_delivery_ready` | Focused desktop target receives USB input reliably | `USB: focus delivery ok` |
| `media_ready` | Removable USB media enumerates and mounts | `USBSTOR: media ready` |
| `recovery_media_bootstrap` | Removable media can enter the recovery workflow | `RECOVERY: removable media ready` |
| `error_recoverable` | Runtime error with successful recovery | recovery accounting increments |
| `error_fatal` | Non-recoverable error requiring escalation | gate must fail and emit reason |

## Contract rules

- Required transitional and modern storage/network drivers must observe
  `probe_found`, `init_ready`, `runtime_ok`, and `irq_vector_bound`.
- `virtio-scsi-pci` must observe deterministic probe/init/runtime evidence,
  including `SCSI: inquiry ok`.
- `virtio-gpu-pci` must additionally observe `framebuffer_console_present` and
  `display_scanout_ready`.
- `e1000e` and `rtl8169` must observe `probe_found`, `init_ready`,
  `link_ready`, `runtime_ok`, and `irq_vector_bound`.
- `xhci` plus `usb-hid` qualification runs must observe `probe_found`,
  `init_ready`, `hid_ready`, and `focus_delivery_ready`.
- `usb-storage` qualification runs must observe `probe_found`, `init_ready`,
  `media_ready`, and `recovery_media_bootstrap`.
- `probe_missing` markers are mandatory for negative-path checks.
- Any `error_fatal` state makes the v6 shadow gate fail.
- Display-class claims are bounded by
  `docs/desktop/display_stack_contract_v1.md` and require the desktop display
  bridge to pass.
- Lifecycle claims are bounded by `docs/hw/support_matrix_v6.md`.
- Bare-metal I/O lifecycle claims are additionally bounded by
  `docs/hw/baremetal_io_profile_v1.md` and
  `docs/hw/usb_input_removable_contract_v1.md`.

## Required device classes for v6 coverage

- Storage:
  - `virtio-blk-pci` transitional
  - `virtio-blk-pci` modern
  - `virtio-scsi-pci`
- Network:
  - `virtio-net-pci` transitional
  - `virtio-net-pci` modern
- Display:
  - `virtio-gpu-pci`

## Additional M46 bare-metal device classes

- Wired NIC:
  - `e1000e`
  - `rtl8169`
- USB input:
  - `xhci`
  - `usb-hid`
- Removable media:
  - `usb-storage`

Required negative-path markers for these classes:

- `NET: e1000e not found`
- `NET: rtl8169 not found`
- `USB: hid not found`
- `USBSTOR: not found`

## Cross references

- Matrix policy: `docs/hw/support_matrix_v6.md`
- VirtIO platform profile: `docs/hw/virtio_platform_profile_v1.md`
- Display contract: `docs/desktop/display_stack_contract_v1.md`
- Bare-metal I/O profile: `docs/hw/baremetal_io_profile_v1.md`
- USB/removable contract: `docs/hw/usb_input_removable_contract_v1.md`
