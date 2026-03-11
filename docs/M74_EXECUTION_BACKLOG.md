# M74 Execution Backlog (Catalog Federation + Build Farm v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Add a federated app catalog and reproducible build-farm baseline with explicit
attestation and moderation policy.

M74 source of truth remains:

- `docs/POST_G2_EXTENDED_MILESTONES.md`
- `docs/M39_EXECUTION_BACKLOG.md`
- `docs/M73_EXECUTION_BACKLOG.md`
- this backlog

## Current State Summary

- Ecosystem scale and distribution policy exists from earlier milestones, and
  M73 defines the SDK surface external contributors depend on.
- There is still no versioned catalog federation, build-farm attestation, or
  moderation workflow contract in the post-M70 plan.
- Build provenance remains bounded to project-controlled artifacts.
- M74 must define those semantics before broader community and release-train
  operations depend on them.

## Execution plan

- PR-1: catalog federation contract freeze
- PR-2: build-farm and moderation campaign baseline
- PR-3: catalog federation gate wiring + closure

## Execution status

- PR-1: pending
- PR-2: pending
- PR-3: pending

## PR-1: Catalog Federation Contract Freeze

### Objective

Define federation, attestation, and moderation semantics before implementation
widens ecosystem claims.

### Scope

- Add docs:
  - `docs/pkg/catalog_federation_policy_v1.md`
  - `docs/pkg/build_farm_attestation_v1.md`
  - `docs/pkg/moderation_workflow_v1.md`
- Add tests:
  - `tests/pkg/test_catalog_federation_docs_v1.py`

### Primary files

- `docs/pkg/catalog_federation_policy_v1.md`
- `docs/pkg/build_farm_attestation_v1.md`
- `docs/pkg/moderation_workflow_v1.md`
- `tests/pkg/test_catalog_federation_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/pkg/test_catalog_federation_docs_v1.py -v`

### Done criteria for PR-1

- Catalog federation, attestation, and moderation behavior is explicit and
  versioned.
- Quarantine and rejection paths remain deterministic and reviewable.

## PR-2: Build-farm and Moderation Campaign Baseline

### Objective

Implement deterministic evidence for catalog ingest, build-farm attestations,
and moderation/quarantine workflows.

### Scope

- Add tooling:
  - `tools/run_catalog_federation_v1.py`
  - `tools/run_build_farm_attestation_v1.py`
  - `tools/run_catalog_quarantine_v1.py`
- Add tests:
  - `tests/pkg/test_catalog_federation_v1.py`
  - `tests/pkg/test_build_farm_attestation_v1.py`
  - `tests/pkg/test_catalog_quarantine_v1.py`
  - `tests/pkg/test_catalog_federation_negative_v1.py`

### Primary files

- `tools/run_catalog_federation_v1.py`
- `tools/run_build_farm_attestation_v1.py`
- `tools/run_catalog_quarantine_v1.py`
- `tests/pkg/test_catalog_federation_v1.py`
- `tests/pkg/test_build_farm_attestation_v1.py`
- `tests/pkg/test_catalog_quarantine_v1.py`
- `tests/pkg/test_catalog_federation_negative_v1.py`

### Acceptance checks

- `python tools/run_catalog_federation_v1.py --out out/catalog-federation-v1.json`
- `python tools/run_build_farm_attestation_v1.py --out out/build-farm-attestation-v1.json`
- `python tools/run_catalog_quarantine_v1.py --out out/catalog-quarantine-v1.json`
- `python -m pytest tests/pkg/test_catalog_federation_v1.py tests/pkg/test_build_farm_attestation_v1.py tests/pkg/test_catalog_quarantine_v1.py tests/pkg/test_catalog_federation_negative_v1.py -v`

### Done criteria for PR-2

- Catalog federation artifacts are deterministic and machine-readable.
- `CAT: ingest ok` and attestation markers are stable.
- Moderation and quarantine behavior remains explicit and auditable.

## PR-3: Catalog Health v2 Gate + Build-farm Sub-gate

### Objective

Make catalog federation and build-farm attestation release-blocking for
declared ecosystem profiles.

### Scope

- Add local gates:
  - `Makefile` target `test-app-catalog-health-v2`
  - `Makefile` target `test-build-farm-attestation-v1`
- Add CI steps:
  - `App catalog health v2 gate`
  - `Build farm attestation v1 gate`
- Add aggregate tests:
  - `tests/pkg/test_app_catalog_health_gate_v2.py`
  - `tests/pkg/test_build_farm_attestation_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/pkg/test_app_catalog_health_gate_v2.py`
- `tests/pkg/test_build_farm_attestation_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-app-catalog-health-v2`
- `make test-build-farm-attestation-v1`

### Done criteria for PR-3

- Catalog-health and build-farm sub-gates are required in local and CI release
  lanes.
- M74 can be marked done only with deterministic ingest, attestation, and
  moderation evidence.

## Non-goals for M74 backlog

- automatic approval of unaudited third-party packages
- community governance and support-channel work owned by M82-M84
- fleet admission and secrets work owned by M77
