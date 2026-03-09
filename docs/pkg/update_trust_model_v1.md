# Update Trust Model v1

Date: 2026-03-09  
Milestone: M26 Package/Repo Ecosystem v3  
Status: active required sub-gate

## Purpose

Define deterministic trust checks that block unsafe update metadata in release
lanes.

## Contract identifiers

- Trust Model ID: `rugo.update_trust_model.v1`
- Report schema: `rugo.update_trust_report.v1`

## Threat classes covered

- rollback attack
- replay/freeze attack
- metadata expiry bypass attack
- mix-and-match target metadata attack
- revoked-key acceptance attack

## Verification requirements

- Metadata expiry is mandatory and validated against current UTC.
- Monotonic sequence is enforced per channel and rejects rollback/replay.
- Signed metadata must bind target set and digest set.
- Metadata signed by revoked keys must be rejected after cutoff.
- Trust report gate outcome: `total_failures` must be `0`.
- Maximum allowed trust failures: `0`.

## Tooling and gate wiring

- Trust checker: `tools/check_update_trust_v1.py`
- Key-rotation drill: `tools/run_update_key_rotation_drill_v1.py`
- Sub-gate: `make test-update-trust-v1`
- Parent gate: `make test-pkg-ecosystem-v3`

## Required executable tests

- `tests/pkg/test_update_trust_docs_v1.py`
- `tests/pkg/test_update_metadata_expiry_v1.py`
- `tests/pkg/test_update_freeze_attack_v1.py`
- `tests/pkg/test_update_mix_and_match_v1.py`
- `tests/pkg/test_update_key_rotation_v1.py`
- `tests/pkg/test_update_trust_gate_v1.py`
