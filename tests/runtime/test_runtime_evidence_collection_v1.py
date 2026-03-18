"""M40 PR-2: deterministic runtime evidence collection checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import collect_booted_runtime_v1 as capture_tool  # noqa: E402
import collect_crash_dump_v1 as crash_tool  # noqa: E402
import collect_diagnostic_snapshot_v2 as diag_tool  # noqa: E402
import collect_runtime_evidence_v1 as collector  # noqa: E402
import collect_trace_bundle_v2 as trace_tool  # noqa: E402
import run_perf_baseline_v1 as perf_baseline  # noqa: E402
import check_perf_regression_v1 as perf_regression  # noqa: E402
import symbolize_crash_dump_v1 as symbolizer  # noqa: E402


def _strip_timestamp(payload: dict) -> dict:
    stable = dict(payload)
    stable.pop("created_utc", None)
    return stable


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
    assert (
        diag_tool.main(
            [
                "--runtime-capture",
                str(capture),
                "--trace-bundle",
                str(trace),
                "--out",
                str(diag),
            ]
        )
        == 0
    )
    assert crash_tool.main(["--fixture", "--out", str(crash)]) == 0
    assert symbolizer.main(["--dump", str(crash), "--out", str(sym)]) == 0
    assert perf_baseline.main(["--runtime-capture", str(capture), "--out", str(perf_base)]) == 0
    assert (
        perf_regression.main(
            [
                "--baseline",
                str(perf_base),
                "--runtime-capture",
                str(capture),
                "--out",
                str(perf_reg),
            ]
        )
        == 0
    )
    return {
        "capture": capture,
        "trace": trace,
        "diag": diag,
        "crash": crash,
        "sym": sym,
        "perf_base": perf_base,
        "perf_reg": perf_reg,
    }


def test_runtime_evidence_report_is_seed_deterministic():
    capture = capture_tool.runtime_capture.build_fixture_capture()
    crash = crash_tool.build_dump(
        panic_code=0xDEAD,
        release_image_digest=str(capture["image_digest"]),
        panic_trace_digest="panic-trace",
    )
    baseline = perf_baseline.run_baseline(capture)
    trace = trace_tool.collect_trace_bundle(capture, window_seconds=300)
    diag = diag_tool.collect_snapshot(capture, trace)
    sym = symbolizer.symbolize(crash)
    first = collector.collect_runtime_evidence(
        runtime_capture_payload=capture,
        runtime_capture_path="out/booted-runtime-v1.json",
        trace_bundle_payload=trace,
        trace_bundle_path="out/trace-bundle-v2.json",
        diagnostic_snapshot_payload=diag,
        diagnostic_snapshot_path="out/diagnostic-snapshot-v2.json",
        crash_dump_payload=crash,
        crash_dump_path="out/crash-dump-v1.json",
        crash_symbolized_payload=sym,
        crash_symbolized_path="out/crash-dump-symbolized-v1.json",
        perf_baseline_payload=baseline,
        perf_baseline_path="out/perf-baseline-v1.json",
        perf_regression_payload={"schema": "rugo.perf_regression_report.v1", "digest": "same", "gate_pass": True},
        perf_regression_path="out/perf-regression-v1.json",
    )
    second = collector.collect_runtime_evidence(
        runtime_capture_payload=capture,
        runtime_capture_path="out/booted-runtime-v1.json",
        trace_bundle_payload=trace,
        trace_bundle_path="out/trace-bundle-v2.json",
        diagnostic_snapshot_payload=diag,
        diagnostic_snapshot_path="out/diagnostic-snapshot-v2.json",
        crash_dump_payload=crash,
        crash_dump_path="out/crash-dump-v1.json",
        crash_symbolized_payload=sym,
        crash_symbolized_path="out/crash-dump-symbolized-v1.json",
        perf_baseline_payload=baseline,
        perf_baseline_path="out/perf-baseline-v1.json",
        perf_regression_payload={"schema": "rugo.perf_regression_report.v1", "digest": "same", "gate_pass": True},
        perf_regression_path="out/perf-regression-v1.json",
    )
    assert _strip_timestamp(first) == _strip_timestamp(second)


def test_runtime_evidence_collection_v1_schema_and_pass(tmp_path: Path):
    artifacts = _write_runtime_artifacts(tmp_path)
    out = tmp_path / "runtime-evidence-v1.json"
    rc = collector.main(
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
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.runtime_evidence_report.v1"
    assert data["evidence_integrity_policy_id"] == "rugo.evidence_integrity_policy.v1"
    assert data["runtime_evidence_schema_id"] == "rugo.runtime_evidence_schema.v1"
    assert data["gate_provenance_policy_id"] == "rugo.gate_provenance_policy.v1"
    assert data["gate"] == "test-evidence-integrity-v1"
    assert data["release_image_path"] == "out/os-go.iso"
    assert data["gate_pass"] is True
    assert data["total_failures"] == 0
    assert data["summary"]["execution"]["pass"] is True
    assert data["summary"]["provenance"]["pass"] is True
    assert data["summary"]["synthetic"]["pass"] is True
    assert data["totals"]["evidence_items"] >= 7
    assert data["totals"]["synthetic_items"] == 0

    lanes = {trace["execution_lane"] for trace in data["traces"]}
    assert {"qemu", "panic"}.issubset(lanes)

    for item in data["evidence_items"]:
        assert item["synthetic"] is False
        assert item["trace_id"]
        assert item["trace_digest"]
        assert item["signature"]["valid"] is True

    assert _check(data, "trace_linkage_ratio")["pass"] is True
    assert _check(data, "synthetic_only_artifacts")["pass"] is True
    assert _check(data, "default_image_binding_ratio")["pass"] is True


def test_runtime_evidence_collection_detects_synthetic_only_failure(tmp_path: Path):
    artifacts = _write_runtime_artifacts(tmp_path)
    out = tmp_path / "runtime-evidence-v1-fail.json"
    rc = collector.main(
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
            "synthetic_only_artifacts",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    assert data["summary"]["synthetic"]["failures"] >= 1
    assert _check(data, "synthetic_only_artifacts")["pass"] is False


def test_runtime_evidence_collection_rejects_unknown_check_id(tmp_path: Path):
    artifacts = _write_runtime_artifacts(tmp_path)
    out = tmp_path / "runtime-evidence-v1-error.json"
    rc = collector.main(
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
            "runtime_nonexistent_check",
            "--out",
            str(out),
        ]
    )
    assert rc == 2
    assert not out.exists()
