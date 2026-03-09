"""M22 PR-2: deterministic reliability artifact schema checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_fault_campaign_kernel_v1 as fault_campaign  # noqa: E402
import run_kernel_soak_v1 as soak  # noqa: E402


def _strip_timestamp(payload: dict) -> dict:
    stable = dict(payload)
    stable.pop("created_utc", None)
    return stable


def test_reliability_tools_are_seed_deterministic():
    soak_first = soak.run_soak(seed=20260306, iterations=360)
    soak_second = soak.run_soak(seed=20260306, iterations=360)
    assert _strip_timestamp(soak_first) == _strip_timestamp(soak_second)

    fault_first = fault_campaign.run_campaign(seed=20260306, iterations=360)
    fault_second = fault_campaign.run_campaign(seed=20260306, iterations=360)
    assert _strip_timestamp(fault_first) == _strip_timestamp(fault_second)


def test_reliability_artifact_schema_v1(tmp_path: Path):
    soak_out = tmp_path / "kernel-soak-v1.json"
    fault_out = tmp_path / "kernel-fault-campaign-v1.json"

    assert soak.main(["--seed", "20260306", "--out", str(soak_out)]) == 0
    assert (
        fault_campaign.main(["--seed", "20260306", "--out", str(fault_out)]) == 0
    )

    soak_data = json.loads(soak_out.read_text(encoding="utf-8"))
    fault_data = json.loads(fault_out.read_text(encoding="utf-8"))

    for key in [
        "schema",
        "created_utc",
        "seed",
        "total_failures",
    ]:
        assert key in soak_data
        assert key in fault_data

    assert soak_data["schema"] == "rugo.kernel_soak_report.v1"
    assert soak_data["gate_pass"] is True
    assert soak_data["total_failures"] <= soak_data["max_failures"]

    assert fault_data["schema"] == "rugo.kernel_fault_campaign_report.v1"
    assert fault_data["meets_target"] is True
    assert fault_data["total_failures"] <= fault_data["max_failures"]

