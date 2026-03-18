# M29 Execution Backlog (Observability + Diagnostics v2)

Date: 2026-03-06  
Lane: Rugo (Rust kernel + Go user space)  
Status: done

## Goal

Make operational diagnosis first-class with stable telemetry, tracing,
diagnostic bundles, and crash-dump postmortem flow.

M29 source of truth remains `docs/M21_M34_MATURITY_PARITY_ROADMAP.md`,
`MILESTONES.md`, and this backlog.

## Current State Summary

- Observability contract v2 and crash/postmortem contracts are explicit and
  versioned.
- Boot-backed trace bundle, diagnostic snapshot, crash dump, and crash
  symbolization artifacts are implemented.
- Observability v2 and crash-dump v1 are wired as required local and CI gates
  from the shipped image path.

## Execution Result

- PR-1: complete (2026-03-09)
- PR-2: complete (2026-03-09)
- PR-3: complete (2026-03-09)

## PR-1: Observability + Crash Dump Contracts

### Objective

Freeze observability and crash-dump contract semantics.

### Scope

- Add docs:
  - `docs/runtime/observability_contract_v2.md`
  - `docs/runtime/crash_dump_contract_v1.md`
  - `docs/runtime/postmortem_triage_playbook_v1.md`
- Add tests:
  - `tests/runtime/test_observability_docs_v2.py`
  - `tests/runtime/test_crash_dump_docs_v1.py`

### Primary files

- `docs/runtime/observability_contract_v2.md`
- `docs/runtime/crash_dump_contract_v1.md`
- `docs/runtime/postmortem_triage_playbook_v1.md`
- `tests/runtime/test_observability_docs_v2.py`
- `tests/runtime/test_crash_dump_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/runtime/test_observability_docs_v2.py tests/runtime/test_crash_dump_docs_v1.py -v`

### Done criteria for PR-1

- Observability/crash-dump contracts are versioned and test-referenced.

### PR-1 completion summary

- Added observability and postmortem contract docs:
  - `docs/runtime/observability_contract_v2.md`
  - `docs/runtime/crash_dump_contract_v1.md`
  - `docs/runtime/postmortem_triage_playbook_v1.md`
- Added executable PR-1 doc contract checks:
  - `tests/runtime/test_observability_docs_v2.py`
  - `tests/runtime/test_crash_dump_docs_v1.py`

## PR-2: Trace/Diagnostic + Crash Pipeline Tooling

### Objective

Generate boot-backed observability and crash postmortem artifacts.

### Scope

- Add tooling:
  - `tools/collect_booted_runtime_v1.py`
  - `tools/runtime_capture_common_v1.py`
  - `tools/collect_trace_bundle_v2.py`
  - `tools/collect_diagnostic_snapshot_v2.py`
  - `tools/collect_crash_dump_v1.py`
  - `tools/symbolize_crash_dump_v1.py`
- Add tests:
  - `tests/runtime/test_booted_runtime_capture_v1.py`
  - `tests/runtime/test_trace_bundle_v2.py`
  - `tests/runtime/test_diag_snapshot_v2.py`
  - `tests/runtime/test_crash_dump_capture_v1.py`
  - `tests/runtime/test_crash_dump_symbolization_v1.py`

### Primary files

- `tools/collect_booted_runtime_v1.py`
- `tools/runtime_capture_common_v1.py`
- `tools/collect_trace_bundle_v2.py`
- `tools/collect_diagnostic_snapshot_v2.py`
- `tools/collect_crash_dump_v1.py`
- `tools/symbolize_crash_dump_v1.py`
- `tests/runtime/test_booted_runtime_capture_v1.py`
- `tests/runtime/test_trace_bundle_v2.py`
- `tests/runtime/test_diag_snapshot_v2.py`
- `tests/runtime/test_crash_dump_capture_v1.py`
- `tests/runtime/test_crash_dump_symbolization_v1.py`

### Acceptance checks

