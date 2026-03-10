"""M41 PR-2: deterministic process model v4 campaign checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_compat_surface_campaign_v2 as campaign  # noqa: E402


def _strip_timestamp(payload: dict) -> dict:
    stable = dict(payload)
    stable.pop("created_utc", None)
    return stable


def _check(data: dict, check_id: str) -> dict:
    rows = [entry for entry in data["checks"] if entry["check_id"] == check_id]
    assert len(rows) == 1
    return rows[0]


def test_process_model_v4_campaign_is_seed_deterministic():
    first = campaign.run_campaign(seed=20260310)
    second = campaign.run_campaign(seed=20260310)
    assert _strip_timestamp(first) == _strip_timestamp(second)


def test_process_model_v4_campaign_schema_and_pass(tmp_path: Path):
    out = tmp_path / "compat-surface-v2-process.json"
    rc = campaign.main(["--seed", "20260310", "--out", str(out)])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.compat_surface_campaign_report.v2"
    assert data["profile_id"] == "rugo.compat_profile.v5"
    assert data["process_model_id"] == "rugo.process_model.v4"
    assert data["gate_pass"] is True
    assert data["summary"]["process"]["pass"] is True
    assert data["process"]["checks_pass"] is True
    assert data["process"]["spawn_exec_ms"] <= 130
    assert data["process"]["wait_reap_ms"] <= 22
    assert data["process"]["pid_reuse_violations"] == 0
    assert _check(data, "process_wait_reap_once")["pass"] is True
    assert _check(data, "process_sigkill_terminal")["pass"] is True
    assert _check(data, "process_pid_reuse_guard")["pass"] is True


def test_process_model_v4_detects_wait_reap_regression(tmp_path: Path):
    out = tmp_path / "compat-surface-v2-process-fail.json"
    rc = campaign.main(
        [
            "--inject-failure",
            "process_wait_reap_once",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    assert data["summary"]["process"]["failures"] >= 1
    assert _check(data, "process_wait_reap_once")["pass"] is False


def test_process_model_v4_rejects_unknown_check_id(tmp_path: Path):
    out = tmp_path / "compat-surface-v2-process-error.json"
    rc = campaign.main(
        [
            "--inject-failure",
            "process_nonexistent_check",
            "--out",
            str(out),
        ]
    )
    assert rc == 2
    assert not out.exists()
