# Compatibility Profile v3

Date: 2026-03-09  
Milestone: M27 External App Compatibility Program v3  
Status: active release gate

## Purpose

Scale external application compatibility from curated demos to practical
workload classes with deterministic, repeatable thresholds.

## Contract identity

- Compatibility profile identifier: `rugo.compat_profile.v3`.
- App compatibility tier schema: `rugo.app_compat_tiers.v1`.
- App compatibility report schema: `rugo.app_compat_matrix_report.v3`.

## Compatibility classes

- `required`: release-blocking for v3.
- `optional`: may be covered by future milestones.
- `unsupported`: explicitly out of scope and must fail deterministically.

## Required workload classes (`required`)

### CLI workload class (`cli`)

- Tier: `tier_cli`.
- Scope: non-interactive command-line tools and shell-oriented utilities.
- Constraints:
  - deterministic stdout/stderr markers,
  - deterministic process exit status,
  - signed package artifact provenance.

Conformance coverage:
- `tests/compat/test_cli_suite_v3.py`

### Runtime workload class (`runtime`)

- Tier: `tier_runtime`.
- Scope: language/runtime-driven apps (for example stock-Go style workloads).
- Constraints:
  - deterministic runtime startup and process lifecycle markers,
  - deterministic dynamic/runtime dependency behavior,
  - ABI profile pinning to `compat_profile_v3`.

Conformance coverage:
- `tests/compat/test_runtime_suite_v3.py`

### Service workload class (`service`)

- Tier: `tier_service`.
- Scope: long-lived service processes with bounded restart behavior.
- Constraints:
  - deterministic service boot/readiness markers,
  - deterministic dependency and restart semantics,
  - signed deterministic deployment artifacts.

Conformance coverage:
- `tests/compat/test_service_suite_v3.py`

## Optional classes (`optional`)

- `cli_interactive`: interactive terminal fidelity expansion beyond baseline
  non-interactive tooling checks.
- `runtime_jit`: high-end runtime/JIT optimization behavior checks.

## Explicit unsupported list (`unsupported`)

- Desktop/GUI application compatibility claims.
- Full Linux ABI parity across unsupported syscall families.
- Container orchestration/runtime parity surfaces not declared in v3 contracts.

## Tier thresholds

| Class | Tier | Minimum eligible cases | Minimum pass rate |
|------|------|------------------------|-------------------|
| `cli` | `tier_cli` | 14 | 0.90 |
| `runtime` | `tier_runtime` | 10 | 0.80 |
| `service` | `tier_service` | 8 | 0.80 |

Additional gate rules:

- Unsigned artifacts are release-blocking.
- Non-deterministic case results are release-blocking.
- ABI profile mismatch (`compat_profile_v3` expected) is release-blocking.

## Conformance and release gate

- Local gate: `make test-app-compat-v3`
- Required tool: `tools/run_app_compat_matrix_v3.py`
- Required tests:
  - `tests/compat/test_app_tier_docs_v1.py`
  - `tests/compat/test_cli_suite_v3.py`
  - `tests/compat/test_runtime_suite_v3.py`
  - `tests/compat/test_service_suite_v3.py`
  - `tests/compat/test_app_compat_gate_v3.py`
- CI step: `App compatibility v3 gate`

## Related contracts

- `docs/abi/app_compat_tiers_v1.md`
- `docs/abi/compat_profile_v2.md`
