# Observability Contract v2

Date: 2026-03-18  
Milestone: M29 Observability + Diagnostics v2  
Status: active release gate

## Purpose

Define the runtime observability contract used for boot-backed diagnosis,
incident triage, and release-blocking evidence generation.

## Contract identifiers

- Observability contract ID: `rugo.observability_contract.v2`
- Booted runtime schema: `rugo.booted_runtime_capture.v1`
- Trace bundle schema: `rugo.trace_bundle.v2`
- Diagnostic snapshot schema: `rugo.diagnostic_snapshot.v2`
- Crash dump schema: `rugo.crash_dump.v1`
- Symbolized crash schema: `rugo.crash_dump_symbolized.v1`
- Postmortem triage playbook: `rugo.postmortem_triage_playbook.v1`

## Required telemetry surfaces

- Structured log stream for `goinit`, `gosvcm`, `timesvc`, `diagsvc`, `shell`,
  `storage`, `network`, and `isolation`.
- Diagnostic health checks for `service_manager`, `memory_pressure`,
  `filesystem_recovery`, `network_stack`, and `isolation_observer`.
- Crash dump capture containing panic code, register set, stack frames, and
  runtime provenance.
- Cross-layer identifiers: `build_id`, `trace_id`, `boot_id`, and
  `panic_trace_id`.
- Symbolization output bound to the release symbol map identifier and the
  panic provenance bundle.

## Gate thresholds

- Trace window seconds: `300`.
- Maximum allowed trace errors: `0`.
- Maximum allowed dropped trace spans: `0`.
- Maximum allowed unhealthy diagnostic checks: `0`.
- Maximum allowed unresolved crash frames: `0`.

## Tooling and gate wiring

- Booted runtime capture tool: `tools/collect_booted_runtime_v1.py`
- Trace bundle tool: `tools/collect_trace_bundle_v2.py`
- Diagnostic snapshot tool: `tools/collect_diagnostic_snapshot_v2.py`
- Crash dump capture tool: `tools/collect_crash_dump_v1.py`
- Crash dump symbolizer: `tools/symbolize_crash_dump_v1.py`
- Default release image: `out/os-go.iso`
- Local gate: `make test-observability-v2`
- Sub-gate: `make test-crash-dump-v1`
- CI steps:
  - `Observability v2 gate`
  - `Crash dump v1 gate`

## Required executable checks

- `tests/runtime/test_booted_runtime_capture_v1.py`
- `tests/runtime/test_observability_docs_v2.py`
- `tests/runtime/test_trace_bundle_v2.py`
- `tests/runtime/test_diag_snapshot_v2.py`
- `tests/runtime/test_observability_gate_v2.py`
- `tests/runtime/test_crash_dump_docs_v1.py`
- `tests/runtime/test_crash_dump_capture_v1.py`
- `tests/runtime/test_crash_dump_symbolization_v1.py`
- `tests/runtime/test_crash_dump_gate_v1.py`
