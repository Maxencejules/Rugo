"""M40 PR-2: runtime evidence to trace linkage contract checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import check_perf_regression_v1 as perf_regression  # noqa: E402
import collect_booted_runtime_v1 as capture_tool  # noqa: E402
import collect_crash_dump_v1 as crash_tool  # noqa: E402
import collect_diagnostic_snapshot_v2 as diag_tool  # noqa: E402
import collect_runtime_evidence_v1 as collector  # noqa: E402
import collect_trace_bundle_v2 as trace_tool  # noqa: E402
import run_perf_baseline_v1 as perf_baseline  # noqa: E402
import symbolize_crash_dump_v1 as symbolizer  # noqa: E402


def _check(data: dict, check_id: str) -> dict:
    rows = [entry for entry in data["checks"] if entry["check_id"] == check_id]
    assert len(rows) == 1
    return rows[0]


def _write_runtime_artifacts(tmp_path: Path) -> dict[str, Path]:
    capture = tmp_path / "booted-runtime-v1.json"
    trace = tmp_path / "trace-bundle-v2.json"
    diag = tmp_path / "diagnostic-snapshot-v2.json"
    crash = tmp_path / "crash-dump-v1.json"
    sym = tmp_path / "crash-dump-symbolized-v1.json"
    perf_base = tmp_path / "perf-baseline-v1.json"
    perf_reg = tmp_path / "perf-regression-v1.json"

    assert capture_tool.main(["--fixture", "--out", str(capture)]) == 0
    assert trace_tool.main(["--runtime-capture", str(capture), "--out", str(trace)]) == 0
    assert diag_tool.main(["--runtime-capture", str(capture), "--trace-bundle", str(trace), "--out", str(diag)]) == 0
    assert crash_tool.main(["--fixture", "--out", str(crash)]) == 0
    assert symbolizer.main(["--dump", str(crash), "--out", str(sym)]) == 0
    assert perf_baseline.main(["--runtime-capture", str(capture), "--out", str(perf_base)]) == 0
    assert perf_regression.main(["--baseline", str(perf_base), "--runtime-capture", str(capture), "--out", str(perf_reg)]) == 0
    return {
        "capture": capture,
        "trace": trace,
        "diag": diag,
        "crash": crash,
        "sym": sym,
        "perf_base": perf_base,
        "perf_reg": perf_reg,
    }


def test_evidence_trace_linkage_v1_passes_for_runtime_artifacts(tmp_path: Path):
    artifacts = _write_runtime_artifacts(tmp_path)
    out = tmp_path / "runtime-evidence-v1.json"
    assert (
        collector.main(
            [
                "--runtime-capture",
                str(artifacts["capture"]),
                "--trace-bundle",
                str(artifacts["trace"]),
                "--diagnostic-snapshot",
                str(artifacts["diag"]),
                "--crash-dump",
                str(artifacts["crash"]),
                "--crash-symbolized",
                str(artifacts["sym"]),
                "--perf-baseline",
                str(artifacts["perf_base"]),
                "--perf-regression",
                str(artifacts["perf_reg"]),
                "--out",
                str(out),
            ]
        )
        == 0
    )

    data = json.loads(out.read_text(encoding="utf-8"))
    trace_index = {trace["trace_id"]: trace for trace in data["traces"]}

    for item in data["evidence_items"]:
        trace_id = item["trace_id"]
        assert trace_id in trace_index
        linked_trace = trace_index[trace_id]
        assert item["trace_digest"] == linked_trace["trace_digest"]
        assert item["execution_lane"] == linked_trace["execution_lane"]

    assert _check(data, "trace_linkage_ratio")["pass"] is True
    assert _check(data, "detached_trace_count")["pass"] is True


def test_evidence_trace_linkage_v1_detects_detached_trace_failure(tmp_path: Path):
    artifacts = _write_runtime_artifacts(tmp_path)
    out = tmp_path / "runtime-evidence-v1-detached.json"
    assert (
        collector.main(
            [
                "--runtime-capture",
                str(artifacts["capture"]),
                "--trace-bundle",
                str(artifacts["trace"]),
                "--diagnostic-snapshot",
                str(artifacts["diag"]),
                "--crash-dump",
                str(artifacts["crash"]),
                "--crash-symbolized",
                str(artifacts["sym"]),
                "--perf-baseline",
                str(artifacts["perf_base"]),
                "--perf-regression",
                str(artifacts["perf_reg"]),
                "--inject-failure",
                "detached_trace_count",
                "--out",
                str(out),
            ]
        )
        == 1
    )

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    assert data["totals"]["detached_trace_count"] >= 1
    assert _check(data, "detached_trace_count")["pass"] is False
