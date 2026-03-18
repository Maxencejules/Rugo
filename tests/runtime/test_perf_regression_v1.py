"""M24 PR-2: deterministic performance baseline and regression tooling checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import collect_booted_runtime_v1 as capture_tool  # noqa: E402
import check_perf_regression_v1 as regression  # noqa: E402
import run_perf_baseline_v1 as baseline  # noqa: E402


def _strip_timestamp(payload: dict) -> dict:
    stable = dict(payload)
    stable.pop("created_utc", None)
    return stable


def test_perf_baseline_v1_is_seed_deterministic():
    capture = capture_tool.runtime_capture.build_fixture_capture()
    first = baseline.run_baseline(capture)
    second = baseline.run_baseline(capture)
    assert _strip_timestamp(first) == _strip_timestamp(second)


def test_perf_regression_v1_report_schema_and_pass(tmp_path: Path):
    capture_out = tmp_path / "booted-runtime-v1.json"
    baseline_out = tmp_path / "perf-baseline-v1.json"
    regression_out = tmp_path / "perf-regression-v1.json"

    assert capture_tool.main(["--fixture", "--out", str(capture_out)]) == 0
    assert baseline.main(["--runtime-capture", str(capture_out), "--out", str(baseline_out)]) == 0
    assert (
        regression.main(
            [
                "--baseline",
                str(baseline_out),
                "--runtime-capture",
                str(capture_out),
                "--out",
                str(regression_out),
            ]
        )
        == 0
    )

    capture_data = json.loads(capture_out.read_text(encoding="utf-8"))
    baseline_data = json.loads(baseline_out.read_text(encoding="utf-8"))
    regression_data = json.loads(regression_out.read_text(encoding="utf-8"))

    assert baseline_data["schema"] == "rugo.perf_baseline.v1"
    assert baseline_data["budget_id"] == "rugo.performance_budget.v1"
    assert baseline_data["workload_count"] >= 6
    assert baseline_data["release_image_path"] == "out/os-go.iso"
    assert baseline_data["runtime_capture_digest"] == capture_data["digest"]

    assert regression_data["schema"] == "rugo.perf_regression_report.v1"
    assert regression_data["baseline_schema"] == "rugo.perf_baseline.v1"
    assert regression_data["release_image_path"] == "out/os-go.iso"
    assert regression_data["total_violations"] == 0
    assert regression_data["requires_action"] is False
    assert regression_data["gate_pass"] is True


def test_perf_regression_v1_detects_threshold_breach(tmp_path: Path):
    capture_out = tmp_path / "booted-runtime-v1.json"
    baseline_out = tmp_path / "perf-baseline-v1.json"
    regression_out = tmp_path / "perf-regression-v1.json"

    assert capture_tool.main(["--fixture", "--out", str(capture_out)]) == 0
    assert baseline.main(["--runtime-capture", str(capture_out), "--out", str(baseline_out)]) == 0
    rc = regression.main(
        [
            "--baseline",
            str(baseline_out),
            "--runtime-capture",
            str(capture_out),
            "--inject-regression",
            "cpu_service_cycle:throughput_ops_per_sec:12.5",
            "--out",
            str(regression_out),
        ]
    )
    assert rc == 1

    data = json.loads(regression_out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.perf_regression_report.v1"
    assert data["total_violations"] >= 1
    assert data["requires_action"] is True
    assert data["gate_pass"] is False

    assert any(
        violation["workload"] == "cpu_service_cycle"
        and violation["metric"] == "throughput_ops_per_sec"
        for violation in data["violations"]
    )
