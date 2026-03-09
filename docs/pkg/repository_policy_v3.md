# Repository Policy v3

Date: 2026-03-09  
Milestone: M26 Package/Repo Ecosystem v3  
Status: active release gate

## Purpose

Define the policy controls required for v3 repository publication, validation,
and auditability.

## Contract identifiers

- Policy ID: `rugo.repository_policy.v3`
- Policy report schema: `rugo.repo_policy_report.v3`

## Policy requirements

### Publication and mutability

- Repository index sequence must be strictly monotonic.
- Index updates are append-only by sequence.
- Artifact replacement at an existing sequence is prohibited.

### Metadata quality and expiry

- Required fields must be present for every package index entry.
- Metadata expiry is mandatory.
- Maximum metadata validity window hours: `168`.
- Allowed metadata clock skew seconds: `300`.

### Artifact and rebuild integrity

- Every indexed artifact must provide `artifact_sha256` and `artifact_size`.
- Repository records must include `source_commit` and `build_recipe_id`.
- Rebuild manifests are mandatory for release-bound artifacts.

### Trust controls

- Revoked signing key IDs must be present in index metadata as
  `revoked_key_ids`.
- Client policy must reject metadata signed only by revoked keys.
- Update trust controls are release-blocking through sub-gate
  `make test-update-trust-v1`.

## Enforcement and evidence

- Policy checker: `tools/repo_policy_check_v3.py`
- Rebuild checker: `tools/pkg_rebuild_verify_v3.py`
- Local gate: `make test-pkg-ecosystem-v3`
- CI step: `Package ecosystem v3 gate`

## Required tests

- `tests/pkg/test_pkg_contract_docs_v3.py`
- `tests/pkg/test_repo_policy_v3.py`
- `tests/pkg/test_pkg_rebuild_repro_v3.py`
- `tests/pkg/test_pkg_ecosystem_gate_v3.py`

