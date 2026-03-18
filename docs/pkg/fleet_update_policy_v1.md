# Fleet Update Policy v1

Date: 2026-03-09  
Milestone: M33 Fleet-Scale Operations Baseline v1  
Status: active release gate

## Objective

Define deterministic fleet-wide update orchestration and rollback coordination
rules for staged package rollouts.

## Policy identifiers

- Fleet update policy ID: `rugo.fleet_update_policy.v1`
- Fleet update simulation schema: `rugo.fleet_update_sim_report.v1`
- Rollout simulation schema: `rugo.canary_rollout_report.v1`
- Abort drill schema: `rugo.rollout_abort_drill_report.v1`

## Update orchestration contract

- Fleet update simulation must evaluate at least three node groups:
  `canary`, `batch_a`, `batch_b`.
- runtime control plane mode: `runtime_lab`
- runtime capture artifact: `out/booted-runtime-v1.json`
- Each node group includes:
  - `target_version`
  - `current_version`
  - `nodes_total`
  - `nodes_updated`
  - `success_rate`
  - `rollback_triggered`
- Group promotion requires success rate meeting or exceeding `0.98`.
- Any group below threshold must trigger rollback coordination and block further
  promotion.

## Rollback coordination rules

- Rollback is mandatory when:
  - update success rate drops below `0.98`
  - canary SLO gate reports `auto_halt = true`
  - fleet health gate reports fleet `degraded`
- Rollback outcome must be recorded as release artifact evidence.

## Enforcement

- Fleet update simulation command:
  - `python tools/run_fleet_update_sim_v1.py --seed 20260309 --out out/fleet-update-sim-v1.json`
- Rollout safety sub-gate command:
  - `make test-fleet-rollout-safety-v1`

Fleet operations pass only when `max_failures = 0`.

## Required gates

- Local gate: `make test-fleet-ops-v1`
- CI gate: `Fleet ops v1 gate`
- CI sub-gate: `Fleet rollout safety v1 gate`
