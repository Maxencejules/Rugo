# Maturity Qualification Policy v1

Date: 2026-03-09  
Milestone: M34 Maturity Qualification + LTS Declaration  
Status: active final release gate

## Objective

Define deterministic, machine-verifiable maturity qualification criteria that
aggregate cross-domain release evidence into one final gate.

## Policy identifiers

- Maturity qualification policy ID: `rugo.maturity_qualification_policy.v1`
- Qualification bundle schema: `rugo.maturity_qualification_bundle.v1`
- LTS declaration policy ID: `rugo.lts_declaration_policy.v1`
- LTS declaration schema: `rugo.lts_declaration_report.v1`

## Qualification window requirements

- minimum qualified release count: `3`
- minimum support window for LTS baseline: `730 days`
- all cross-domain evidence checks must pass with `max_failures = 0`
- runtime capture evidence artifact: `out/booted-runtime-v1.json`
- package rebuild evidence artifact: `out/pkg-rebuild-v3.json`

## Required evidence domains

The qualification bundle must include all of the following evidence:

- Security response evidence:
  - advisory lint: `rugo.security_advisory_lint_report.v1`
  - embargo drill: `rugo.security_embargo_drill_report.v1`
- Supply-chain continuity evidence:
  - revalidation: `rugo.supply_chain_revalidation_report.v1`
  - attestation verification: `rugo.release_attestation_verification.v1`
- Rollout safety evidence:
  - canary rollout simulation: `rugo.canary_rollout_report.v1`
  - rollback/abort drill: `rugo.rollout_abort_drill_report.v1`
- Fleet and conformance evidence:
  - fleet update simulation: `rugo.fleet_update_sim_report.v1`
  - fleet health simulation: `rugo.fleet_health_report.v1`
  - profile conformance: `rugo.profile_conformance_report.v1`
  - package rebuild verification: `rugo.pkg_rebuild_report.v3`
- Lifecycle and reliability evidence:
  - release branch audit: `rugo.release_branch_audit.v2`
  - support window audit: `rugo.support_window_audit.v1`
  - measured boot report: `rugo.measured_boot_report.v1`
  - crash dump symbolization: `rugo.crash_dump_symbolized.v1`

## LTS scope

- `server_v1` and `appliance_v1` are the only LTS-qualified profiles.
- `developer_v1` remains outside the LTS declaration surface.
- LTS scope is bound to the `qemu-q35-default-lane` support matrix entry.

## Required upstream gate anchors

- `make test-vuln-response-v1`
- `make test-supply-chain-revalidation-v1`
- `make test-fleet-rollout-safety-v1`
- `make test-fleet-ops-v1`
- `make test-conformance-v1`
- `make test-release-lifecycle-v2`
- `make test-firmware-attestation-v1`
- `make test-crash-dump-v1`

## Enforcement

- Qualification bundle command:
  - `python tools/run_maturity_qualification_v1.py --seed 20260309 --out out/maturity-qualification-v1.json`
- Final local gate: `make test-maturity-qual-v1`
- Final CI gate: `Maturity qualification v1 gate`

Qualification is considered pass only when:

- `qualification_pass = true`
- `lts_declaration.eligible = true`
- `max_failures = 0`
