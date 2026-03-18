# Gate Provenance Policy v1

Date: 2026-03-18  
Milestone: M40 Runtime-Backed Evidence Integrity v1  
Status: active release gate

## Purpose

Define audit checks that enforce runtime provenance for release-bound evidence
and ban synthetic-only artifact sets.

## Policy identifiers

- Policy ID: `rugo.gate_provenance_policy.v1`.
- Parent evidence policy ID: `rugo.evidence_integrity_policy.v1`.
- Runtime evidence schema ID: `rugo.runtime_evidence_schema.v1`.
- Audit report schema: `rugo.gate_evidence_audit_report.v1`.

## Audit execution contract

- Audit tool: `tools/audit_gate_evidence_v1.py`.
- Input evidence schema: `rugo.runtime_evidence_report.v1`.
- Gate pass requires `total_failures = 0`.
- Synthetic-only evidence is release-blocking.
- Runtime lane coverage expects `qemu` and `panic`.

## Required audit checks

- `evidence_file_present`
- `evidence_schema_valid`
- `evidence_policy_id_match`
- `runtime_schema_id_match`
- `gate_provenance_policy_match`
- `runtime_capture_ratio`
- `qemu_trace_presence_ratio`
- `panic_trace_presence_ratio`
- `trace_linkage_ratio`
- `default_image_binding_ratio`
- `boot_instance_binding_ratio`
- `provenance_fields_ratio`
- `synthetic_evidence_ratio`
- `synthetic_only_artifacts`
- `detached_trace_count`
- `unsigned_artifact_count`
- `runtime_lane_coverage`
- `release_image_binding`
- `release_gate_binding`

## Gate wiring

- Local gate: `make test-synthetic-evidence-ban-v1`.
- CI gate: `Synthetic evidence ban v1 gate`.
- Parent gate integration: `make test-evidence-integrity-v1`.

## Required artifacts

- `out/runtime-evidence-v1.json`
- `out/gate-evidence-audit-v1.json`
- `out/pytest-synthetic-evidence-ban-v1.xml`
