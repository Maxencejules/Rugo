# M45-M47 Hardware Expansion Roadmap (Post-M44)

Date: 2026-03-10  
Lane: Rugo (Rust kernel + Go user space)  
Status: Proposed

## Why this document exists

M40-M44 closed the current general-purpose parity phase, but the hardware story
is still intentionally bounded:

1. v5 release confidence is strong, but still anchored around transitional
   VirtIO plus a narrow native-driver set.
2. Desktop qualification exists, but the hardware matrix does not yet bind
   display/input/removable-media classes into support claims.
3. Promotion from evidence-only hardware coverage to claimable support is still
   policy-light beyond the current v4/v5 promotion baseline.

This roadmap defines the next hardware-focused phase without diluting the
project's existing matrix discipline.

## Scope and boundaries

In scope:

- Start M45-M47 as the next hardware expansion phase.
- Keep v5 as the active release-blocking floor until replacement criteria are
  explicitly met.
- Expand hardware classes only through contract docs, deterministic campaigns,
  and auditable promotion rules.

Out of scope:

- declaring universal PC compatibility,
- broad Wi-Fi, Bluetooth, audio, webcam, or laptop power-management breadth,
- weakening gate thresholds to accelerate support claims.

## Sequencing map

| Milestone | Focus | Primary gate |
|---|---|---|
| M45 | Modern Virtual Platform Parity v1 | `test-hw-matrix-v6` |
| M46 | Bare-Metal I/O Baseline v1 | `test-baremetal-io-baseline-v1` |
| M47 | Hardware Claim Promotion Program v1 | `test-hw-claim-promotion-v1` |

### Cross-cutting sub-gates (required)

| Sub-gate | Anchored milestone | Focus |
|---|---|---|
| `test-virtio-platform-v1` | M45 | modern VirtIO class parity and display-device bridge evidence |
| `test-usb-input-removable-v1` | M46 | USB input/removable-media baseline bound to desktop and recovery contracts |
| `test-hw-support-tier-audit-v1` | M47 | auditable support-tier and promotion-policy enforcement |

## Suggested cadence

- Planning cadence: 1 milestone per 6-10 weeks.
- Each milestone follows the established 3-PR pattern:
  - PR-1: contract freeze,
  - PR-2: implementation and tooling,
  - PR-3: release-gate wiring and closure.

## M45: Modern Virtual Platform Parity v1

### Objective

Move the matrix from transitional-VirtIO dependence toward modern virtual
platform parity while tying display-class evidence to desktop qualification.

### PR-1 (contract freeze)

- Docs:
  - `docs/hw/support_matrix_v6.md`
  - `docs/hw/driver_lifecycle_contract_v6.md`
  - `docs/hw/virtio_platform_profile_v1.md`
  - extend `docs/desktop/display_stack_contract_v1.md`
- Tests:
  - `tests/hw/test_hw_matrix_docs_v6.py`
  - `tests/hw/test_virtio_platform_profile_v1.py`

### PR-2 (implementation + deterministic campaigns)

- Tooling:
  - `tools/run_hw_matrix_v6.py`
  - extend `tools/run_desktop_smoke_v1.py`
- Tests:
  - `tests/hw/test_virtio_modern_storage_v1.py`
  - `tests/hw/test_virtio_modern_net_v1.py`
  - `tests/hw/test_virtio_scsi_v1.py`
  - `tests/hw/test_virtio_gpu_framebuffer_v1.py`
  - `tests/hw/test_driver_lifecycle_v6.py`
  - `tests/hw/test_hw_negative_paths_v5.py`
  - `tests/desktop/test_display_device_bridge_v1.py`

### PR-3 (shadow gate + closure)

- Gates:
  - `test-hw-matrix-v6`
  - sub-gate `test-virtio-platform-v1`
- Aggregate tests:
  - `tests/hw/test_hw_gate_v6.py`
  - `tests/hw/test_virtio_platform_gate_v1.py`

### Done criteria

