"""M24 aggregate gate: performance baseline/regression v1 wiring and closure."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import collect_booted_runtime_v1 as capture_tool  # noqa: E402
import check_perf_regression_v1 as regression  # noqa: E402
import run_perf_baseline_v1 as baseline  # noqa: E402


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_perf_gate_v1_wiring_and_artifacts(tmp_path: Path):
    required = [
        "docs/M24_EXECUTION_BACKLOG.md",
        "docs/runtime/performance_budget_v1.md",
        "docs/runtime/benchmark_policy_v1.md",
        "tools/collect_booted_runtime_v1.py",
        "tools/run_perf_baseline_v1.py",
        "tools/check_perf_regression_v1.py",
        "tests/runtime/test_booted_runtime_capture_v1.py",
        "tests/runtime/test_perf_budget_docs_v1.py",
        "tests/runtime/test_perf_regression_v1.py",
        "tests/runtime/test_perf_gate_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M24 artifact: {rel}"

    roadmap = _read("docs/M21_M34_MATURITY_PARITY_ROADMAP.md")
    makefile = _read("Makefile")
    ci = _read(".github/workflows/ci.yml")
    backlog = _read("docs/M24_EXECUTION_BACKLOG.md")
    milestones = _read("MILESTONES.md")
    status = _read("docs/STATUS.md")
    readme = _read("README.md")

    assert "test-perf-regression-v1" in roadmap
    assert "test-perf-regression-v1" in makefile
    for entry in [
        "tools/collect_booted_runtime_v1.py --image $(OUT)/os-go.iso --kernel $(OUT)/kernel-go.elf --out $(OUT)/booted-runtime-v1.json",
        "tools/run_perf_baseline_v1.py --runtime-capture $(OUT)/booted-runtime-v1.json --out $(OUT)/perf-baseline-v1.json",
        "tools/check_perf_regression_v1.py --baseline $(OUT)/perf-baseline-v1.json --runtime-capture $(OUT)/booted-runtime-v1.json --out $(OUT)/perf-regression-v1.json",
        "tests/runtime/test_booted_runtime_capture_v1.py",
        "tests/runtime/test_perf_budget_docs_v1.py",
        "tests/runtime/test_perf_regression_v1.py",
        "tests/runtime/test_perf_gate_v1.py",
    ]:
        assert entry in makefile
    assert "pytest-perf-regression-v1.xml" in makefile

    assert "Performance regression v1 gate" in ci
    assert "make test-perf-regression-v1" in ci
    assert "perf-regression-v1-artifacts" in ci
    assert "out/pytest-perf-regression-v1.xml" in ci
    assert "out/booted-runtime-v1.json" in ci
    assert "out/perf-baseline-v1.json" in ci
    assert "out/perf-regression-v1.json" in ci

    assert "Status: done" in backlog
    assert "M24" in milestones
    assert "M24" in status
    assert "docs/architecture/SOURCE_MAP.md" in readme
    assert "docs/archive/README.md" in readme

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
    report = json.loads(regression_out.read_text(encoding="utf-8"))
    assert report["schema"] == "rugo.perf_regression_report.v1"
    assert report["gate_pass"] is True
