# M37 Execution Backlog (Hardware Breadth + Driver Matrix v4)

Date: 2026-03-09  
Lane: Rugo (Rust kernel + Go user space)  
Status: done

## Goal

Expand hardware confidence from current matrix focus to broader device classes
and repeatable bare-metal promotion evidence.

M37 source of truth remains `docs/M35_M39_GENERAL_PURPOSE_EXPANSION_ROADMAP.md`,
`MILESTONES.md`, and this backlog.

## Current State Summary

- Hardware v3 gates are strong but release-blocking tiers remain QEMU-focused.
- Bare-metal evidence remains promotion-gated and policy-bounded.
- M37 introduces matrix v4 and explicit bare-metal promotion controls.

## Execution Result

- PR-1: complete (2026-03-09)
- PR-2: complete (2026-03-09)
- PR-3: complete (2026-03-09)

## PR-1: Matrix v4 Contract Freeze

### Objective

Define hardware matrix v4 and driver lifecycle expectations for broader coverage.

### Scope

- Add docs:
  - `docs/hw/support_matrix_v4.md`
  - `docs/hw/driver_lifecycle_contract_v4.md`
  - `docs/hw/bare_metal_promotion_policy_v1.md`
- Add tests:
  - `tests/hw/test_hw_matrix_docs_v4.py`

### Primary files

- `docs/hw/support_matrix_v4.md`
- `docs/hw/driver_lifecycle_contract_v4.md`
- `docs/hw/bare_metal_promotion_policy_v1.md`
- `tests/hw/test_hw_matrix_docs_v4.py`

### Acceptance checks

- `python -m pytest tests/hw/test_hw_matrix_docs_v4.py -v`

### Done criteria for PR-1

- Matrix v4 tiers and promotion policy are explicit, versioned, and test-backed.

### PR-1 completion summary

- Added matrix v4 contract docs:
  - `docs/hw/support_matrix_v4.md`
  - `docs/hw/driver_lifecycle_contract_v4.md`
  - `docs/hw/bare_metal_promotion_policy_v1.md`
- Added executable contract checks:
  - `tests/hw/test_hw_matrix_docs_v4.py`

## PR-2: Matrix v4 Diagnostics + Campaigns

### Objective

Implement deterministic matrix v4 checks and bare-metal promotion evidence flow.

### Scope

- Add tooling:
  - `tools/run_hw_matrix_v4.py`
  - `tools/collect_hw_promotion_evidence_v1.py`
- Add tests:
  - `tests/hw/test_hw_matrix_v4.py`
  - `tests/hw/test_driver_lifecycle_v4.py`
  - `tests/hw/test_baremetal_promotion_v1.py`
  - `tests/hw/test_hw_negative_paths_v4.py`

### Primary files

- `tools/run_hw_matrix_v4.py`
- `tools/collect_hw_promotion_evidence_v1.py`
- `tests/hw/test_hw_matrix_v4.py`
- `tests/hw/test_driver_lifecycle_v4.py`
- `tests/hw/test_baremetal_promotion_v1.py`
- `tests/hw/test_hw_negative_paths_v4.py`

### Acceptance checks

- `python tools/run_hw_matrix_v4.py --out out/hw-matrix-v4.json`
- `python tools/collect_hw_promotion_evidence_v1.py --out out/hw-promotion-v1.json`
- `python -m pytest tests/hw/test_hw_matrix_v4.py tests/hw/test_driver_lifecycle_v4.py tests/hw/test_baremetal_promotion_v1.py tests/hw/test_hw_negative_paths_v4.py -v`

### Done criteria for PR-2

- Matrix v4 and promotion artifacts are deterministic and machine-readable.
- Driver lifecycle and negative-path behavior are executable and auditable.

### PR-2 completion summary

- Added deterministic matrix and promotion tooling:
  - `tools/run_hw_matrix_v4.py`
  - `tools/collect_hw_promotion_evidence_v1.py`
- Added executable matrix/promotion checks:
  - `tests/hw/test_hw_matrix_v4.py`
  - `tests/hw/test_driver_lifecycle_v4.py`
  - `tests/hw/test_baremetal_promotion_v1.py`
  - `tests/hw/test_hw_negative_paths_v4.py`

## PR-3: Matrix v4 Gate + Bare-Metal Sub-gate

### Objective

Make expanded hardware and promotion checks release-blocking.

### Scope

- Add local gates:
  - `Makefile` target `test-hw-matrix-v4`
  - `Makefile` target `test-hw-baremetal-promotion-v1`
- Add CI steps:
  - `Hardware matrix v4 gate`
  - `Hardware bare-metal promotion v1 gate`
- Add aggregate tests:
  - `tests/hw/test_hw_gate_v4.py`
  - `tests/hw/test_hw_baremetal_promotion_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/hw/test_hw_gate_v4.py`
- `tests/hw/test_hw_baremetal_promotion_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-hw-matrix-v4`
- `make test-hw-baremetal-promotion-v1`

### Done criteria for PR-3

- Hardware v4 and bare-metal promotion sub-gates are required in local and CI.
- M37 can be marked done with deterministic promotion evidence.

### PR-3 completion summary

- Added aggregate gate checks:
  - `tests/hw/test_hw_gate_v4.py`
  - `tests/hw/test_hw_baremetal_promotion_gate_v1.py`
- Added local gates:
  - `make test-hw-matrix-v4`
  - `make test-hw-baremetal-promotion-v1`
- Added CI gates and artifacts:
  - `Hardware matrix v4 gate`
  - `Hardware bare-metal promotion v1 gate`
- Updated closure docs:
  - `MILESTONES.md`
  - `docs/STATUS.md`
  - `README.md`

## X2 runtime-backed closure addendum (2026-03-18)

- This backlog is now governed by the shared x2 hardware runtime qualification lane.
- Matrix v4 now feeds the shared x2 hardware runtime qualification device
  registry instead of remaining an isolated promotion-era report.
- `make test-x2-hardware-runtime-v1` consumes the v4 driver lifecycle rows as
  reusable probe/runtime foundation for later X2 targets.
- Additional device classes must land through the shared registry and target
  gate, not as one-off matrix expansions.

## Non-goals for M37 backlog

- Unsupported hardware-family claims without matrix v4 evidence.
- One-off bringup results treated as release support commitments.
