# Distribution Workflow v2

Date: 2026-03-10  
Milestone: M44 Real Desktop + Ecosystem Qualification v2  
Status: active release gate

## Purpose

Define runtime-qualified distribution stages and reproducibility controls for
ecosystem claims under M44.

## Policy identity

- Policy ID: `rugo.distribution_workflow.v2`.
- Parent ecosystem policy ID: `rugo.ecosystem_scale_policy.v2`.
- Parent desktop profile ID: `rugo.desktop_profile.v2`.
- Workflow report schema: `rugo.real_catalog_audit_report.v2`.
- Install report schema: `rugo.real_pkg_install_campaign_report.v2`.

## Required workflow stages

- `ingest`
- `vet`
- `sign`
- `runtime_qualify`
- `stage`
- `rollout`
- `rollback`

## Runtime workflow thresholds

- Workflow stage completeness ratio: `>= 1.0`.
- Release signoff ratio: `>= 1.0`.
- Rollback drill pass ratio: `>= 1.0`.
- Mirror index consistency ratio: `>= 1.0`.
- Replication lag p95 minutes: `<= 10`.
- Runtime trace coverage ratio: `>= 1.0`.
- Signed artifact ratio: `>= 1.0`.
- Unresolved policy exceptions: `0`.

## Gate wiring

- Workflow audit runner: `tools/run_real_catalog_audit_v2.py`.
- Install quality runner: `tools/run_real_pkg_install_campaign_v2.py`.
- Local gate: `make test-real-ecosystem-desktop-v2`.
- Local sub-gate: `make test-real-app-catalog-v2`.
- CI gate: `Real ecosystem desktop v2 gate`.
- CI sub-gate: `Real app catalog v2 gate`.

## Failure handling

- Any failed workflow stage blocks release promotion.
- Any runtime trace linkage gap blocks release promotion.
- Any signed artifact ratio regression is release-blocking.
- Distribution claims are bounded to this workflow version.
