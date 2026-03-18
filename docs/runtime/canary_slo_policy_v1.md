# Canary SLO Policy v1

Date: 2026-03-09  
Milestone: M33 Fleet-Scale Operations Baseline v1  
Status: active rollout safety gate

## Objective

Define deterministic SLO-triggered halt and rollback behavior for canary
stages.

## Policy identifiers

- Canary SLO policy ID: `rugo.canary_slo_policy.v1`
- Abort drill schema: `rugo.rollout_abort_drill_report.v1`

## Mandatory threshold fields

- `slo_error_rate_threshold`
- `slo_latency_p95_ms_threshold`
- `observed_error_rate`
- `observed_latency_p95_ms`

## Enforcement fields

- `auto_halt`
- `rollback_triggered`
- `policy_enforced`
- `halt_reason`
- `recovery_actions`

## Threshold baseline

- default canary error-rate threshold: `0.02`
- default canary latency p95 threshold: `200 ms`

## Enforcement rules

- Any SLO breach is gate-blocking.
- Any gate-blocking breach must set `auto_halt = true`.
- Any `auto_halt` outcome must set `rollback_triggered = true`.
- Any missing halt/rollback on breach is policy-violation.

## Required gates

- Local sub-gate: `make test-fleet-rollout-safety-v1`
- Parent gate: `make test-fleet-ops-v1`
- CI sub-gate: `Fleet rollout safety v1 gate`
