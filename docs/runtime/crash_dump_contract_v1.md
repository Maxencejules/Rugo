# Crash Dump Contract v1

Date: 2026-03-18  
Milestone: M29 Observability + Diagnostics v2  
Status: active required sub-gate

## Purpose

Define boot-backed panic-dump capture and symbolization requirements for
release lanes.

## Contract identifiers

- Crash dump contract ID: `rugo.crash_dump_contract.v1`
- Dump schema: `rugo.crash_dump.v1`
- Symbolized schema: `rugo.crash_dump_symbolized.v1`
- Symbol map ID: `rugo.kernel_symbol_map.v1`
- Triage playbook linkage: `rugo.postmortem_triage_playbook.v1`

## Required dump fields

- `panic_code` and `panic_reason`.
- Register set with `rip`, `rsp`, and `rbp`.
- Ordered `stack_frames` list with `ip` and `offset`.
- Runtime provenance with `release_image_path`, `panic_image_path`,
  `panic_boot_id`, `panic_trace_id`, and `serial_digest`.
- Release identity with `kernel_build_id` and `release_channel`.

## Symbolization and retention policy

- Symbolization must preserve frame order and annotate each frame with a
  symbol.
- Symbolization must preserve `runtime_provenance`.
- Release lanes require `unresolved_frames == 0`.
- Default panic lane boots `out/os-panic.iso` while preserving linkage to
  `out/os-go.iso`.
- Dump retention minimum: `30` days.
- Symbol map retention minimum: `90` days.
- Triage handoff must include deterministic artifact references.

## Tooling and gate wiring

- Capture tool: `tools/collect_crash_dump_v1.py`
- Symbolizer tool: `tools/symbolize_crash_dump_v1.py`
- Local sub-gate: `make test-crash-dump-v1`
- Parent gate: `make test-observability-v2`

## Required executable checks

- `tests/runtime/test_crash_dump_docs_v1.py`
- `tests/runtime/test_crash_dump_capture_v1.py`
- `tests/runtime/test_crash_dump_symbolization_v1.py`
- `tests/runtime/test_crash_dump_gate_v1.py`
