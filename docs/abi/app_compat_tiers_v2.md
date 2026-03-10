# App Compatibility Tiers v2

Date: 2026-03-10  
Milestone: M44 Real Desktop + Ecosystem Qualification v2  
Status: active release gate

## Purpose

Define runtime-qualified app workload tiers and threshold rules used by M44
desktop/ecosystem release gates.

## Contract identifiers

- Tier contract ID: `rugo.app_compat_tiers.v2`.
- Parent desktop profile ID: `rugo.desktop_profile.v2`.
- Runtime matrix schema: `rugo.real_gui_app_matrix_report.v2`.
- Install campaign schema: `rugo.real_pkg_install_campaign_report.v2`.
- Audit schema: `rugo.real_catalog_audit_report.v2`.

## Tier taxonomy

### Tier `tier_productivity_runtime`

- Workload class: `productivity`.
- Focus: desktop productivity suites and document workflows.
- Minimum eligible cases: `8`.
- Minimum pass rate: `0.875`.

### Tier `tier_media_runtime`

- Workload class: `media`.
- Focus: media playback, render, and hardware-assisted decode paths.
- Minimum eligible cases: `6`.
- Minimum pass rate: `0.833`.

### Tier `tier_utility_runtime`

- Workload class: `utility`.
- Focus: platform utilities and desktop tooling workflows.
- Minimum eligible cases: `7`.
- Minimum pass rate: `0.857`.

## Determinism and trust rules

- Every case must carry signed provenance (`signed_provenance=true`).
- Every case must be deterministic (`deterministic=true`).
- Every case must be reproducible (`reproducible=true`).
- Every case must link to runtime trace evidence (`runtime_trace_id`).
- Every case must declare `runtime_source=runtime_capture`.
- Unknown workload classes or tier mismatches are release-blocking.

## Tooling and gate wiring

- Runtime matrix runner: `tools/run_real_gui_app_matrix_v2.py`.
- Install campaign runner: `tools/run_real_pkg_install_campaign_v2.py`.
- Catalog audit runner: `tools/run_real_catalog_audit_v2.py`.
- Local gate: `make test-real-ecosystem-desktop-v2`.
- Local sub-gate: `make test-real-app-catalog-v2`.
- CI gate: `Real ecosystem desktop v2 gate`.
- CI sub-gate: `Real app catalog v2 gate`.
