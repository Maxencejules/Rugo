# M26 Execution Backlog (Package/Repo Ecosystem v3)

Date: 2026-03-06  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Move package/repo operation to v3 with policy enforcement, rebuild integrity,
and explicit update trust/key-rotation controls.

M26 source of truth remains `docs/M21_M34_MATURITY_PARITY_ROADMAP.md`,
`MILESTONES.md`, and this backlog.

## Current State Summary

- Package/update baseline exists from prior release-engineering work.
- M26 elevates ecosystem policy and rebuild integrity to v3.
- New sub-gate `test-update-trust-v1` must be integrated for maturity parity.

## Execution Result

- PR-1: planned
- PR-2: planned
- PR-3: planned

## PR-1: Package/Repo + Update Trust Contracts

### Objective

Freeze v3 package/repo contract and v1 update trust/key rotation policy.

### Scope

- Add docs:
  - `docs/pkg/package_format_v3.md`
  - `docs/pkg/repository_policy_v3.md`
  - `docs/pkg/update_trust_model_v1.md`
  - `docs/security/update_key_rotation_policy_v1.md`
- Add tests:
  - `tests/pkg/test_pkg_contract_docs_v3.py`
  - `tests/pkg/test_update_trust_docs_v1.py`

### Primary files

- `docs/pkg/package_format_v3.md`
- `docs/pkg/repository_policy_v3.md`
- `docs/pkg/update_trust_model_v1.md`
- `docs/security/update_key_rotation_policy_v1.md`
- `tests/pkg/test_pkg_contract_docs_v3.py`
- `tests/pkg/test_update_trust_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/pkg/test_pkg_contract_docs_v3.py tests/pkg/test_update_trust_docs_v1.py -v`

### Done criteria for PR-1

- Package/repo and update-trust policies are versioned and executable-check linked.

## PR-2: Ecosystem + Trust Enforcement Tooling

### Objective

Enforce v3 policy, rebuild verification, and update-trust attack resistance.

### Scope

- Add tooling:
  - `tools/repo_policy_check_v3.py`
  - `tools/pkg_rebuild_verify_v3.py`
  - `tools/check_update_trust_v1.py`
  - `tools/run_update_key_rotation_drill_v1.py`
- Add tests:
  - `tests/pkg/test_pkg_rebuild_repro_v3.py`
  - `tests/pkg/test_repo_policy_v3.py`
  - `tests/pkg/test_update_metadata_expiry_v1.py`
  - `tests/pkg/test_update_freeze_attack_v1.py`
  - `tests/pkg/test_update_mix_and_match_v1.py`
  - `tests/pkg/test_update_key_rotation_v1.py`

### Primary files

- `tools/repo_policy_check_v3.py`
- `tools/pkg_rebuild_verify_v3.py`
- `tools/check_update_trust_v1.py`
- `tools/run_update_key_rotation_drill_v1.py`
- `tests/pkg/test_pkg_rebuild_repro_v3.py`
- `tests/pkg/test_repo_policy_v3.py`
- `tests/pkg/test_update_metadata_expiry_v1.py`
- `tests/pkg/test_update_freeze_attack_v1.py`
- `tests/pkg/test_update_mix_and_match_v1.py`
- `tests/pkg/test_update_key_rotation_v1.py`

### Acceptance checks

- `python -m pytest tests/pkg/test_pkg_rebuild_repro_v3.py tests/pkg/test_repo_policy_v3.py tests/pkg/test_update_metadata_expiry_v1.py tests/pkg/test_update_freeze_attack_v1.py tests/pkg/test_update_mix_and_match_v1.py tests/pkg/test_update_key_rotation_v1.py -v`

### Done criteria for PR-2

- Rebuild/metadata integrity and trust checks are deterministic and auditable.

## PR-3: Ecosystem Gate + Update Trust Sub-gate

### Objective

Promote package/repo v3 and update-trust v1 checks to release-blocking status.

### Scope

- Add local gates:
  - `Makefile` target `test-pkg-ecosystem-v3`
  - `Makefile` target `test-update-trust-v1`
- Add CI steps:
  - `Package ecosystem v3 gate`
  - `Update trust v1 gate`
- Add aggregate tests:
  - `tests/pkg/test_pkg_ecosystem_gate_v3.py`
  - `tests/pkg/test_update_trust_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/pkg/test_pkg_ecosystem_gate_v3.py`
- `tests/pkg/test_update_trust_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`

### Acceptance checks

- `make test-pkg-ecosystem-v3`
- `make test-update-trust-v1`

### Done criteria for PR-3

- Ecosystem and update-trust gates are required in local and CI release lanes.
- M26 can be marked done with policy and artifact evidence.

## Non-goals for M26 backlog

- Full third-party package ecosystem breadth parity.
- Skipping trust controls for convenience in release lanes.

