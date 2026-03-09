"""M27 PR-2: deterministic CLI app compatibility suite checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_app_compat_matrix_v3 as matrix  # noqa: E402


def _strip_timestamp(payload: dict) -> dict:
    stable = dict(payload)
    stable.pop("created_utc", None)
    return stable


def test_cli_suite_v3_is_seed_deterministic():
    first = matrix.run_matrix(seed=20260309)
    second = matrix.run_matrix(seed=20260309)
    assert _strip_timestamp(first) == _strip_timestamp(second)


def test_cli_suite_v3_report_schema_and_thresholds(tmp_path: Path):
    out = tmp_path / "app-compat-matrix-v3.json"
    assert matrix.main(["--seed", "20260309", "--out", str(out)]) == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.app_compat_matrix_report.v3"
    assert data["profile_id"] == "rugo.compat_profile.v3"
    assert data["tier_schema"] == "rugo.app_compat_tiers.v1"

    cli = data["classes"]["cli"]
    assert cli["tier"] == "tier_cli"
    assert cli["eligible"] == 14
    assert cli["passed"] == 13
    assert cli["pass_rate"] >= 0.90
    assert cli["meets_threshold"] is True
    assert data["gate_pass"] is True
