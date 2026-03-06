# M32 Execution Backlog (Conformance + Profile Qualification v1)

Date: 2026-03-06  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Define profile-level conformance and qualify each release against explicit
machine-verifiable profile requirements.

M32 source of truth remains `docs/M21_M34_MATURITY_PARITY_ROADMAP.md`,
`MILESTONES.md`, and this backlog.

## Current State Summary

- Maturity program has subsystem gates, but profile-level qualification is missing.
- M32 adds server/developer/appliance profile conformance framework.
- Release artifacts currently lack formal profile qualification reports.

## Execution Result

- PR-1: planned
- PR-2: planned
- PR-3: planned

## PR-1: Profile Conformance Contract

### Objective

Freeze profile definitions and qualification criteria for v1.

### Scope

- Add docs:
  - `docs/runtime/profile_conformance_v1.md`
- Add tests:
  - `tests/runtime/test_profile_conformance_docs_v1.py`

### Primary files

- `docs/runtime/profile_conformance_v1.md`
- `tests/runtime/test_profile_conformance_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/runtime/test_profile_conformance_docs_v1.py -v`

### Done criteria for PR-1

- Profile requirements are explicit, versioned, and test-referenced.

## PR-2: Conformance Suite + Profile Tests

### Objective

Implement profile qualification tooling and executable checks.

### Scope

- Add tooling:
  - `tools/run_conformance_suite_v1.py`
- Add tests:
  - `tests/runtime/test_server_profile_v1.py`
  - `tests/runtime/test_dev_profile_v1.py`

### Primary files

- `tools/run_conformance_suite_v1.py`
- `tests/runtime/test_server_profile_v1.py`
- `tests/runtime/test_dev_profile_v1.py`

### Acceptance checks

- `python tools/run_conformance_suite_v1.py --out out/conformance-v1.json`
- `python -m pytest tests/runtime/test_server_profile_v1.py tests/runtime/test_dev_profile_v1.py -v`

### Done criteria for PR-2

- Conformance artifacts are deterministic and machine-readable.
- Profile checks are reproducible across release lanes.

## PR-3: Conformance Gate + Closure

### Objective

Make profile qualification release-blocking.

### Scope

- Add local gate:
  - `Makefile` target `test-conformance-v1`
- Add CI step:
  - `Conformance v1 gate`
- Add aggregate test:
  - `tests/runtime/test_conformance_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/runtime/test_conformance_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`

### Acceptance checks

- `make test-conformance-v1`

### Done criteria for PR-3

- Conformance gate is required in local and CI release lanes.
- M32 can be marked done with profile qualification artifacts.

## Non-goals for M32 backlog

- Immediate expansion to many profile variants without owner assignments.
- Certification claims beyond declared conformance scope.

