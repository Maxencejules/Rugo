"""M41 PR-2: deterministic readiness/epoll deferred-surface compatibility checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_compat_surface_campaign_v2 as campaign  # noqa: E402
import run_posix_gap_report_v2 as gap  # noqa: E402


def _check(data: dict, check_id: str) -> dict:
    rows = [entry for entry in data["checks"] if entry["check_id"] == check_id]
    assert len(rows) == 1
    return rows[0]


def _deferred_row(data: dict, surface: str) -> dict:
    rows = [entry for entry in data["deferred_surfaces"] if entry["surface"] == surface]
    assert len(rows) == 1
    return rows[0]


def test_epoll_surface_campaign_v2_schema_and_pass(tmp_path: Path):
    out = tmp_path / "compat-surface-v2-epoll.json"
    rc = campaign.main(["--seed", "20260310", "--out", str(out)])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.compat_surface_campaign_report.v2"
    assert data["readiness_model_id"] == "rugo.readiness_io_model.v1"
    assert data["summary"]["readiness"]["pass"] is True
    assert data["readiness"]["checks_pass"] is True
    assert data["readiness"]["poll_wakeup_ms"] <= 11
    assert data["readiness"]["ppoll_wakeup_ms"] <= 10
    assert _check(data, "readiness_poll_wakeup")["pass"] is True
    assert _check(data, "deferred_epoll_enosys")["pass"] is True


def test_epoll_surface_campaign_v2_detects_readiness_regression(tmp_path: Path):
    out = tmp_path / "compat-surface-v2-epoll-fail.json"
    rc = campaign.main(
        [
            "--inject-failure",
            "readiness_poll_wakeup",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    assert data["summary"]["readiness"]["failures"] >= 1
    assert _check(data, "readiness_poll_wakeup")["pass"] is False


def test_posix_gap_report_tracks_epoll_deferred_enosys(tmp_path: Path):
    out = tmp_path / "posix-gap-report-v2-epoll.json"
    rc = gap.main(["--seed", "20260310", "--out", str(out)])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    row = _deferred_row(data, "epoll")
    assert row["status"] == "deferred"
    assert row["deterministic"] is True
    assert row["deterministic_error"] == "ENOSYS"
