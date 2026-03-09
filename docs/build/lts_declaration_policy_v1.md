# LTS Declaration Policy v1

Date: 2026-03-09  
Milestone: M34 Maturity Qualification + LTS Declaration  
Status: active declaration policy

## Objective

Define explicit criteria for declaring an auditable LTS baseline after maturity
qualification passes.

## Policy identifiers

- LTS declaration policy ID: `rugo.lts_declaration_policy.v1`
- Declaration report schema: `rugo.lts_declaration_report.v1`
- Qualification dependency schema: `rugo.maturity_qualification_bundle.v1`

## Declaration criteria

- minimum qualified releases: `3`
- minimum support window: `730 days`
- required release channels: `stable`, `lts`
- maximum advisory SLA breach count: `0`
- supply-chain drift tolerance: `0`

All declaration criteria are release-blocking for LTS publication.

## Required declaration evidence

- qualification bundle: `out/maturity-qualification-v1.json`
- support window audit: `out/support-window-audit-v1.json`
- release branch audit: `out/release-branch-audit-v2.json`
- security embargo drill: `out/security-embargo-drill-v1.json`
- advisory lint report: `out/security-advisory-lint-v1.json`

## Publication and governance requirements

- LTS declaration must include version, support window, and evidence artifact IDs.
- Any failed declaration criterion must block release promotion.
- Exceptions require a recorded waiver linked to the failed criterion.

## Required gates

- Maturity gate: `make test-maturity-qual-v1`
- CI gate: `Maturity qualification v1 gate`
