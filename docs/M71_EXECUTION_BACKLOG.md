# M71 Execution Backlog (Package Manager + Dependency Solver v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Add a first-class package manager with explicit solver, transaction, and repo-
pinning semantics that build on the existing ecosystem policy baseline.

M71 source of truth remains:

- `docs/POST_G2_EXTENDED_MILESTONES.md`
- `docs/M39_EXECUTION_BACKLOG.md`
- `docs/M70_EXECUTION_BACKLOG.md`
- this backlog

## Current State Summary

- Package and catalog policy work exists, but the post-desktop roadmap still
  lacks a declared first-class dependency solver.
- Desktop productivity workflows make package installation semantics more
  visible to users and app developers.
- There is no versioned package-manager, solver, or repo-metadata-v4 contract
  in-tree.
- M71 must define those boundaries before app bundles and SDK workflows build
  on them.

## Execution plan

- PR-1: package manager contract freeze
- PR-2: solver and transaction campaign baseline
- PR-3: package-manager gate wiring + closure

## Execution status

- PR-1: pending
- PR-2: pending
- PR-3: pending

## PR-1: Package Manager Contract Freeze

### Objective

Define package-manager, dependency-solver, and pinning semantics before
implementation broadens ecosystem claims.

### Scope

- Add docs:
  - `docs/pkg/package_manager_contract_v1.md`
  - `docs/pkg/dependency_solver_policy_v1.md`
  - `docs/pkg/repo_metadata_v4.md`
- Add tests:
  - `tests/pkg/test_pkg_manager_docs_v1.py`

### Primary files

- `docs/pkg/package_manager_contract_v1.md`
- `docs/pkg/dependency_solver_policy_v1.md`
- `docs/pkg/repo_metadata_v4.md`
- `tests/pkg/test_pkg_manager_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/pkg/test_pkg_manager_docs_v1.py -v`

### Done criteria for PR-1

- Solver, transaction, and pinning semantics are explicit and versioned.
- Deferred resolver behavior remains deterministic and machine-verifiable.

## PR-2: Solver and Transaction Campaign Baseline

### Objective

Implement deterministic evidence for solving, rollback, and repo pinning
behavior.

### Scope

- Add tooling:
  - `tools/run_pkg_solver_v1.py`
  - `tools/run_pkg_transaction_audit_v1.py`
- Add tests:
  - `tests/pkg/test_dependency_solver_v1.py`
  - `tests/pkg/test_pkg_transaction_rollback_v1.py`
  - `tests/pkg/test_repo_pinning_v1.py`
  - `tests/pkg/test_pkg_solver_negative_v1.py`

### Primary files

- `tools/run_pkg_solver_v1.py`
- `tools/run_pkg_transaction_audit_v1.py`
- `tests/pkg/test_dependency_solver_v1.py`
- `tests/pkg/test_pkg_transaction_rollback_v1.py`
- `tests/pkg/test_repo_pinning_v1.py`
- `tests/pkg/test_pkg_solver_negative_v1.py`

### Acceptance checks

- `python tools/run_pkg_solver_v1.py --out out/pkg-solver-v1.json`
- `python tools/run_pkg_transaction_audit_v1.py --out out/pkg-transaction-v1.json`
- `python -m pytest tests/pkg/test_dependency_solver_v1.py tests/pkg/test_pkg_transaction_rollback_v1.py tests/pkg/test_repo_pinning_v1.py tests/pkg/test_pkg_solver_negative_v1.py -v`

### Done criteria for PR-2

- Package-manager artifacts are deterministic and machine-readable.
- `PKG: solve ok` and rollback markers are stable across seeded resolver cases.
- Later app-bundle and SDK work can reference one explicit install/rollback
  contract.

## PR-3: Package Manager Gate + Transaction Audit Sub-gate

### Objective

Make the package-manager baseline release-blocking for declared ecosystem
profiles.

### Scope

- Add local gates:
  - `Makefile` target `test-pkg-manager-v1`
  - `Makefile` target `test-pkg-transaction-audit-v1`
- Add CI steps:
  - `Package manager v1 gate`
  - `Package transaction audit v1 gate`
- Add aggregate tests:
  - `tests/pkg/test_pkg_manager_gate_v1.py`
  - `tests/pkg/test_pkg_transaction_audit_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/pkg/test_pkg_manager_gate_v1.py`
- `tests/pkg/test_pkg_transaction_audit_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-pkg-manager-v1`
- `make test-pkg-transaction-audit-v1`

### Done criteria for PR-3

- Package-manager and transaction-audit sub-gates are required in local and CI
  release lanes.
- M71 can be marked done only with deterministic solver and rollback evidence
  for the declared ecosystem profile.

## Non-goals for M71 backlog

- app sandboxing owned by M72
- developer SDK and porting workflows owned by M73
- federated catalog and build farm work owned by M74
