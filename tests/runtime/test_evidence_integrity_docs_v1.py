"""M40 PR-1: evidence integrity policy and schema doc contract checks."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_m40_pr1_evidence_integrity_artifacts_exist():
    required = [
        "docs/M40_EXECUTION_BACKLOG.md",
        "docs/runtime/evidence_integrity_policy_v1.md",
        "docs/runtime/runtime_evidence_schema_v1.md",
        "docs/runtime/gate_provenance_policy_v1.md",
        "tools/collect_booted_runtime_v1.py",
        "tests/runtime/test_booted_runtime_capture_v1.py",
        "tests/runtime/test_evidence_integrity_docs_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M40 PR-1 artifact: {rel}"


def test_evidence_integrity_policy_v1_doc_declares_required_tokens():
    doc = _read("docs/runtime/evidence_integrity_policy_v1.md")
    for token in [
        "Policy ID: `rugo.evidence_integrity_policy.v1`.",
        "Runtime evidence schema ID: `rugo.runtime_evidence_schema.v1`.",
        "Gate provenance policy ID: `rugo.gate_provenance_policy.v1`.",
        "Runtime evidence report schema: `rugo.runtime_evidence_report.v1`.",
        "Gate evidence audit schema: `rugo.gate_evidence_audit_report.v1`.",
        "Runtime item count: `>= 7.0`.",
        "Runtime capture ratio: `>= 1.0`.",
        "QEMU trace presence ratio: `>= 1.0`.",
        "Panic trace presence ratio: `>= 1.0`.",
        "Trace linkage ratio: `>= 1.0`.",
        "Default image binding ratio: `>= 1.0`.",
        "Boot instance binding ratio: `>= 1.0`.",
        "Provenance fields ratio: `>= 1.0`.",
        "Synthetic evidence ratio: `<= 0.0`.",
        "Synthetic-only artifact flag: `0`.",
        "Detached trace count: `0`.",
        "Unsigned artifact count: `0`.",
        "`qemu`",
        "`panic`",
        "tools/collect_booted_runtime_v1.py",
        "Local gate: `make test-evidence-integrity-v1`.",
        "Local sub-gate: `make test-synthetic-evidence-ban-v1`.",
        "CI gate: `Evidence integrity v1 gate`.",
        "CI sub-gate: `Synthetic evidence ban v1 gate`.",
    ]:
        assert token in doc


def test_runtime_evidence_schema_v1_doc_declares_required_tokens():
    doc = _read("docs/runtime/runtime_evidence_schema_v1.md")
    for token in [
        "Schema ID: `rugo.runtime_evidence_schema.v1`.",
        "Primary report schema: `rugo.runtime_evidence_report.v1`.",
        "Parent evidence policy ID: `rugo.evidence_integrity_policy.v1`.",
        "Gate provenance policy ID: `rugo.gate_provenance_policy.v1`.",
        "Top-level field: `schema`.",
        "Top-level field: `traces`.",
        "Top-level field: `evidence_items`.",
        "Top-level field: `checks`.",
        "Top-level field: `summary`.",
        "Top-level field: `totals`.",
        "Top-level field: `artifact_refs`.",
        "Top-level field: `release_image_path`.",
        "Top-level field: `digest`.",
        "Trace field: `trace_id`.",
        "Trace field: `execution_lane`.",
        "Trace field: `capture_kind`.",
        "Trace field: `trace_path`.",
        "Trace field: `trace_digest`.",
        "Trace field: `release_image_path`.",
        "Trace field: `boot_id`.",
        "Evidence field: `artifact_id`.",
        "Evidence field: `runtime_source.kind`.",
        "Evidence field: `runtime_source.collector`.",
        "Evidence field: `synthetic`.",
        "Evidence field: `trace_id`.",
        "Evidence field: `trace_digest`.",
        "Evidence field: `artifact_digest`.",
        "Evidence field: `provenance`.",
        "Evidence field: `signature.valid`.",
        "Every evidence item must link to exactly one trace by `trace_id`.",
        "Evidence and trace lane values must match.",
        "Release image binding must match the default shipped image for release lanes.",
        "Deterministic digest algorithm: `sha256`.",
    ]:
        assert token in doc


def test_gate_provenance_policy_v1_doc_declares_required_tokens():
    doc = _read("docs/runtime/gate_provenance_policy_v1.md")
    for token in [
        "Policy ID: `rugo.gate_provenance_policy.v1`.",
        "Parent evidence policy ID: `rugo.evidence_integrity_policy.v1`.",
        "Runtime evidence schema ID: `rugo.runtime_evidence_schema.v1`.",
        "Audit report schema: `rugo.gate_evidence_audit_report.v1`.",
        "Audit tool: `tools/audit_gate_evidence_v1.py`.",
        "`evidence_file_present`",
        "`evidence_schema_valid`",
        "`runtime_capture_ratio`",
        "`qemu_trace_presence_ratio`",
        "`panic_trace_presence_ratio`",
        "`trace_linkage_ratio`",
        "`default_image_binding_ratio`",
        "`boot_instance_binding_ratio`",
        "`provenance_fields_ratio`",
        "`synthetic_evidence_ratio`",
        "`synthetic_only_artifacts`",
        "`detached_trace_count`",
        "`unsigned_artifact_count`",
        "`runtime_lane_coverage`",
        "`release_image_binding`",
        "`release_gate_binding`",
        "Gate pass requires `total_failures = 0`.",
        "Synthetic-only evidence is release-blocking.",
        "Runtime lane coverage expects `qemu` and `panic`.",
        "Local gate: `make test-synthetic-evidence-ban-v1`.",
        "CI gate: `Synthetic evidence ban v1 gate`.",
        "Parent gate integration: `make test-evidence-integrity-v1`.",
    ]:
        assert token in doc


def test_m40_m44_roadmap_anchor_declares_m40_gates():
    roadmap = _read("docs/M40_M44_GENERAL_PURPOSE_PARITY_ROADMAP.md")
    assert "test-evidence-integrity-v1" in roadmap
    assert "test-synthetic-evidence-ban-v1" in roadmap
