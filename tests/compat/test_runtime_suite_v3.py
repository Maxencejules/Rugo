"""M27 PR-2: deterministic runtime app compatibility suite checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_app_compat_matrix_v3 as matrix  # noqa: E402


def test_runtime_suite_v3_report_thresholds(tmp_path: Path):
    out = tmp_path / "app-compat-runtime-v3.json"
    assert matrix.main(["--seed", "20260309", "--out", str(out)]) == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    runtime = data["classes"]["runtime"]
    assert runtime["tier"] == "tier_runtime"
    assert runtime["eligible"] == 10
    assert runtime["passed"] == 8
    assert runtime["pass_rate"] >= 0.80
    assert runtime["meets_threshold"] is True


def test_runtime_suite_v3_rejects_profile_mismatch(tmp_path: Path):
    out = tmp_path / "app-compat-runtime-v3-mismatch.json"
    assert (
        matrix.main(
            [
                "--inject-profile-mismatch",
                "runtime-00",
                "--out",
                str(out),
            ]
        )
        == 1
    )

    data = json.loads(out.read_text(encoding="utf-8"))
    reasons = {item["reason"] for item in data["issues"]}
    assert "abi_profile_mismatch" in reasons
    assert data["gate_pass"] is False
