# M17 Execution Backlog (Compatibility Profile v2)

Date: 2026-03-06  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Increase external software viability via ABI/loader contract maturity, POSIX
subset expansion, and deterministic compatibility tier gating.

M17 source of truth remains `docs/M15_M20_MULTIPURPOSE_PLAN.md`,
`MILESTONES.md`, and this backlog.

## Current State Summary

- Compatibility profile v1 exists and is gate-backed.
- ABI and loader contracts are already documented at v1.
- M17 closes v2 profile scope and deterministic external app tier behavior.

## Execution Result

- PR-1: planned
- PR-2: planned
- PR-3: planned

## PR-1: ABI + Loader Contract v2

### Objective

Freeze ABI and loader v2 expectations before expanding POSIX/profile coverage.

### Scope

- Add docs:
  - `docs/abi/syscall_v2.md`
  - `docs/abi/compat_profile_v2.md`
  - `docs/abi/elf_loader_contract_v2.md`
- Add tests:
  - `tests/compat/test_abi_profile_v2_docs.py`
  - `tests/compat/test_elf_loader_dynamic_v2.py`

### Primary files

- `docs/abi/syscall_v2.md`
- `docs/abi/compat_profile_v2.md`
- `docs/abi/elf_loader_contract_v2.md`
- `tests/compat/test_abi_profile_v2_docs.py`
- `tests/compat/test_elf_loader_dynamic_v2.py`

### Acceptance checks

- `python -m pytest tests/compat/test_abi_profile_v2_docs.py tests/compat/test_elf_loader_dynamic_v2.py -v`

### Done criteria for PR-1

- ABI/profile docs are versioned and executable-check referenced.
- Loader contract behavior is deterministic for covered profiles.

## PR-2: POSIX Subset Expansion + External App Tier

### Objective

Define v2 syscall coverage and external app tier pass thresholds.

### Scope

- Add docs:
  - `docs/runtime/syscall_coverage_matrix_v2.md`
- Add tests:
  - `tests/compat/test_posix_profile_v2.py`
  - `tests/compat/test_external_apps_tier_v2.py`
- Add fixture model:
  - `tests/compat/v2_model.py`

### Primary files

- `docs/runtime/syscall_coverage_matrix_v2.md`
- `tests/compat/test_posix_profile_v2.py`
- `tests/compat/test_external_apps_tier_v2.py`
- `tests/compat/v2_model.py`

### Acceptance checks

- `python -m pytest tests/compat/test_posix_profile_v2.py tests/compat/test_external_apps_tier_v2.py -v`

### Done criteria for PR-2

- Supported/unsupported profile surfaces are explicit.
- External app tier thresholds are deterministic and repeatable.

## PR-3: Compatibility Gate + CI Promotion

### Objective

Make compatibility profile v2 a release-blocking gate.

### Scope

- Add aggregate test:
  - `tests/compat/test_compat_gate_v2.py`
- Add local gate:
  - `Makefile` target `test-compat-v2`
- Add CI gate:
  - `.github/workflows/ci.yml` step `Compatibility profile v2 gate`

### Primary files

- `tests/compat/test_compat_gate_v2.py`
- `Makefile`
- `.github/workflows/ci.yml`
- `MILESTONES.md`
- `docs/STATUS.md`

### Acceptance checks

- `make test-compat-v2`

### Done criteria for PR-3

- Compatibility profile v2 gate is required in local and CI release lanes.
- M17 status can be marked done with evidence pointers.

## Non-goals for M17 backlog

- Full Linux distribution compatibility parity.
- Broad GUI/desktop runtime compatibility scope.

