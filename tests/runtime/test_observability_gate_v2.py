"""M29 aggregate gate: observability v2 wiring and closure checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import collect_booted_runtime_v1 as capture_tool  # noqa: E402
import collect_crash_dump_v1 as crash_collector  # noqa: E402
import collect_diagnostic_snapshot_v2 as diag_tool  # noqa: E402
import collect_trace_bundle_v2 as trace_tool  # noqa: E402
import symbolize_crash_dump_v1 as symbolizer  # noqa: E402


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_observability_gate_v2_wiring_and_artifacts(tmp_path: Path):
    required = [
        "docs/M29_EXECUTION_BACKLOG.md",
        "docs/runtime/observability_contract_v2.md",
        "docs/runtime/crash_dump_contract_v1.md",
        "docs/runtime/postmortem_triage_playbook_v1.md",
        "tools/collect_booted_runtime_v1.py",
        "tools/collect_trace_bundle_v2.py",
        "tools/collect_diagnostic_snapshot_v2.py",
        "tools/collect_crash_dump_v1.py",
        "tools/symbolize_crash_dump_v1.py",
        "tests/runtime/test_booted_runtime_capture_v1.py",
        "tests/runtime/test_observability_docs_v2.py",
        "tests/runtime/test_trace_bundle_v2.py",
        "tests/runtime/test_diag_snapshot_v2.py",
        "tests/runtime/test_observability_gate_v2.py",
        "tests/runtime/test_crash_dump_docs_v1.py",
        "tests/runtime/test_crash_dump_capture_v1.py",
        "tests/runtime/test_crash_dump_symbolization_v1.py",
        "tests/runtime/test_crash_dump_gate_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M29 artifact: {rel}"

    roadmap = _read("docs/M21_M34_MATURITY_PARITY_ROADMAP.md")
    makefile = _read("Makefile")
    ci = _read(".github/workflows/ci.yml")
    backlog = _read("docs/M29_EXECUTION_BACKLOG.md")
    milestones = _read("MILESTONES.md")
    status = _read("docs/STATUS.md")
    readme = _read("README.md")

    assert "test-observability-v2" in roadmap
    assert "test-crash-dump-v1" in roadmap

    assert "test-observability-v2" in makefile
    for entry in [
        "tools/collect_booted_runtime_v1.py --image $(OUT)/os-go.iso --kernel $(OUT)/kernel-go.elf --panic-image $(OUT)/os-panic.iso --out $(OUT)/booted-runtime-v1.json",
        "tools/collect_trace_bundle_v2.py --runtime-capture $(OUT)/booted-runtime-v1.json --window-seconds 300 --out $(OUT)/trace-bundle-v2.json",
        "tools/collect_diagnostic_snapshot_v2.py --runtime-capture $(OUT)/booted-runtime-v1.json --trace-bundle $(OUT)/trace-bundle-v2.json --out $(OUT)/diagnostic-snapshot-v2.json",
        "tests/runtime/test_booted_runtime_capture_v1.py",
        "tests/runtime/test_observability_docs_v2.py",
        "tests/runtime/test_trace_bundle_v2.py",
        "tests/runtime/test_diag_snapshot_v2.py",
        "tests/runtime/test_observability_gate_v2.py",
    ]:
        assert entry in makefile
    assert "pytest-observability-v2.xml" in makefile

    assert "Observability v2 gate" in ci
    assert "make test-observability-v2" in ci
    assert "observability-v2-artifacts" in ci
    assert "out/pytest-observability-v2.xml" in ci
    assert "out/booted-runtime-v1.json" in ci
    assert "out/trace-bundle-v2.json" in ci
    assert "out/diagnostic-snapshot-v2.json" in ci

    assert "Status: done" in backlog
    assert "M29" in milestones
    assert "M29" in status
    assert "docs/architecture/SOURCE_MAP.md" in readme
    assert "docs/archive/README.md" in readme

    capture_out = tmp_path / "booted-runtime-v1.json"
    trace_out = tmp_path / "trace-bundle-v2.json"
    diag_out = tmp_path / "diagnostic-snapshot-v2.json"
    dump_out = tmp_path / "crash-dump-v1.json"
    sym_out = tmp_path / "crash-dump-symbolized-v1.json"

    assert capture_tool.main(["--fixture", "--out", str(capture_out)]) == 0
    assert trace_tool.main(["--runtime-capture", str(capture_out), "--out", str(trace_out)]) == 0
    assert (
        diag_tool.main(
            [
                "--runtime-capture",
                str(capture_out),
                "--trace-bundle",
                str(trace_out),
                "--out",
                str(diag_out),
            ]
        )
        == 0
    )
    assert crash_collector.main(["--fixture", "--out", str(dump_out)]) == 0
    assert symbolizer.main(["--dump", str(dump_out), "--out", str(sym_out)]) == 0

    trace_data = json.loads(trace_out.read_text(encoding="utf-8"))
    diag_data = json.loads(diag_out.read_text(encoding="utf-8"))
    sym_data = json.loads(sym_out.read_text(encoding="utf-8"))

    assert trace_data["schema"] == "rugo.trace_bundle.v2"
    assert trace_data["gate_pass"] is True
    assert diag_data["schema"] == "rugo.diagnostic_snapshot.v2"
    assert diag_data["gate_pass"] is True
    assert sym_data["schema"] == "rugo.crash_dump_symbolized.v1"
    assert sym_data["gate_pass"] is True
