# M73 Execution Backlog (Developer SDK + Porting Kit v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Add a versioned SDK and porting kit so external developers can build and port
apps against Rugo's declared ABI and desktop profiles.

M73 source of truth remains:

- `docs/POST_G2_EXTENDED_MILESTONES.md`
- `docs/M11_EXECUTION_BACKLOG.md`
- `docs/M72_EXECUTION_BACKLOG.md`
- this backlog

## Current State Summary

- Runtime maturity and compatibility surface work provide the low-level base
  for external application development.
- App-bundle work in M72 defines how third-party apps will be distributed.
- There is no versioned SDK contract, app-porting profile, or SDK
  reproducibility policy in-tree.
- M73 must define those semantics before catalog federation scales community
  contributions.

## Execution plan

- PR-1: SDK and porting contract freeze
- PR-2: template build and repro campaign baseline
- PR-3: SDK gate wiring + closure

## Execution status

- PR-1: pending
- PR-2: pending
- PR-3: pending

## PR-1: SDK and Porting Contract Freeze

### Objective

Define the SDK surface, porting profile, and repro policy before external
developer claims broaden.

### Scope

- Add docs:
  - `docs/pkg/sdk_contract_v1.md`
  - `docs/abi/app_porting_profile_v1.md`
  - `docs/build/sdk_repro_policy_v1.md`
- Add tests:
  - `tests/pkg/test_sdk_docs_v1.py`

### Primary files

- `docs/pkg/sdk_contract_v1.md`
- `docs/abi/app_porting_profile_v1.md`
- `docs/build/sdk_repro_policy_v1.md`
- `tests/pkg/test_sdk_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/pkg/test_sdk_docs_v1.py -v`

### Done criteria for PR-1

- SDK and porting semantics are explicit and versioned.
- Template expectations and reproducibility knobs are reviewable before
  implementation lands.

## PR-2: Template Build and Repro Campaign Baseline

### Objective

Implement deterministic evidence for template builds, porting-profile checks,
and SDK reproducibility.

### Scope

- Add tooling:
  - `tools/run_sdk_template_builds_v1.py`
  - `tools/run_sdk_repro_audit_v1.py`
- Add tests:
  - `tests/pkg/test_sdk_template_builds_v1.py`
  - `tests/compat/test_app_porting_profile_v1.py`
  - `tests/build/test_sdk_reproducibility_v1.py`
  - `tests/pkg/test_sdk_negative_v1.py`

### Primary files

- `tools/run_sdk_template_builds_v1.py`
- `tools/run_sdk_repro_audit_v1.py`
- `tests/pkg/test_sdk_template_builds_v1.py`
- `tests/compat/test_app_porting_profile_v1.py`
- `tests/build/test_sdk_reproducibility_v1.py`
- `tests/pkg/test_sdk_negative_v1.py`

### Acceptance checks

- `python tools/run_sdk_template_builds_v1.py --out out/sdk-templates-v1.json`
- `python tools/run_sdk_repro_audit_v1.py --out out/sdk-repro-v1.json`
- `python -m pytest tests/pkg/test_sdk_template_builds_v1.py tests/compat/test_app_porting_profile_v1.py tests/build/test_sdk_reproducibility_v1.py tests/pkg/test_sdk_negative_v1.py -v`

### Done criteria for PR-2

- SDK artifacts are deterministic and machine-readable.
- `SDK: template ok` and repro markers are stable.
- Later catalog and build-farm work can depend on one explicit SDK contract.

## PR-3: SDK Gate + Repro Sub-gate

### Objective

Make SDK and porting behavior release-blocking for declared developer flows.

### Scope

- Add local gates:
  - `Makefile` target `test-sdk-porting-v1`
  - `Makefile` target `test-sdk-repro-v1`
- Add CI steps:
  - `SDK porting v1 gate`
  - `SDK repro v1 gate`
- Add aggregate tests:
  - `tests/pkg/test_sdk_porting_gate_v1.py`
  - `tests/build/test_sdk_repro_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/pkg/test_sdk_porting_gate_v1.py`
- `tests/build/test_sdk_repro_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-sdk-porting-v1`
- `make test-sdk-repro-v1`

### Done criteria for PR-3

- SDK and repro sub-gates are required in local and CI release lanes.
- M73 can be marked done only with deterministic template and reproducibility
  evidence for the declared SDK profile.

## Non-goals for M73 backlog

- federated catalog and moderation work owned by M74
- full third-party IDE integration breadth
- unversioned or host-drift-sensitive SDK flows
