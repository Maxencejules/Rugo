# M41 Execution Backlog (Process + Readiness Compatibility Closure v1)

Date: 2026-03-09  
Lane: Rugo (Rust kernel + Go user space)  
Status: done

## Goal

Close high-impact process and readiness compatibility gaps needed for mainstream
userspace workloads while preserving deterministic deferred-surface behavior.

M41 source of truth remains `docs/M40_M44_GENERAL_PURPOSE_PARITY_ROADMAP.md`,
`MILESTONES.md`, and this backlog.

## Current State Summary

- Compatibility profile v4 still defers key surfaces (`fork`, `clone`,
  `epoll`, `io_uring`, namespace/cgroup, and some socket families).
- Existing M36 artifacts and gates are deterministic but need expanded runtime
  closure for process/readiness parity claims.
- Deferred behavior policy must remain explicit and release-blocking until
  closure is contractized and verified.

## Execution Result

- PR-1: complete (2026-03-10)
- PR-2: complete (2026-03-10)
- PR-3: complete (2026-03-10)

## PR-1: Process/Readiness Contract Freeze

### Objective

Define compatibility profile v5 and process/readiness closure semantics as
executable contracts.

### Scope

- Add docs:
  - `docs/abi/compat_profile_v5.md`
  - `docs/runtime/syscall_coverage_matrix_v4.md`
  - `docs/abi/process_model_v4.md`
  - `docs/abi/readiness_io_model_v1.md`
- Add tests:
  - `tests/compat/test_compat_docs_v5.py`

### Primary files

- `docs/abi/compat_profile_v5.md`
- `docs/runtime/syscall_coverage_matrix_v4.md`
- `docs/abi/process_model_v4.md`
- `docs/abi/readiness_io_model_v1.md`
- `tests/compat/test_compat_docs_v5.py`

### Acceptance checks

- `python -m pytest tests/compat/test_compat_docs_v5.py -v`

### Done criteria for PR-1

- Process/readiness compatibility boundaries are explicit, versioned, and
  test-backed.
- Deferred surfaces and closure intent are machine-verifiable.

### PR-1 completion summary

- Added process/readiness contract docs:
  - `docs/abi/compat_profile_v5.md`
  - `docs/runtime/syscall_coverage_matrix_v4.md`
  - `docs/abi/process_model_v4.md`
  - `docs/abi/readiness_io_model_v1.md`
- Added executable contract checks:
  - `tests/compat/test_compat_docs_v5.py`

## PR-2: Process/Readiness Closure Campaign

### Objective

Implement deterministic process/readiness compatibility behavior and runtime
campaign checks.

### Scope

- Add tooling:
  - `tools/run_compat_surface_campaign_v2.py`
  - `tools/run_posix_gap_report_v2.py`
- Add tests:
  - `tests/compat/test_fork_clone_surface_v1.py`
  - `tests/compat/test_epoll_surface_v1.py`
  - `tests/compat/test_process_model_v4.py`
  - `tests/compat/test_deferred_surface_behavior_v2.py`

### Primary files

- `tools/run_compat_surface_campaign_v2.py`
- `tools/run_posix_gap_report_v2.py`
- `tests/compat/test_fork_clone_surface_v1.py`
- `tests/compat/test_epoll_surface_v1.py`
- `tests/compat/test_process_model_v4.py`
- `tests/compat/test_deferred_surface_behavior_v2.py`

### Acceptance checks

- `python tools/run_compat_surface_campaign_v2.py --out out/compat-surface-v2.json`
- `python tools/run_posix_gap_report_v2.py --out out/posix-gap-report-v2.json`
- `python -m pytest tests/compat/test_fork_clone_surface_v1.py tests/compat/test_epoll_surface_v1.py tests/compat/test_process_model_v4.py tests/compat/test_deferred_surface_behavior_v2.py -v`

### Done criteria for PR-2

- Expanded process/readiness behavior is deterministic and machine-verifiable.
- Deferred surfaces remain explicit with deterministic unsupported outcomes.

### PR-2 completion summary

- Added deterministic compatibility campaign tooling:
  - `tools/run_compat_surface_campaign_v2.py`
  - `tools/run_posix_gap_report_v2.py`
- Added executable process/readiness checks:
  - `tests/compat/test_fork_clone_surface_v1.py`
  - `tests/compat/test_epoll_surface_v1.py`
  - `tests/compat/test_process_model_v4.py`
  - `tests/compat/test_deferred_surface_behavior_v2.py`

## PR-3: Process/Readiness Gate + POSIX Sub-gate

### Objective

Make process/readiness compatibility closure release-blocking.

### Scope

- Add local gates:
  - `Makefile` target `test-process-readiness-parity-v1`
  - `Makefile` target `test-posix-gap-closure-v2`
- Add CI steps:
  - `Process readiness parity v1 gate`
  - `POSIX gap closure v2 gate`
- Add aggregate tests:
  - `tests/compat/test_process_readiness_gate_v1.py`
  - `tests/compat/test_posix_gap_closure_gate_v2.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/compat/test_process_readiness_gate_v1.py`
- `tests/compat/test_posix_gap_closure_gate_v2.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-process-readiness-parity-v1`
- `make test-posix-gap-closure-v2`

### Done criteria for PR-3

- Process/readiness and POSIX sub-gates are required in local and CI release
  lanes.
- M41 can be marked done only with deterministic campaign artifacts.

### PR-3 completion summary

- Added aggregate gate checks:
  - `tests/compat/test_process_readiness_gate_v1.py`
  - `tests/compat/test_posix_gap_closure_gate_v2.py`
- Added local gates:
  - `make test-process-readiness-parity-v1`
  - `make test-posix-gap-closure-v2`
- Added CI gates and artifacts:
  - `Process readiness parity v1 gate`
  - `POSIX gap closure v2 gate`
- Updated closure docs:
  - `MILESTONES.md`
  - `docs/STATUS.md`
  - `README.md`

## Non-goals for M41 backlog

- Claiming full Linux ABI parity in one milestone.
- Implicitly supporting deferred APIs without explicit contract updates.
