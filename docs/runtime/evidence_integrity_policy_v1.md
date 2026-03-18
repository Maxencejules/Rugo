# Evidence Integrity Policy v1

Date: 2026-03-18  
Milestone: M40 Runtime-Backed Evidence Integrity v1  
Status: active release gate

## Purpose

Define runtime-evidence requirements for release-bound artifacts and make
synthetic-only evidence release-blocking.

## Contract identifiers

- Policy ID: `rugo.evidence_integrity_policy.v1`.
- Runtime evidence schema ID: `rugo.runtime_evidence_schema.v1`.
- Gate provenance policy ID: `rugo.gate_provenance_policy.v1`.
- Runtime evidence report schema: `rugo.runtime_evidence_report.v1`.
- Gate evidence audit schema: `rugo.gate_evidence_audit_report.v1`.

## Required runtime provenance thresholds

- Runtime item count: `>= 7.0`.
- Runtime capture ratio: `>= 1.0`.
- QEMU trace presence ratio: `>= 1.0`.
- Panic trace presence ratio: `>= 1.0`.
- Trace linkage ratio: `>= 1.0`.
- Default image binding ratio: `>= 1.0`.
- Boot instance binding ratio: `>= 1.0`.
- Provenance fields ratio: `>= 1.0`.
- Synthetic evidence ratio: `<= 0.0`.
- Synthetic-only artifact flag: `0`.
- Detached trace count: `0`.
- Unsigned artifact count: `0`.

## Required execution lanes

- `qemu`
- `panic`

Both lanes must emit runtime-captured traces linked to release evidence from
the default shipped image flow.

## Tooling and gate wiring

- Booted runtime capture tool: `tools/collect_booted_runtime_v1.py`.
- Runtime evidence collector: `tools/collect_runtime_evidence_v1.py`.
- Gate evidence audit tool: `tools/audit_gate_evidence_v1.py`.
- Local gate: `make test-evidence-integrity-v1`.
- Local sub-gate: `make test-synthetic-evidence-ban-v1`.
- CI gate: `Evidence integrity v1 gate`.
- CI sub-gate: `Synthetic evidence ban v1 gate`.

## Required executable checks

- `tests/runtime/test_booted_runtime_capture_v1.py`
- `tests/runtime/test_evidence_integrity_docs_v1.py`
- `tests/runtime/test_runtime_evidence_collection_v1.py`
- `tests/runtime/test_gate_evidence_audit_v1.py`
- `tests/runtime/test_evidence_trace_linkage_v1.py`
- `tests/runtime/test_evidence_integrity_gate_v1.py`
- `tests/runtime/test_synthetic_evidence_ban_v1.py`
