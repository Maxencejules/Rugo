"""M22 PR-1: kernel fault-injection matrix baseline checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_fault_campaign_kernel_v1 as campaign  # noqa: E402


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_fault_matrix_tokens_in_reliability_model():
    model = _read("docs/runtime/kernel_reliability_model_v1.md")
    for token in [
        "Fault classes in v1 campaign:",
        "`irq_storm`",
        "`scheduler_starvation`",
        "`allocator_pressure`",
        "`ipc_queue_saturation`",
        "`virtio_retry_timeout`",
        "`timer_drift_burst`",
    ]:
        assert token in model


def test_fault_campaign_kernel_v1_report_schema_and_thresholds(tmp_path: Path):
    out = tmp_path / "kernel-fault-campaign-v1.json"
    rc = campaign.main(
        [
            "--seed",
            "20260306",
            "--iterations",
            "1200",
            "--max-failures",
            "0",
            "--out",
            str(out),
        ]
    )
    assert rc == 0
    assert out.is_file()

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.kernel_fault_campaign_report.v1"
    assert data["fault_matrix_id"] == "rugo.kernel_fault_matrix.v1"
    assert data["iterations"] == 1200
    assert data["total_scenarios"] == len(data["scenarios"])
    assert data["total_scenarios"] >= 6
    assert data["failed_cases"] == 0
    assert data["total_failures"] == 0
    assert data["recovered_cases"] == data["total_injections"]
    assert data["meets_target"] is True
    for scenario in data["scenarios"]:
        assert scenario["failures"] == 0
        assert scenario["recovered"] == scenario["injections"]

