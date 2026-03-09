# Postmortem Triage Playbook v1

Date: 2026-03-09  
Milestone: M29 Observability + Diagnostics v2  
Status: active required sub-gate contract

## Purpose

Define deterministic panic-to-triage workflow and artifact linkage rules used in
release incident response.

## Contract identifiers

- Playbook ID: `rugo.postmortem_triage_playbook.v1`
- Triage bundle schema: `rugo.postmortem_triage_bundle.v1`
- Crash dump input schema: `rugo.crash_dump.v1`
- Symbolized input schema: `rugo.crash_dump_symbolized.v1`

## Workflow

1. Capture crash dump via `tools/collect_crash_dump_v1.py`.
2. Symbolize via `tools/symbolize_crash_dump_v1.py`.
3. Classify incident (`panic`, `deadlock`, `oom`, `data_corruption`).
4. Record remediation owner and regression test linkage.
5. Publish triage bundle pointer in release evidence.

## Required triage fields

- `incident_id`
- `panic_code`
- `root_cause_class`
- `affected_release`
- `symbol_map_id`
- `remediation_ticket`
- `regression_test_ids`

## SLA and retention

- Initial triage SLA: `4` hours.
- Root-cause classification SLA: `24` hours.
- Dump retention minimum: `30` days.
- Symbol map retention minimum: `90` days.
- Triage bundles must be reproducible from stored artifacts.

## Gate linkage

- Sub-gate: `make test-crash-dump-v1`
- Parent gate: `make test-observability-v2`
