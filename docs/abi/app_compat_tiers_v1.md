# App Compatibility Tiers v1

Date: 2026-03-09  
Milestone: M27 External App Compatibility Program v3  
Status: active release gate

## Purpose

Define the public workload-tier taxonomy and pass thresholds used by M27 app
compatibility release gates.

## Contract identifiers

- Tier contract ID: `rugo.app_compat_tiers.v1`
- Parent compatibility profile: `rugo.compat_profile.v3`
- Report schema: `rugo.app_compat_matrix_report.v3`

## Tier taxonomy

### Tier `tier_cli`

- Workload class: `cli`
- Focus: practical non-interactive command-line utilities.
- Minimum eligible cases: `14`
- Minimum pass rate: `0.90`

### Tier `tier_runtime`

- Workload class: `runtime`
- Focus: runtime-dependent application behavior and startup determinism.
- Minimum eligible cases: `10`
- Minimum pass rate: `0.80`

### Tier `tier_service`

- Workload class: `service`
- Focus: long-lived service lifecycle and restart/dependency determinism.
- Minimum eligible cases: `8`
- Minimum pass rate: `0.80`

## Determinism and trust rules

- Every case must carry a signed artifact provenance bit.
- Every case must be deterministic (`deterministic=true`).
- Every case must declare ABI profile `compat_profile_v3`.
- Unknown workload classes or tier mismatches are release-blocking.

## Expected matrix report fields (`rugo.app_compat_matrix_report.v3`)

- `schema`
- `profile_id`
- `tier_schema`
- `seed`
- `classes`
- `cases`
- `issues`
- `gate_pass`
- `digest`

## Tooling and gate wiring

- Matrix runner: `tools/run_app_compat_matrix_v3.py`
- Local gate: `make test-app-compat-v3`
- CI step: `App compatibility v3 gate`

## Required tests

- `tests/compat/test_app_tier_docs_v1.py`
- `tests/compat/test_cli_suite_v3.py`
- `tests/compat/test_runtime_suite_v3.py`
- `tests/compat/test_service_suite_v3.py`
- `tests/compat/test_app_compat_gate_v3.py`