- Modern VirtIO storage/network parity is deterministic in Tier 0 and Tier 1.
- `virtio-scsi-pci` and `virtio-gpu-pci` are represented in machine-readable
  matrix artifacts.
- v6 runs as a shadow gate beside v5 without introducing regressions.

## M46: Bare-Metal I/O Baseline v1

### Objective

Add a practical bare-metal floor centered on common wired NICs, USB input, and
removable-media paths needed for desktop interaction and recovery workflows.

### PR-1 (contract freeze)

- Docs:
  - `docs/hw/baremetal_io_profile_v1.md`
  - `docs/hw/usb_input_removable_contract_v1.md`
  - extend `docs/hw/driver_lifecycle_contract_v6.md`
  - extend `docs/desktop/input_stack_contract_v1.md`
- Tests:
  - `tests/hw/test_baremetal_io_profile_v1.py`
  - `tests/hw/test_usb_input_removable_docs_v1.py`

### PR-2 (implementation + deterministic campaigns)

- Tooling:
  - `tools/run_baremetal_io_baseline_v1.py`
  - `tools/collect_hw_promotion_evidence_v2.py`
- Tests:
  - `tests/hw/test_e1000e_baseline_v1.py`
  - `tests/hw/test_rtl8169_baseline_v1.py`
  - `tests/hw/test_xhci_usb_hid_v1.py`
  - `tests/hw/test_usb_storage_v1.py`
  - `tests/hw/test_baremetal_io_recovery_v1.py`
  - `tests/desktop/test_usb_input_focus_delivery_v1.py`

### PR-3 (gate + closure)

- Gates:
  - `test-baremetal-io-baseline-v1`
  - sub-gate `test-usb-input-removable-v1`
- Aggregate tests:
  - `tests/hw/test_baremetal_io_gate_v1.py`
  - `tests/hw/test_usb_input_removable_gate_v1.py`

### Done criteria

- Common wired-NIC and USB/removable-media classes have deterministic evidence.
- Input and recovery-linked classes are tied to desktop/recovery contracts.
- At least one Tier 2 board profile can produce a promotion-grade evidence
  bundle without relaxing unsupported-boundary policy.

## M47: Hardware Claim Promotion Program v1

### Objective

Turn selected v6 classes from evidence-only coverage into auditable, support
tier claims without weakening the project's existing support-claim boundaries.

### PR-1 (contract freeze)

- Docs:
  - `docs/hw/support_claim_policy_v1.md`
  - `docs/hw/bare_metal_promotion_policy_v2.md`
  - `docs/hw/support_tier_audit_v1.md`
- Tests:
  - `tests/hw/test_support_claim_docs_v1.py`

### PR-2 (implementation + audits)

- Tooling:
  - `tools/run_hw_claim_promotion_v1.py`
  - `tools/run_hw_support_tier_audit_v1.py`
- Tests:
  - `tests/hw/test_hw_claim_promotion_v1.py`
  - `tests/hw/test_hw_support_tier_audit_v1.py`
  - `tests/hw/test_hw_promotion_regression_v1.py`
  - `tests/hw/test_hw_support_claim_negative_v1.py`

### PR-3 (gate + closure)

- Gates:
  - `test-hw-claim-promotion-v1`
  - sub-gate `test-hw-support-tier-audit-v1`
- Aggregate tests:
  - `tests/hw/test_hw_claim_promotion_gate_v1.py`
  - `tests/hw/test_hw_support_tier_gate_v1.py`

### Done criteria

- Promoted hardware classes are traceable to explicit support tiers and policy
  evidence.
- Unsupported classes remain explicit and machine-auditable.
- Promotion policy becomes release-blocking for any claimed class changes.

## Exit criteria for M45-M47 phase

The phase should be considered complete only when:

1. All M45-M47 primary gates are green in local and CI lanes.
2. Modern VirtIO parity is stable enough to be considered part of normal matrix
   expectations rather than one-off evidence.
3. Bare-metal I/O baseline classes have promotion-grade evidence on declared
   target profiles.
4. Hardware support claims can be audited by tier and promotion history, not
   just inferred from test presence.

