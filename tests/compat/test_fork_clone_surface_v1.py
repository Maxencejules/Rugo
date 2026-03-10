"""M41 PR-2: deterministic fork/clone deferred-surface compatibility checks."""

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


def test_fork_clone_deferred_checks_pass_in_campaign():
    report = campaign.run_campaign(seed=20260310)
    assert report["summary"]["deferred"]["pass"] is True
    assert report["deferred"]["fork_enosys_ratio"] == 1.0
    assert report["deferred"]["clone_enosys_ratio"] == 1.0
    assert _check(report, "deferred_fork_enosys")["pass"] is True
    assert _check(report, "deferred_clone_enosys")["pass"] is True


def test_posix_gap_report_tracks_fork_clone_as_deterministic_deferred(tmp_path: Path):
    out = tmp_path / "posix-gap-report-v2-fork-clone.json"
    rc = gap.main(["--seed", "20260310", "--out", str(out)])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    for surface in ["fork", "clone"]:
        row = _deferred_row(data, surface)
        assert row["status"] == "deferred"
        assert row["deterministic"] is True
        assert row["deterministic_error"] == "ENOSYS"


def test_posix_gap_report_detects_clone_nondeterministic_violation(tmp_path: Path):
    out = tmp_path / "posix-gap-report-v2-clone-violation.json"
    rc = gap.main(
        [
            "--inject-deferred-violation",
            "clone",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    assert "clone" in data["deferred_violations"]
    row = _deferred_row(data, "clone")
    assert row["deterministic"] is False
    assert row["deterministic_error"] != "ENOSYS"
