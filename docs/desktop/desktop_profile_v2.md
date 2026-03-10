# Desktop Profile v2

Date: 2026-03-10  
Milestone: M44 Real Desktop + Ecosystem Qualification v2  
Status: active release gate

## Objective

Define runtime-qualified desktop and GUI compatibility boundaries for M44.

## Profile identifiers

- Desktop profile ID: `rugo.desktop_profile.v2`
- Desktop runtime schema: `rugo.real_gui_app_matrix_report.v2`
- App tier schema: `rugo.app_compat_tiers.v2`
- Ecosystem policy ID: `rugo.ecosystem_scale_policy.v2`

## Boundaries

In scope:
- runtime-backed GUI workload qualification across declared classes
- signed provenance and trace-linked runtime evidence for every case
- reproducible desktop/app qualification artifacts in release lanes

Out of scope:
- universal GUI parity claims outside declared workload tiers
- support claims without signed runtime provenance

## GUI runtime tiers

| Class | Tier | Minimum eligible cases | Minimum pass rate |
|---|---|---:|---:|
| `productivity` | `tier_productivity_runtime` | 8 | 0.875 |
| `media` | `tier_media_runtime` | 6 | 0.833 |
| `utility` | `tier_utility_runtime` | 7 | 0.857 |

## Runtime evidence requirements

- Signed provenance ratio must be `>= 1.0`.
- Runtime trace coverage ratio must be `>= 1.0`.
- Reproducible execution ratio must be `>= 1.0`.
- Synthetic-only evidence ratio must be `<= 0.0`.

## Gate requirements

- Desktop runtime command:
  - `python tools/run_real_gui_app_matrix_v2.py --out out/real-gui-matrix-v2.json`
- App-catalog runtime commands:
  - `python tools/run_real_pkg_install_campaign_v2.py --out out/real-pkg-install-v2.json`
  - `python tools/run_real_catalog_audit_v2.py --out out/real-catalog-audit-v2.json`
- Local gate: `make test-real-ecosystem-desktop-v2`.
- Local sub-gate: `make test-real-app-catalog-v2`.
- CI gate: `Real ecosystem desktop v2 gate`.
- CI sub-gate: `Real app catalog v2 gate`.

Gate pass requires:

- real GUI matrix `total_failures = 0`
- app-catalog install and audit reports `gate_pass = true`
- runtime evidence issues list to be empty (`issues = []`)
