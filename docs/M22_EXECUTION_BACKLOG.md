# M22 Execution Backlog (Kernel Reliability + Soak v1)

Date: 2026-03-06  
Lane: Rugo (Rust kernel + Go user space)  
Status: done

## Goal

Prove long-run kernel stability under mixed workload stress and fault injection.

M22 source of truth remains `docs/M21_M34_MATURITY_PARITY_ROADMAP.md`,
`MILESTONES.md`, and this backlog.

## Current State Summary

- Stress/fault concepts exist from prior milestones.
- M22 formalizes release-blocking reliability thresholds.
- Dedicated 24h soak and fault campaign artifacts are now implemented.

## Execution Result

- PR-1: complete (2026-03-09)
- PR-2: complete (2026-03-09)
- PR-3: complete (2026-03-09)

## PR-1: Reliability Model + Soak Baseline

### Objective

Freeze reliability model and baseline soak/fault expectations.

### Scope

- Add docs:
  - `docs/runtime/kernel_reliability_model_v1.md`
- Add tests:
  - `tests/stress/test_kernel_soak_24h_v1.py`
  - `tests/stress/test_fault_injection_matrix_v1.py`

### Primary files

- `docs/runtime/kernel_reliability_model_v1.md`
- `tests/stress/test_kernel_soak_24h_v1.py`
- `tests/stress/test_fault_injection_matrix_v1.py`

### Acceptance checks

- `python -m pytest tests/stress/test_kernel_soak_24h_v1.py tests/stress/test_fault_injection_matrix_v1.py -v`

### Done criteria for PR-1

- Reliability thresholds are explicit and versioned.
- Soak/fault model assumptions are test-referenced.

### PR-1 completion summary

- Added versioned reliability model doc:
  - `docs/runtime/kernel_reliability_model_v1.md`
- Added executable PR-1 baseline checks:
  - `tests/stress/test_kernel_soak_24h_v1.py`
  - `tests/stress/test_fault_injection_matrix_v1.py`

## PR-2: Campaign Tooling + Artifact Schema

### Objective

Generate deterministic reliability artifacts for release evidence.

### Scope

- Add tooling:
  - `tools/run_kernel_soak_v1.py`
  - `tools/run_fault_campaign_kernel_v1.py`
- Add tests:
  - `tests/stress/test_reliability_artifact_schema_v1.py`

### Primary files

- `tools/run_kernel_soak_v1.py`
- `tools/run_fault_campaign_kernel_v1.py`
- `tests/stress/test_reliability_artifact_schema_v1.py`

### Acceptance checks

- `python tools/run_kernel_soak_v1.py --out out/kernel-soak-v1.json`
- `python tools/run_fault_campaign_kernel_v1.py --out out/kernel-fault-campaign-v1.json`
- `python -m pytest tests/stress/test_reliability_artifact_schema_v1.py -v`

### Done criteria for PR-2

- Reliability artifacts are stable and machine-readable.
- Campaign behavior is deterministic across seeded runs.

### PR-2 completion summary

- Added deterministic reliability tooling:
  - `tools/run_kernel_soak_v1.py`
  - `tools/run_fault_campaign_kernel_v1.py`
- Added executable schema and determinism checks:
  - `tests/stress/test_reliability_artifact_schema_v1.py`

## PR-3: Reliability Gate + Closure

### Objective

Make reliability/soak checks release-blocking.

### Scope

- Add local gate:
  - `Makefile` target `test-kernel-reliability-v1`
- Add CI step:
  - `Kernel reliability v1 gate`
- Add aggregate test:
  - `tests/stress/test_kernel_reliability_gate_v1.py`
- Mark closure docs after green gate:
  - `MILESTONES.md`
  - `docs/STATUS.md`
  - `README.md`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/stress/test_kernel_reliability_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-kernel-reliability-v1`

### Done criteria for PR-3

- Reliability v1 gate is required in local and CI release lanes.
- M22 can be marked done with soak/fault evidence pointers.

### PR-3 completion summary

- Added aggregate gate test:
  - `tests/stress/test_kernel_reliability_gate_v1.py`
- Added local gate:
  - `make test-kernel-reliability-v1`
  - JUnit output: `out/pytest-kernel-reliability-v1.xml`
- Added CI gate + artifact upload:
  - step: `Kernel reliability v1 gate`
  - artifact: `kernel-reliability-v1-artifacts`
- Updated closure docs:
  - `MILESTONES.md`
  - `docs/STATUS.md`
  - `README.md`

## Non-goals for M22 backlog

- Infinite-duration soak claims beyond bounded campaign windows.
- Hardware-coverage claims outside declared matrix tiers.
