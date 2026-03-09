# Update Key Rotation Policy v1

Date: 2026-03-09  
Milestone: M26 Package/Repo Ecosystem v3  
Status: active required sub-gate

## Purpose

Define deterministic trust-root and signing-key rotation behavior for update
metadata.

## Contract identifiers

- Policy ID: `rugo.update_key_rotation_policy.v1`
- Drill report schema: `rugo.update_key_rotation_drill.v1`

## Rotation process

- Stage 1: `old_key_only`
- Stage 2: `overlap_window`
- Stage 3: `new_key_primary`
- Stage 4: `old_key_revoked`
- Stage 5: `revocation_enforced`

## Mandatory controls

- Overlap window must exist for key handoff.
- Maximum overlap window days: `14`.
- Emergency revoke path must be drill-tested.
- Clients must reject metadata from revoked keys after cutoff.
- Revocation propagation SLA hours: `24`.
- Drill gate outcome: `success` must be `true`.

## Tooling and gate wiring

- Drill tool: `tools/run_update_key_rotation_drill_v1.py`
- Trust checker: `tools/check_update_trust_v1.py`
- Sub-gate: `make test-update-trust-v1`
- Parent gate: `make test-pkg-ecosystem-v3`

