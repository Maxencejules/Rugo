# M27 Execution Backlog (External App Compatibility Program v3)

Date: 2026-03-06  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Scale external app compatibility from curated demos to practical app classes
with repeatable pass thresholds.

M27 source of truth remains `docs/M21_M34_MATURITY_PARITY_ROADMAP.md`,
`MILESTONES.md`, and this backlog.

## Current State Summary

- Compatibility v1/v2 planning exists with existing profile gates.
- M27 defines stable public compatibility tiers and release reports for v3.
- App-class matrix tooling and gate aggregation are pending.

## Execution Result

- PR-1: planned
- PR-2: planned
- PR-3: planned

## PR-1: Compatibility Tier Contract v3

### Objective

Freeze v3 compatibility profile and app-tier taxonomy.

### Scope

- Add docs:
  - `docs/abi/compat_profile_v3.md`
  - `docs/abi/app_compat_tiers_v1.md`
- Add tests:
  - `tests/compat/test_app_tier_docs_v1.py`

### Primary files

- `docs/abi/compat_profile_v3.md`
- `docs/abi/app_compat_tiers_v1.md`
- `tests/compat/test_app_tier_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/compat/test_app_tier_docs_v1.py -v`

### Done criteria for PR-1

- App compatibility tiers and pass rules are explicit and versioned.

## PR-2: App-Class Suite Expansion

### Objective

Execute deterministic app compatibility suites across key workload classes.

### Scope

- Add tests:
  - `tests/compat/test_cli_suite_v3.py`
  - `tests/compat/test_runtime_suite_v3.py`
  - `tests/compat/test_service_suite_v3.py`
- Add tooling:
  - `tools/run_app_compat_matrix_v3.py`

### Primary files

- `tests/compat/test_cli_suite_v3.py`
- `tests/compat/test_runtime_suite_v3.py`
- `tests/compat/test_service_suite_v3.py`
- `tools/run_app_compat_matrix_v3.py`

### Acceptance checks

- `python tools/run_app_compat_matrix_v3.py --out out/app-compat-matrix-v3.json`
- `python -m pytest tests/compat/test_cli_suite_v3.py tests/compat/test_runtime_suite_v3.py tests/compat/test_service_suite_v3.py -v`

### Done criteria for PR-2

- App compatibility reports are deterministic and machine-readable.
- Tier thresholds are reproducible across release lanes.

## PR-3: Compatibility v3 Gate + Closure

### Objective

Make app compatibility v3 release-blocking.

### Scope

- Add local gate:
  - `Makefile` target `test-app-compat-v3`
- Add CI step:
  - `App compatibility v3 gate`
- Add aggregate test:
  - `tests/compat/test_app_compat_gate_v3.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/compat/test_app_compat_gate_v3.py`
- `MILESTONES.md`
- `docs/STATUS.md`

### Acceptance checks

- `make test-app-compat-v3`

### Done criteria for PR-3

- App compatibility v3 gate is required in local and CI release lanes.
- M27 can be marked done with pass-history evidence.

## Non-goals for M27 backlog

- Universal app compatibility claims outside declared tiers.
- Unsupported ABI/runtime surfaces without explicit contract updates.

