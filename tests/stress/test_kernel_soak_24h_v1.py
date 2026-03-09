"""M22 PR-1: kernel soak reliability baseline and model tokens."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_kernel_soak_v1 as soak  # noqa: E402


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_m22_pr1_artifacts_exist():
    required = [
        "docs/M22_EXECUTION_BACKLOG.md",
        "docs/runtime/kernel_reliability_model_v1.md",
        "tests/stress/test_kernel_soak_24h_v1.py",
        "tests/stress/test_fault_injection_matrix_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M22 PR-1 artifact: {rel}"


def test_kernel_reliability_model_v1_required_tokens():
    model = _read("docs/runtime/kernel_reliability_model_v1.md")
    for token in [
        "Reliability Model ID: `rugo.kernel_reliability_model.v1`",
        "Fault Matrix ID: `rugo.kernel_fault_matrix.v1`",
        "Soak duration target: 24 hours.",
        "Minimum simulated campaign length in CI: 1440 iterations (1 iteration = 1 minute).",
        "Maximum allowed kernel panics: 0.",
        "Maximum allowed watchdog resets: 0.",
        "Maximum allowed deadlock events: 0.",
        "Maximum allowed data-corruption events: 0.",
        "tools/run_kernel_soak_v1.py",
        "tools/run_fault_campaign_kernel_v1.py",
        "make test-kernel-reliability-v1",
    ]:
        assert token in model


def test_kernel_soak_24h_v1_report_schema_and_thresholds(tmp_path: Path):
    out = tmp_path / "kernel-soak-v1.json"
    rc = soak.main(
        [
            "--seed",
            "20260306",
            "--iterations",
            "1440",
            "--duration-hours-target",
            "24",
            "--max-failures",
            "0",
            "--out",
            str(out),
        ]
    )
    assert rc == 0
    assert out.is_file()

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.kernel_soak_report.v1"
    assert data["reliability_model_id"] == "rugo.kernel_reliability_model.v1"
    assert data["simulated_minutes"] == 1440
    assert data["duration_hours_target"] == 24
    assert sum(data["workload_mix"].values()) == 1440
    assert data["total_failures"] == 0
    assert data["meets_duration_target"] is True
    assert data["meets_target"] is True
    assert data["gate_pass"] is True

