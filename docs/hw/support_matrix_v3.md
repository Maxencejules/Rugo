# Hardware Support Matrix v3

Date: 2026-03-09  
Milestone: M23  
Lane: Rugo (Rust kernel + Go user space)  
Status: active release gate

## Goal

Expand matrix confidence from v2 to v3 by adding deterministic driver lifecycle,
suspend/resume baseline coverage, and firmware attestation sub-gate evidence.

## Tier definitions and pass criteria

| Tier | Target class | Minimum pass criteria | Gate policy |
|---|---|---|---|
| Tier 0 | QEMU reference (`q35`) | Storage/network smoke, driver lifecycle checks, suspend/resume baseline, hotplug baseline | Release-blocking in local and CI |
| Tier 1 | QEMU compatibility (`pc`/i440fx) | Same checks as Tier 0 with marker parity and zero lifecycle regressions | Release-blocking in local and CI |
| Tier 2 | Bare-metal candidate boards | v3 diagnostics artifact + firmware attestation evidence + promotion runbook thresholds | Manual promotion only |
| Tier 3 | Exploratory hardware profiles | Evidence-only bringup notes | Never used for release support claims |

### Tier policy details

- Tier 0 and Tier 1 must pass `make test-hw-matrix-v3` with zero failing tests.
- Firmware sub-gate is mandatory for v3 promotion:
  `make test-firmware-attestation-v1`.
- Tier 2 requires repeated evidence: at least 10 consecutive green matrix runs and
  measured-boot policy pass evidence.
- Tier 3 remains non-claiming exploratory coverage only.

## Matrix targets (v3)

| Tier | Machine profile | Storage profile | Network profile | Lifecycle addenda | Expected outcome |
|---|---|---|---|---|---|
| Tier 0 | `-machine q35` | `virtio-blk-pci` transitional (`disable-modern=on`) | `virtio-net-pci` transitional (`disable-modern=on`) | suspend/resume + hotplug baseline | Deterministic pass |
| Tier 1 | `-machine pc` (`i440fx`) | `virtio-blk-pci` transitional (`disable-modern=on`) | `virtio-net-pci` transitional (`disable-modern=on`) | suspend/resume + hotplug baseline | Deterministic pass |

## Evidence artifact schema (v3)

Schema identifier: `rugo.hw_matrix_evidence.v3`

Required top-level fields:

- `schema`
- `created_utc`
- `matrix_contract_id`
- `driver_contract_id`
- `seed`
- `gate`
- `tier_results`
- `suspend_resume`
- `hotplug_baseline`
- `driver_lifecycle`
- `artifact_refs`
- `gate_pass`

Required `tier_results[]` fields:

- `tier`
- `machine`
- `storage_smoke`
- `network_smoke`
- `driver_lifecycle`
- `suspend_resume`
- `hotplug_baseline`
- `status`

Required `suspend_resume` fields:

- `cycles_target`
- `cycles_completed`
- `suspend_failures`
- `resume_timeouts`
- `max_resume_latency_ms`
- `resume_latency_budget_ms`
- `status`

Required `hotplug_baseline` fields:

- `events_target`
- `events_completed`
- `failures`
- `max_settle_ms`
- `settle_budget_ms`
- `status`

Required `artifact_refs` fields:

- `junit`: path to `out/pytest-hw-matrix-v3.xml`
- `diagnostics`: path to `out/hw-diagnostics-v3.json`
- `firmware_report`: path to `out/measured-boot-v1.json`
- `ci_artifact`: `hw-matrix-v3-artifacts`
- `firmware_ci_artifact`: `firmware-attestation-v1-artifacts`

## Executable conformance suite

- `tests/hw/test_hardware_matrix_v3.py`
- `tests/hw/test_driver_lifecycle_v3.py`
- `tests/hw/test_suspend_resume_v1.py`
- `tests/hw/test_hotplug_baseline_v1.py`
- `tests/hw/test_hw_gate_v3.py`

## Gate binding

- Local gate: `make test-hw-matrix-v3`
- Firmware sub-gate: `make test-firmware-attestation-v1`
- CI gate: `.github/workflows/ci.yml` step `Hardware matrix v3 gate`
- CI firmware sub-gate: `.github/workflows/ci.yml` step `Firmware attestation v1 gate`

## Hardware claims boundary

- Hardware support claims are bounded to matrix evidence only.
- A target without current v3 matrix and firmware evidence is unsupported for
  release claims.
- Tier labels are versioned policy contracts and must be updated through v3
  docs/tests before behavior changes are accepted.
