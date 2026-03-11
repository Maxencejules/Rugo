# M81 Execution Backlog (Reliability Fuzzing + Chaos Qualification v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Add bounded chaos and fuzz qualification semantics so reliability becomes a
measured, release-gated property rather than an anecdotal one.

M81 source of truth remains:

- `docs/POST_G2_EXTENDED_MILESTONES.md`
- `docs/M22_EXECUTION_BACKLOG.md`
- `docs/M80_EXECUTION_BACKLOG.md`
- this backlog

## Current State Summary

- Kernel reliability, security fuzzing, and runtime evidence already exist as
  building blocks.
- There is no versioned chaos-qualification, fuzz-coverage, or reliability-SLO
  contract in the post-M80 plan.
- Long-run reliability evidence is not yet unified under one declared gate.
- M81 must define those semantics before community and release operations build
  on them.

## Execution plan

- PR-1: chaos and fuzz contract freeze
- PR-2: chaos campaign and coverage baseline
- PR-3: reliability gate wiring + closure

## Execution status

- PR-1: pending
- PR-2: pending
- PR-3: pending

## PR-1: Chaos and Fuzz Contract Freeze

### Objective

Define chaos-qualification, fuzz-coverage, and reliability-SLO semantics before
implementation broadens stability claims.

### Scope

- Add docs:
  - `docs/runtime/chaos_qualification_policy_v1.md`
  - `docs/security/fuzz_coverage_contract_v1.md`
  - `docs/build/reliability_slo_v1.md`
- Add tests:
  - `tests/runtime/test_chaos_docs_v1.py`
  - `tests/security/test_fuzz_coverage_docs_v1.py`

### Primary files

- `docs/runtime/chaos_qualification_policy_v1.md`
- `docs/security/fuzz_coverage_contract_v1.md`
- `docs/build/reliability_slo_v1.md`
- `tests/runtime/test_chaos_docs_v1.py`
- `tests/security/test_fuzz_coverage_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/runtime/test_chaos_docs_v1.py tests/security/test_fuzz_coverage_docs_v1.py -v`

### Done criteria for PR-1

- Chaos, fuzz-coverage, and reliability-SLO semantics are explicit and
  versioned.
- Seed handling and unsupported-drill paths remain deterministic.

## PR-2: Chaos Campaign and Coverage Baseline

### Objective

Implement deterministic evidence for chaos campaigns, fuzz-coverage budgets,
and reliability SLO evaluation.

### Scope

- Add tooling:
  - `tools/run_chaos_campaign_v1.py`
  - `tools/run_fuzz_coverage_audit_v1.py`
- Add tests:
  - `tests/runtime/test_chaos_campaign_v1.py`
  - `tests/security/test_fuzz_coverage_budget_v1.py`
  - `tests/runtime/test_reliability_slo_v1.py`
  - `tests/runtime/test_chaos_negative_v1.py`

### Primary files

- `tools/run_chaos_campaign_v1.py`
- `tools/run_fuzz_coverage_audit_v1.py`
- `tests/runtime/test_chaos_campaign_v1.py`
- `tests/security/test_fuzz_coverage_budget_v1.py`
- `tests/runtime/test_reliability_slo_v1.py`
- `tests/runtime/test_chaos_negative_v1.py`

### Acceptance checks

- `python tools/run_chaos_campaign_v1.py --out out/chaos-campaign-v1.json`
- `python tools/run_fuzz_coverage_audit_v1.py --out out/fuzz-coverage-v1.json`
- `python -m pytest tests/runtime/test_chaos_campaign_v1.py tests/security/test_fuzz_coverage_budget_v1.py tests/runtime/test_reliability_slo_v1.py tests/runtime/test_chaos_negative_v1.py -v`

### Done criteria for PR-2

- Chaos and fuzz artifacts are deterministic and machine-readable.
- `CHAOS: pass seed=20260311` and fuzz-budget markers are stable.
- Reliability consumers can reference one explicit SLO contract and seeded drill
  model.

## PR-3: Kernel Reliability v2 Gate + Chaos Qualification Sub-gate

### Objective

Make chaos and fuzz qualification release-blocking for declared reliability
profiles.

### Scope

- Add local gates:
  - `Makefile` target `test-kernel-reliability-v2`
  - `Makefile` target `test-chaos-qualification-v1`
- Add CI steps:
  - `Kernel reliability v2 gate`
  - `Chaos qualification v1 gate`
- Add aggregate tests:
  - `tests/runtime/test_kernel_reliability_gate_v2.py`
  - `tests/runtime/test_chaos_qualification_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/runtime/test_kernel_reliability_gate_v2.py`
- `tests/runtime/test_chaos_qualification_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-kernel-reliability-v2`
- `make test-chaos-qualification-v1`

### Done criteria for PR-3

- Reliability and chaos-qualification sub-gates are required in local and CI
  release lanes.
- M81 can be marked done only with deterministic chaos, fuzz, and SLO evidence
  for declared reliability profiles.

## Non-goals for M81 backlog

- community governance and release-train work owned by M82-M84
- unseeded or ad hoc chaos runs outside declared contract scope
- treating flaky evidence as acceptable for milestone closure
