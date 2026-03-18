"""M40 aggregate gate: evidence integrity v1 wiring and closure checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import audit_gate_evidence_v1 as audit  # noqa: E402
import check_perf_regression_v1 as perf_regression  # noqa: E402
import collect_booted_runtime_v1 as capture_tool  # noqa: E402
import collect_crash_dump_v1 as crash_tool  # noqa: E402
import collect_diagnostic_snapshot_v2 as diag_tool  # noqa: E402
import collect_runtime_evidence_v1 as collector  # noqa: E402
import collect_trace_bundle_v2 as trace_tool  # noqa: E402
import run_perf_baseline_v1 as perf_baseline  # noqa: E402
import symbolize_crash_dump_v1 as symbolizer  # noqa: E402


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_evidence_integrity_gate_v1_wiring_and_artifacts(tmp_path: Path):
    required = [
        "docs/M40_EXECUTION_BACKLOG.md",
        "docs/runtime/evidence_integrity_policy_v1.md",
        "docs/runtime/runtime_evidence_schema_v1.md",
        "docs/runtime/gate_provenance_policy_v1.md",
        "tools/collect_booted_runtime_v1.py",
        "tools/collect_runtime_evidence_v1.py",
        "tools/audit_gate_evidence_v1.py",
        "tests/runtime/test_booted_runtime_capture_v1.py",
        "tests/runtime/test_evidence_integrity_docs_v1.py",
        "tests/runtime/test_runtime_evidence_collection_v1.py",
        "tests/runtime/test_gate_evidence_audit_v1.py",
        "tests/runtime/test_evidence_trace_linkage_v1.py",
        "tests/runtime/test_synthetic_evidence_ban_v1.py",
        "tests/runtime/test_evidence_integrity_gate_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M40 artifact: {rel}"

    roadmap = _read("docs/M40_M44_GENERAL_PURPOSE_PARITY_ROADMAP.md")
    makefile = _read("Makefile")
    ci = _read(".github/workflows/ci.yml")
    backlog = _read("docs/M40_EXECUTION_BACKLOG.md")
    milestones = _read("MILESTONES.md")
    status = _read("docs/STATUS.md")
    readme = _read("README.md")

    assert "test-evidence-integrity-v1" in roadmap
    assert "test-synthetic-evidence-ban-v1" in roadmap

    assert "test-evidence-integrity-v1" in makefile
    for entry in [
        "tools/collect_booted_runtime_v1.py --image $(OUT)/os-go.iso --kernel $(OUT)/kernel-go.elf --panic-image $(OUT)/os-panic.iso --out $(OUT)/booted-runtime-v1.json",
        "tools/run_perf_baseline_v1.py --runtime-capture $(OUT)/booted-runtime-v1.json --out $(OUT)/perf-baseline-v1.json",
        "tools/check_perf_regression_v1.py --baseline $(OUT)/perf-baseline-v1.json --runtime-capture $(OUT)/booted-runtime-v1.json --out $(OUT)/perf-regression-v1.json",
        "tools/collect_trace_bundle_v2.py --runtime-capture $(OUT)/booted-runtime-v1.json --window-seconds 300 --out $(OUT)/trace-bundle-v2.json",
        "tools/collect_diagnostic_snapshot_v2.py --runtime-capture $(OUT)/booted-runtime-v1.json --trace-bundle $(OUT)/trace-bundle-v2.json --out $(OUT)/diagnostic-snapshot-v2.json",
        "tools/collect_crash_dump_v1.py --release-image $(OUT)/os-go.iso --kernel $(OUT)/kernel-go.elf --panic-image $(OUT)/os-panic.iso --out $(OUT)/crash-dump-v1.json",
        "tools/collect_runtime_evidence_v1.py --runtime-capture $(OUT)/booted-runtime-v1.json --trace-bundle $(OUT)/trace-bundle-v2.json --diagnostic-snapshot $(OUT)/diagnostic-snapshot-v2.json --crash-dump $(OUT)/crash-dump-v1.json --crash-symbolized $(OUT)/crash-dump-symbolized-v1.json --perf-baseline $(OUT)/perf-baseline-v1.json --perf-regression $(OUT)/perf-regression-v1.json --out $(OUT)/runtime-evidence-v1.json",
        "tools/audit_gate_evidence_v1.py --evidence $(OUT)/runtime-evidence-v1.json --out $(OUT)/gate-evidence-audit-v1.json",
        "tests/runtime/test_booted_runtime_capture_v1.py",
        "tests/runtime/test_evidence_integrity_docs_v1.py",
        "tests/runtime/test_runtime_evidence_collection_v1.py",
        "tests/runtime/test_gate_evidence_audit_v1.py",
        "tests/runtime/test_evidence_trace_linkage_v1.py",
        "tests/runtime/test_synthetic_evidence_ban_v1.py",
        "tests/runtime/test_evidence_integrity_gate_v1.py",
    ]:
        assert entry in makefile
    assert "pytest-evidence-integrity-v1.xml" in makefile
    assert "pytest-synthetic-evidence-ban-v1.xml" in makefile

    assert "Evidence integrity v1 gate" in ci
    assert "make test-evidence-integrity-v1" in ci
    assert "evidence-integrity-v1-artifacts" in ci
    assert "out/pytest-evidence-integrity-v1.xml" in ci
    assert "out/booted-runtime-v1.json" in ci
    assert "out/perf-baseline-v1.json" in ci
    assert "out/perf-regression-v1.json" in ci
    assert "out/trace-bundle-v2.json" in ci
    assert "out/diagnostic-snapshot-v2.json" in ci
    assert "out/crash-dump-v1.json" in ci
    assert "out/crash-dump-symbolized-v1.json" in ci
    assert "out/runtime-evidence-v1.json" in ci
    assert "out/gate-evidence-audit-v1.json" in ci

    assert "Status: done" in backlog
    assert "| M40 | Runtime-Backed Evidence Integrity v1 | n/a | done |" in milestones
    assert "| **M40** Runtime-Backed Evidence Integrity v1 | n/a | done |" in status
    assert "docs/architecture/SOURCE_MAP.md" in readme
    assert "docs/archive/README.md" in readme

    capture_out = tmp_path / "booted-runtime-v1.json"
    trace_out = tmp_path / "trace-bundle-v2.json"
    diag_out = tmp_path / "diagnostic-snapshot-v2.json"
    crash_out = tmp_path / "crash-dump-v1.json"
    sym_out = tmp_path / "crash-dump-symbolized-v1.json"
    perf_base_out = tmp_path / "perf-baseline-v1.json"
    perf_reg_out = tmp_path / "perf-regression-v1.json"
    evidence_out = tmp_path / "runtime-evidence-v1.json"
    audit_out = tmp_path / "gate-evidence-audit-v1.json"

    assert capture_tool.main(["--fixture", "--out", str(capture_out)]) == 0
    assert trace_tool.main(["--runtime-capture", str(capture_out), "--out", str(trace_out)]) == 0
    assert diag_tool.main(["--runtime-capture", str(capture_out), "--trace-bundle", str(trace_out), "--out", str(diag_out)]) == 0
    assert crash_tool.main(["--fixture", "--out", str(crash_out)]) == 0
    assert symbolizer.main(["--dump", str(crash_out), "--out", str(sym_out)]) == 0
    assert perf_baseline.main(["--runtime-capture", str(capture_out), "--out", str(perf_base_out)]) == 0
    assert perf_regression.main(["--baseline", str(perf_base_out), "--runtime-capture", str(capture_out), "--out", str(perf_reg_out)]) == 0
    assert (
        collector.main(
            [
                "--runtime-capture",
                str(capture_out),
                "--trace-bundle",
                str(trace_out),
                "--diagnostic-snapshot",
                str(diag_out),
                "--crash-dump",
                str(crash_out),
                "--crash-symbolized",
                str(sym_out),
                "--perf-baseline",
                str(perf_base_out),
                "--perf-regression",
                str(perf_reg_out),
                "--out",
                str(evidence_out),
            ]
        )
        == 0
    )
    assert audit.main(["--evidence", str(evidence_out), "--out", str(audit_out)]) == 0

    evidence_data = json.loads(evidence_out.read_text(encoding="utf-8"))
    audit_data = json.loads(audit_out.read_text(encoding="utf-8"))

    assert evidence_data["schema"] == "rugo.runtime_evidence_report.v1"
    assert evidence_data["evidence_integrity_policy_id"] == "rugo.evidence_integrity_policy.v1"
    assert evidence_data["release_image_path"] == "out/os-go.iso"
    assert evidence_data["gate_pass"] is True
    assert evidence_data["total_failures"] == 0

    assert audit_data["schema"] == "rugo.gate_evidence_audit_report.v1"
    assert audit_data["audit_policy_id"] == "rugo.gate_provenance_policy.v1"
    assert audit_data["gate_pass"] is True
    assert audit_data["total_failures"] == 0
