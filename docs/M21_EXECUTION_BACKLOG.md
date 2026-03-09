# M21 Execution Backlog (ABI + API Stability Program v3)

Date: 2026-03-06  
Lane: Rugo (Rust kernel + Go user space)  
Status: done

## Goal

Make ABI/API stability predictable across multi-release windows with explicit
versioning, deprecation, and compatibility obligations.

M21 source of truth remains `docs/M21_M34_MATURITY_PARITY_ROADMAP.md`,
`MILESTONES.md`, and this backlog.

## Current State Summary

- ABI policy foundations exist in prior runtime maturity work.
- M21 is the v3 freeze/enforcement milestone for long-window compatibility.
- Dedicated v3 compatibility diff gates are not yet implemented.

## Execution Result

- PR-1: complete (2026-03-09)
- PR-2: complete (2026-03-09)
- PR-3: complete (2026-03-09)

## PR-1: Contract Freeze

### Objective

Define ABI/API stability v3 obligations before enforcement tooling is enabled.

### Scope

- Add docs:
  - `docs/abi/syscall_v3.md`
  - `docs/runtime/abi_stability_policy_v2.md`
  - `docs/runtime/deprecation_window_policy_v1.md`
- Add tests:
  - `tests/runtime/test_abi_docs_v3.py`
  - `tests/runtime/test_abi_window_v3.py`

### Primary files

- `docs/abi/syscall_v3.md`
- `docs/runtime/abi_stability_policy_v2.md`
- `docs/runtime/deprecation_window_policy_v1.md`
- `tests/runtime/test_abi_docs_v3.py`
- `tests/runtime/test_abi_window_v3.py`

### Acceptance checks

- `python -m pytest tests/runtime/test_abi_docs_v3.py tests/runtime/test_abi_window_v3.py -v`

### Done criteria for PR-1

- ABI v3 contract is versioned and test-referenced.
- Deprecation windows and obligations are explicit.

### PR-1 completion summary

- Added ABI stability contract docs:
  - `docs/abi/syscall_v3.md`
  - `docs/runtime/abi_stability_policy_v2.md`
  - `docs/runtime/deprecation_window_policy_v1.md`
- Added executable PR-1 checks:
  - `tests/runtime/test_abi_docs_v3.py`
  - `tests/runtime/test_abi_window_v3.py`

## PR-2: Compatibility Enforcement

### Objective

Automate ABI diff and syscall compatibility checks.

### Scope

- Add tooling:
  - `tools/check_abi_diff_v3.py`
  - `tools/check_syscall_compat_v3.py`
- Add tests:
  - `tests/runtime/test_abi_diff_gate_v3.py`
  - `tests/compat/test_abi_compat_matrix_v3.py`

### Primary files

- `tools/check_abi_diff_v3.py`
- `tools/check_syscall_compat_v3.py`
- `tests/runtime/test_abi_diff_gate_v3.py`
- `tests/compat/test_abi_compat_matrix_v3.py`

### Acceptance checks

- `python -m pytest tests/runtime/test_abi_diff_gate_v3.py tests/compat/test_abi_compat_matrix_v3.py -v`

### Done criteria for PR-2

- ABI changes are machine-diffed and policy-checked.
- Breaking changes require explicit migration/version actions.

### PR-2 completion summary

- Added deterministic ABI/policy enforcement tooling:
  - `tools/check_abi_diff_v3.py`
  - `tools/check_syscall_compat_v3.py`
- Added executable PR-2 checks:
  - `tests/runtime/test_abi_diff_gate_v3.py`
  - `tests/compat/test_abi_compat_matrix_v3.py`
- Added compatibility-matrix schema and explicit migration-action enforcement for
  breaking ABI diff outcomes.

## PR-3: Gate + Closure

### Objective

Promote ABI stability v3 checks to release-blocking status.

### Scope

- Add local gate:
  - `Makefile` target `test-abi-stability-v3`
- Add CI step:
  - `ABI stability v3 gate`
- Add aggregate test:
  - `tests/runtime/test_abi_stability_gate_v3.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/runtime/test_abi_stability_gate_v3.py`
- `MILESTONES.md`
- `docs/STATUS.md`

### Acceptance checks

- `make test-abi-stability-v3`

### Done criteria for PR-3

- ABI stability v3 gate is required in local and CI release lanes.
- M21 can be marked done with evidence pointers.

### PR-3 completion summary

- Added aggregate gate test:
  - `tests/runtime/test_abi_stability_gate_v3.py`
- Added local gate:
  - `make test-abi-stability-v3`
  - JUnit output: `out/pytest-abi-stability-v3.xml`
- Added CI gate + artifact upload:
  - step: `ABI stability v3 gate`
  - artifact: `abi-stability-v3-artifacts`
- Updated closure docs:
  - `MILESTONES.md`
  - `docs/STATUS.md`

## Non-goals for M21 backlog

- Expanding ABI scope without compatibility policy ownership.
- One-off ABI changes outside versioned release windows.