- `python tools/collect_booted_runtime_v1.py --image out/os-go.iso --kernel out/kernel-go.elf --panic-image out/os-panic.iso --out out/booted-runtime-v1.json`
- `python tools/collect_trace_bundle_v2.py --runtime-capture out/booted-runtime-v1.json --window-seconds 300 --out out/trace-bundle-v2.json`
- `python tools/collect_diagnostic_snapshot_v2.py --runtime-capture out/booted-runtime-v1.json --trace-bundle out/trace-bundle-v2.json --out out/diagnostic-snapshot-v2.json`
- `python tools/collect_crash_dump_v1.py --release-image out/os-go.iso --kernel out/kernel-go.elf --panic-image out/os-panic.iso --out out/crash-dump-v1.json`
- `python tools/symbolize_crash_dump_v1.py --dump out/crash-dump-v1.json --out out/crash-dump-symbolized-v1.json`
- `python -m pytest tests/runtime/test_booted_runtime_capture_v1.py tests/runtime/test_trace_bundle_v2.py tests/runtime/test_diag_snapshot_v2.py tests/runtime/test_crash_dump_capture_v1.py tests/runtime/test_crash_dump_symbolization_v1.py -v`

### Done criteria for PR-2

- Observability and crash artifacts are machine-readable and boot-backed.
- Symbolized postmortem pipeline is reproducible.

### PR-2 completion summary

- Added boot-backed runtime capture and observability tooling:
  - `tools/collect_booted_runtime_v1.py`
  - `tools/runtime_capture_common_v1.py`
  - `tools/collect_trace_bundle_v2.py`
  - `tools/collect_diagnostic_snapshot_v2.py`
- Upgraded boot-backed crash postmortem tooling:
  - `tools/collect_crash_dump_v1.py`
  - `tools/symbolize_crash_dump_v1.py`
- Added executable PR-2 artifact checks:
  - `tests/runtime/test_booted_runtime_capture_v1.py`
  - `tests/runtime/test_trace_bundle_v2.py`
  - `tests/runtime/test_diag_snapshot_v2.py`
  - `tests/runtime/test_crash_dump_capture_v1.py`
  - `tests/runtime/test_crash_dump_symbolization_v1.py`

## PR-3: Observability v2 Gate + Crash Sub-gate

### Objective

Make observability and crash-dump checks release-blocking.

### Scope

- Add local gates:
  - `Makefile` target `test-observability-v2`
  - `Makefile` target `test-crash-dump-v1`
- Add CI steps:
  - `Observability v2 gate`
  - `Crash dump v1 gate`
- Add aggregate tests:
  - `tests/runtime/test_observability_gate_v2.py`
  - `tests/runtime/test_crash_dump_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/runtime/test_observability_gate_v2.py`
- `tests/runtime/test_crash_dump_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`

### Acceptance checks

- `make test-observability-v2`
- `make test-crash-dump-v1`

### Done criteria for PR-3

- Observability and crash-dump gates are required in local and CI lanes.
- M29 can be marked done with trace/diagnostic/postmortem evidence tied to the
  shipped default image and panic lane.

### PR-3 completion summary

- Added aggregate gate tests:
  - `tests/runtime/test_observability_gate_v2.py`
  - `tests/runtime/test_crash_dump_gate_v1.py`
- Added local gates:
  - `make test-observability-v2`
  - `make test-crash-dump-v1`
  - runtime capture artifact: `out/booted-runtime-v1.json`
  - JUnit outputs:
    - `out/pytest-observability-v2.xml`
    - `out/pytest-crash-dump-v1.xml`
- Added CI gates + artifact uploads:
  - step: `Observability v2 gate`
  - artifact: `observability-v2-artifacts`
  - step: `Crash dump v1 gate`
  - artifact: `crash-dump-v1-artifacts`
- Updated closure docs:
  - `MILESTONES.md`
  - `docs/STATUS.md`
  - `README.md`

## Non-goals for M29 backlog

- Full production observability backend deployment.
- Infinite-retention artifact storage policy in this milestone.
