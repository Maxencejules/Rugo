# M72 Execution Backlog (Sandboxed App Bundles + Permissions v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Add bounded app-bundle and permission semantics so third-party apps can be
distributed without full system trust.

M72 source of truth remains:

- `docs/POST_G2_EXTENDED_MILESTONES.md`
- `docs/M42_EXECUTION_BACKLOG.md`
- `docs/M71_EXECUTION_BACKLOG.md`
- this backlog

## Current State Summary

- Isolation and namespace work exists, but app distribution is still missing an
  explicit bundle and portal model.
- M71 adds a package manager that needs a safe delivery format for third-party
  apps.
- There is no versioned app-bundle, permission-policy, or portal API contract
  in-tree.
- M72 must define those semantics before SDK and catalog federation work widens
  the ecosystem surface.

## Execution plan

- PR-1: app bundle contract freeze
- PR-2: permission and portal campaign baseline
- PR-3: app-bundle gate wiring + closure

## Execution status

- PR-1: pending
- PR-2: pending
- PR-3: pending

## PR-1: App Bundle Contract Freeze

### Objective

Define bundle, permission, and portal semantics before third-party app claims
broaden.

### Scope

- Add docs:
  - `docs/pkg/app_bundle_contract_v1.md`
  - `docs/security/app_permission_policy_v1.md`
  - `docs/desktop/portal_api_contract_v1.md`
- Add tests:
  - `tests/pkg/test_app_bundle_docs_v1.py`
  - `tests/security/test_app_permission_docs_v1.py`

### Primary files

- `docs/pkg/app_bundle_contract_v1.md`
- `docs/security/app_permission_policy_v1.md`
- `docs/desktop/portal_api_contract_v1.md`
- `tests/pkg/test_app_bundle_docs_v1.py`
- `tests/security/test_app_permission_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/pkg/test_app_bundle_docs_v1.py tests/security/test_app_permission_docs_v1.py -v`

### Done criteria for PR-1

- App bundle, portal, and permission semantics are explicit and versioned.
- Permission denial and deferred capability paths remain deterministic.

## PR-2: Permission and Portal Campaign Baseline

### Objective

Implement deterministic evidence for bundle install, permission prompts, and
desktop portal behavior.

### Scope

- Add tooling:
  - `tools/run_app_bundle_policy_v1.py`
  - `tools/run_portal_contract_v1.py`
- Add tests:
  - `tests/pkg/test_app_bundle_manifest_v1.py`
  - `tests/security/test_app_permission_prompts_v1.py`
  - `tests/desktop/test_portal_file_dialog_v1.py`
  - `tests/security/test_app_bundle_negative_v1.py`

### Primary files

- `tools/run_app_bundle_policy_v1.py`
- `tools/run_portal_contract_v1.py`
- `tests/pkg/test_app_bundle_manifest_v1.py`
- `tests/security/test_app_permission_prompts_v1.py`
- `tests/desktop/test_portal_file_dialog_v1.py`
- `tests/security/test_app_bundle_negative_v1.py`

### Acceptance checks

- `python tools/run_app_bundle_policy_v1.py --out out/app-bundle-v1.json`
- `python tools/run_portal_contract_v1.py --out out/portal-contract-v1.json`
- `python -m pytest tests/pkg/test_app_bundle_manifest_v1.py tests/security/test_app_permission_prompts_v1.py tests/desktop/test_portal_file_dialog_v1.py tests/security/test_app_bundle_negative_v1.py -v`

### Done criteria for PR-2

- App-bundle artifacts are deterministic and machine-readable.
- `APP: install ok` and denial markers are stable.
- App delivery, prompts, and portal behavior remain capability-bounded and
  auditable.

## PR-3: App Bundle Gate + Permission Sub-gate

### Objective

Make bundle and permission behavior release-blocking for declared app
distribution profiles.

### Scope

- Add local gates:
  - `Makefile` target `test-app-bundles-v1`
  - `Makefile` target `test-app-permissions-v1`
- Add CI steps:
  - `App bundles v1 gate`
  - `App permissions v1 gate`
- Add aggregate tests:
  - `tests/pkg/test_app_bundles_gate_v1.py`
  - `tests/security/test_app_permissions_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/pkg/test_app_bundles_gate_v1.py`
- `tests/security/test_app_permissions_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-app-bundles-v1`
- `make test-app-permissions-v1`

### Done criteria for PR-3

- App-bundle and permission sub-gates are required in local and CI release
  lanes.
- M72 can be marked done only with deterministic bundle, prompt, and portal
  evidence for the declared distribution profile.

## Non-goals for M72 backlog

- developer SDK work owned by M73
- federated catalog and moderation work owned by M74
- fleet admission and attestation work owned by M77
