# Fleet Health Policy v1

Date: 2026-03-09  
Milestone: M33 Fleet-Scale Operations Baseline v1  
Status: active release gate

## Objective

Define deterministic fleet health criteria that gate staged updates and
rollback decisions.

## Policy identifiers

- Fleet health policy ID: `rugo.fleet_health_policy.v1`
- Fleet health simulation schema: `rugo.fleet_health_report.v1`
- Staged rollout schema: `rugo.canary_rollout_report.v1`

## Fleet health states

- `healthy`: all required health checks pass.
- `degraded`: any required health check fails and rollout progression halts.

## SLO and blast-radius controls

- maximum fleet degraded ratio: `0.05`
- maximum fleet error rate: `0.02`
- maximum canary latency p95: `200 ms`
- canary blast-radius budget: `1%`
- runtime capture artifact: `out/booted-runtime-v1.json`

## Required fleet health fields

Each cluster health entry must include:

- `cluster_id`
- `nodes_total`
- `nodes_degraded`
- `degraded_ratio`
- `error_rate`
- `latency_p95_ms`
- `within_slo`

## Enforcement

- Any fleet `degraded` state is gate-blocking.
- Any rollout stage above SLO thresholds is gate-blocking.
- Any breach requires rollback coordination evidence.

## Required gates

- Fleet ops gate: `make test-fleet-ops-v1`
- Rollout safety sub-gate: `make test-fleet-rollout-safety-v1`
- CI gate: `Fleet ops v1 gate`
