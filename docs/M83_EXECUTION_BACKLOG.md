# M83 Execution Backlog (Localization + Docs Quality Pipeline v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Add explicit localization and docs-quality semantics so user-facing materials
stay consistent, testable, and releasable.

M83 source of truth remains:

- `docs/POST_G2_EXTENDED_MILESTONES.md`
- `docs/M52_EXECUTION_BACKLOG.md`
- `docs/M82_EXECUTION_BACKLOG.md`
- this backlog

## Current State Summary

- Contributor process work in M82 establishes the governance baseline for docs
  and localization changes.
- The desktop and installer now need explicit string-catalog and docs-release
  semantics for broader user adoption.
- There is no versioned localization policy, docs-release contract, or desktop
  string-catalog contract in-tree.
- M83 must define those semantics before public release-train and support
  operations depend on them.

## Execution plan

- PR-1: localization and docs contract freeze
- PR-2: catalog and docs-release campaign baseline
- PR-3: docs-quality gate wiring + closure

## Execution status

- PR-1: pending
- PR-2: pending
- PR-3: pending

## PR-1: Localization and Docs Contract Freeze

### Objective

Define localization, docs-release, and string-catalog semantics before
implementation broadens public-release claims.

### Scope

- Add docs:
  - `docs/community/localization_policy_v1.md`
  - `docs/build/docs_release_contract_v1.md`
  - `docs/desktop/string_catalog_contract_v1.md`
- Add tests:
  - `tests/docs/test_localization_docs_v1.py`

### Primary files

- `docs/community/localization_policy_v1.md`
- `docs/build/docs_release_contract_v1.md`
- `docs/desktop/string_catalog_contract_v1.md`
- `tests/docs/test_localization_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/docs/test_localization_docs_v1.py -v`

### Done criteria for PR-1

- Localization, docs-release, and string-catalog semantics are explicit and
  versioned.
- Translation-drift and missing-catalog paths remain deterministic.

## PR-2: Catalog and Docs-release Campaign Baseline

### Objective

Implement deterministic evidence for localization catalog health and docs
release consistency.

### Scope

- Add tooling:
  - `tools/run_localization_catalog_audit_v1.py`
  - `tools/run_docs_release_check_v1.py`
- Add tests:
  - `tests/docs/test_localization_catalog_v1.py`
  - `tests/docs/test_docs_release_contract_v1.py`
  - `tests/desktop/test_string_catalog_consistency_v1.py`
  - `tests/docs/test_localization_negative_v1.py`

### Primary files

- `tools/run_localization_catalog_audit_v1.py`
- `tools/run_docs_release_check_v1.py`
- `tests/docs/test_localization_catalog_v1.py`
- `tests/docs/test_docs_release_contract_v1.py`
- `tests/desktop/test_string_catalog_consistency_v1.py`
- `tests/docs/test_localization_negative_v1.py`

### Acceptance checks

- `python tools/run_localization_catalog_audit_v1.py --out out/localization-catalog-v1.json`
- `python tools/run_docs_release_check_v1.py --out out/docs-release-v1.json`
- `python -m pytest tests/docs/test_localization_catalog_v1.py tests/docs/test_docs_release_contract_v1.py tests/desktop/test_string_catalog_consistency_v1.py tests/docs/test_localization_negative_v1.py -v`

### Done criteria for PR-2

- Localization artifacts are deterministic and machine-readable.
- Catalog consistency and docs-release markers remain stable.
- Public release-train work can reference one explicit docs-quality baseline.

## PR-3: Docs Quality Gate + Localization Sub-gate

### Objective

Make localization and docs-quality behavior release-blocking for public-facing
materials.

### Scope

- Add local gates:
  - `Makefile` target `test-doc-quality-v1`
  - `Makefile` target `test-localization-catalog-v1`
- Add CI steps:
  - `Doc quality v1 gate`
  - `Localization catalog v1 gate`
- Add aggregate tests:
  - `tests/docs/test_doc_quality_gate_v1.py`
  - `tests/docs/test_localization_catalog_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/docs/test_doc_quality_gate_v1.py`
- `tests/docs/test_localization_catalog_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-doc-quality-v1`
- `make test-localization-catalog-v1`

### Done criteria for PR-3

- Docs-quality and localization sub-gates are required in local and CI release
  lanes.
- M83 can be marked done only with deterministic localization and docs-release
  evidence for public-facing materials.

## Non-goals for M83 backlog

- community release-train and support-channel operations owned by M84
- broad translation coverage beyond declared catalog scope
- manual-only docs checks without deterministic validation
