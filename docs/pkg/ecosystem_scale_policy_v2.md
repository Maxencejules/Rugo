# Ecosystem Scale Policy v2

Date: 2026-03-10  
Milestone: M44 Real Desktop + Ecosystem Qualification v2  
Status: active release gate

## Purpose

Define runtime-qualified ecosystem-scale thresholds for desktop app/package
catalog claims under M44.

## Contract identifiers

- Policy ID: `rugo.ecosystem_scale_policy.v2`.
- Distribution workflow ID: `rugo.distribution_workflow.v2`.
- Desktop profile ID: `rugo.desktop_profile.v2`.
- App tier schema ID: `rugo.app_compat_tiers.v2`.
- GUI runtime schema: `rugo.real_gui_app_matrix_report.v2`.
- Install campaign schema: `rugo.real_pkg_install_campaign_report.v2`.
- Catalog audit schema: `rugo.real_catalog_audit_report.v2`.

## Required scale thresholds

- Total catalog entries: `>= 520`.
- Class coverage floor per declared class: `>= 90`.
- Catalog metadata completeness ratio: `>= 0.998`.
- Signed provenance coverage ratio: `>= 1.0`.
- Runtime trace coverage ratio: `>= 1.0`.
- Reproducible install ratio: `>= 0.99`.
- Unsupported workload ratio: `<= 0.01`.
- Policy violation count: `0`.

## Class coverage floors

- `productivity`: `>= 150` entries.
- `devtools`: `>= 120` entries.
- `media`: `>= 90` entries.
- `utility`: `>= 160` entries.

## Gate wiring

- GUI runtime matrix: `tools/run_real_gui_app_matrix_v2.py`.
- Install campaign: `tools/run_real_pkg_install_campaign_v2.py`.
- Catalog reproducibility audit: `tools/run_real_catalog_audit_v2.py`.
- Local gate: `make test-real-ecosystem-desktop-v2`.
- Local sub-gate: `make test-real-app-catalog-v2`.
- CI gate: `Real ecosystem desktop v2 gate`.
- CI sub-gate: `Real app catalog v2 gate`.

## Evidence artifacts

- `out/real-gui-matrix-v2.json`
- `out/real-pkg-install-v2.json`
- `out/real-catalog-audit-v2.json`
- `out/pytest-real-ecosystem-desktop-v2.xml`
- `out/pytest-real-app-catalog-v2.xml`

## Policy boundary

- M44 claims are limited to declared workload classes and thresholds.
- New classes require explicit contract version updates.
