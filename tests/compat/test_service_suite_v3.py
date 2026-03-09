"""M27 PR-2: deterministic service app compatibility suite checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_app_compat_matrix_v3 as matrix  # noqa: E402


def test_service_suite_v3_report_thresholds(tmp_path: Path):
    out = tmp_path / "app-compat-service-v3.json"
    assert matrix.main(["--seed", "20260309", "--out", str(out)]) == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    service = data["classes"]["service"]
    assert service["tier"] == "tier_service"
    assert service["eligible"] == 8
    assert service["passed"] == 7
    assert service["pass_rate"] >= 0.80
    assert service["meets_threshold"] is True


def test_service_suite_v3_detects_unsigned_and_regression(tmp_path: Path):
    out = tmp_path / "app-compat-service-v3-fail.json"
    assert (
        matrix.main(
            [
                "--inject-unsigned",
                "service-00",
                "--inject-failure",
                "service-01",
                "--inject-failure",
                "service-02",
                "--out",
                str(out),
            ]
        )
        == 1
    )

    data = json.loads(out.read_text(encoding="utf-8"))
    reasons = {item["reason"] for item in data["issues"]}
    assert "unsigned_artifact" in reasons
    assert data["classes"]["service"]["meets_threshold"] is False
    assert data["gate_pass"] is False
