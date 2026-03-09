# Package/Repository Format v3

Date: 2026-03-09  
Milestone: M26 Package/Repo Ecosystem v3  
Status: active release gate

## Purpose

Freeze the package artifact and rebuild-integrity contract for ecosystem v3.

## Contract identifiers

- Package Format ID: `rugo.pkg_format.v3`
- Package schema: `rugo.pkg.v3`
- Repository index schema: `rugo.repo_index.v3`
- Rebuild manifest schema: `rugo.pkg_rebuild_manifest.v3`
- Rebuild report schema: `rugo.pkg_rebuild_report.v3`

## Package blob contract (`rugo.pkg.v3`)

- Deterministic package identity:
  - `name`
  - `version`
  - `release`
  - `source_commit`
  - `build_recipe_id`
- Payload integrity:
  - `payload_sha256`
  - `payload_size`
- Build inputs:
  - `toolchain_digest`
  - `build_env_digest`
  - `dependency_lock_digest`
- Signature bundle:
  - `signatures[]` with `key_id`, `alg`, `sig`

Metadata serialization must use canonical JSON (sorted keys, compact separators).

## Repository index contract (`rugo.repo_index.v3`)

- Required top-level fields:
  - `schema` = `rugo.repo_index.v3`
  - `sequence`
  - `generated_utc`
  - `expires_utc`
  - `packages[]`
  - `revoked_key_ids[]`
- `packages[]` entries bind package identity and immutable payload fields:
  - `name`, `version`, `release`
  - `artifact_path`, `artifact_sha256`, `artifact_size`
  - `source_commit`, `build_recipe_id`

Repository mutation policy is append-only at the sequence boundary; replacement
of already-published artifacts is prohibited.

## Rebuild integrity contract

- Every published package requires a reproducible rebuild manifest:
  - `package`, `source_commit`, `build_recipe_id`, `toolchain_digest`,
    `dependency_lock_digest`, `expected_artifact_sha256`.
- Rebuild verification runs from the manifest and emits
  `rugo.pkg_rebuild_report.v3`.
- Gate rule: `total_mismatches` must be `0`.

## Gate wiring

- Local gate: `make test-pkg-ecosystem-v3`
- Required tool: `tools/pkg_rebuild_verify_v3.py`
- Required tests:
  - `tests/pkg/test_pkg_contract_docs_v3.py`
  - `tests/pkg/test_pkg_rebuild_repro_v3.py`
  - `tests/pkg/test_pkg_ecosystem_gate_v3.py`

