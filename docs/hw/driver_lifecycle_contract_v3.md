# Driver Lifecycle Contract v3

Date: 2026-03-09  
Milestone: M23  
Lane: Rugo (Rust kernel + Go user space)  
Status: active contract

## Goal

Define deterministic v3 lifecycle semantics for storage and network drivers
across probe, init, runtime, suspend/resume, and hotplug paths.

## Schema and report identity

Schema identifier: `rugo.driver_lifecycle_report.v3`

Required report fields:

- `driver`
- `states_observed`
- `probe_attempts`
- `probe_successes`
- `init_failures`
- `runtime_errors`
- `recoveries`
- `status`

## Deterministic lifecycle states

| State id | Meaning | Deterministic marker expectation |
|---|---|---|
| `probe_missing` | Device class not discovered | `BLK: not found` or `NET: not found` |
| `probe_found` | Device class discovered | `BLK: found virtio-blk` or `NET: virtio-net ready` |
| `init_ready` | Driver init path complete | device-specific ready marker emitted exactly once |
| `runtime_ok` | Runtime operation succeeds | `BLK: rw ok` or `NET: udp echo` |
| `suspend_prepare` | Driver quiesce before suspend | lifecycle report state transition recorded |
| `resume_ok` | Driver restore after resume | lifecycle report state transition recorded |
| `hotplug_add` | Device add event handled | lifecycle report add event recorded |
| `hotplug_remove` | Device remove event handled | lifecycle report remove event recorded |
| `error_recoverable` | Runtime error with successful recovery | recovery event count increments |
| `error_fatal` | Non-recoverable error requiring escalation | gate must fail and emit reason |

## Contract rules

- Tier 0 and Tier 1 must observe `probe_found`, `init_ready`, and `runtime_ok`
  for required drivers.
- Suspend/resume and hotplug states are mandatory in v3 lifecycle evidence.
- Any `error_fatal` state makes the matrix gate fail.
- Runtime errors are allowed only when paired with `error_recoverable` and
  explicit recovery accounting.
- Lifecycle claims are bounded by `docs/hw/support_matrix_v3.md`.

## Required driver classes

- Storage: `virtio-blk-pci`
- Network: `virtio-net-pci`

## Cross references

- Matrix policy: `docs/hw/support_matrix_v3.md`
- Firmware resiliency policy: `docs/hw/firmware_resiliency_policy_v1.md`
- Measured boot attestation: `docs/security/measured_boot_attestation_v1.md`
