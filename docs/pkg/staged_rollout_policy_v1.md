# Staged Rollout Policy v1

Date: 2026-03-09  
Milestone: M33 Fleet-Scale Operations Baseline v1  
Status: active rollout safety gate

## Objective

Define deterministic stage progression, blast-radius controls, and
promotion/abort criteria for fleet rollouts.

## Policy identifiers

- Staged rollout policy ID: `rugo.staged_rollout_policy.v1`
- Report schema: `rugo.canary_rollout_report.v1`
- Abort drill schema: `rugo.rollout_abort_drill_report.v1`

## Stage budgets

- Stage: `canary`
  - blast-radius budget: `1%`
- Stage: `small_batch`
  - blast-radius budget: `10%`
- Stage: `broad`
  - blast-radius budget: `100%`

## Promotion and halt criteria

- maximum canary error rate: `0.02`
- maximum canary latency p95: `200 ms`
- runtime control plane mode: `runtime_lab`
- runtime capture artifact: `out/booted-runtime-v1.json`
- stage latency caps: `canary=200 ms`, `small_batch=220 ms`, `broad=250 ms`
- Each stage must pass health and SLO checks before promotion.
- Any breach requires automatic halt and rollback coordination.
- Promotion must stop after first failed stage.

## Required report fields

- `stage`
- `blast_radius_budget_pct`
- `observed_error_rate`
- `observed_latency_p95_ms`
- `promoted`
- `auto_halt`
- `rollback_recommended`

## Required gates

- Local sub-gate: `make test-fleet-rollout-safety-v1`
- Parent gate: `make test-fleet-ops-v1`
- CI sub-gate: `Fleet rollout safety v1 gate`
