# Runtime Evidence Schema v1

Date: 2026-03-18  
Milestone: M40 Runtime-Backed Evidence Integrity v1  
Status: active release gate

## Purpose

Define the machine-readable schema contract for runtime-collected evidence used
by M40 release gates.

## Schema identifiers

- Schema ID: `rugo.runtime_evidence_schema.v1`.
- Primary report schema: `rugo.runtime_evidence_report.v1`.
- Parent evidence policy ID: `rugo.evidence_integrity_policy.v1`.
- Gate provenance policy ID: `rugo.gate_provenance_policy.v1`.

## Required top-level fields

- Top-level field: `schema`.
- Top-level field: `created_utc`.
- Top-level field: `traces`.
- Top-level field: `evidence_items`.
- Top-level field: `checks`.
- Top-level field: `summary`.
- Top-level field: `totals`.
- Top-level field: `artifact_refs`.
- Top-level field: `release_image_path`.
- Top-level field: `digest`.

## Required trace object fields

- Trace field: `trace_id`.
- Trace field: `execution_lane`.
- Trace field: `capture_kind`.
- Trace field: `trace_path`.
- Trace field: `trace_digest`.
- Trace field: `release_image_path`.
- Trace field: `boot_id`.

## Required evidence item fields

- Evidence field: `artifact_id`.
- Evidence field: `execution_lane`.
- Evidence field: `runtime_source.kind`.
- Evidence field: `runtime_source.collector`.
- Evidence field: `synthetic`.
- Evidence field: `trace_id`.
- Evidence field: `trace_digest`.
- Evidence field: `artifact_digest`.
- Evidence field: `provenance`.
- Evidence field: `signature.valid`.

## Trace linkage rules

- Every evidence item must link to exactly one trace by `trace_id`.
- Evidence and trace lane values must match.
- `trace_digest` in each evidence item must match the linked trace digest.
- Release image binding must match the default shipped image for release lanes.
- Detached evidence items are release-blocking.

## Determinism rules

- Identical fixture inputs and inject-failure inputs must produce equivalent
  evidence payloads, excluding `created_utc`.
- Check ordering is stable and deterministic.
- Deterministic digest algorithm: `sha256`.

## Required artifacts

- `out/booted-runtime-v1.json`
- `out/perf-baseline-v1.json`
- `out/perf-regression-v1.json`
- `out/trace-bundle-v2.json`
- `out/diagnostic-snapshot-v2.json`
- `out/crash-dump-v1.json`
- `out/crash-dump-symbolized-v1.json`
- `out/runtime-evidence-v1.json`
- `out/gate-evidence-audit-v1.json`
- `out/pytest-evidence-integrity-v1.xml`
- `out/pytest-synthetic-evidence-ban-v1.xml`
