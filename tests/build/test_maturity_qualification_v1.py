"""M34 PR-2: maturity qualification bundle checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_maturity_qualification_v1 as maturity  # noqa: E402


def _strip_timestamps(payload: object) -> object:
    if isinstance(payload, dict):
        return {
            key: _strip_timestamps(value)
            for key, value in payload.items()
            if key != "created_utc"
        }
    if isinstance(payload, list):
        return [_strip_timestamps(value) for value in payload]
    return payload


def test_maturity_qualification_v1_deterministic(tmp_path: Path):
    first = tmp_path / "maturity-qualification-a.json"
    second = tmp_path / "maturity-qualification-b.json"

    assert maturity.main(["--seed", "20260309", "--out", str(first)]) == 0
    assert maturity.main(["--seed", "20260309", "--out", str(second)]) == 0

    first_data = json.loads(first.read_text(encoding="utf-8"))
    second_data = json.loads(second.read_text(encoding="utf-8"))
    assert _strip_timestamps(first_data) == _strip_timestamps(second_data)


def test_maturity_qualification_v1_schema_and_pass(tmp_path: Path):
    out = tmp_path / "maturity-qualification-v1.json"
    rc = maturity.main(["--seed", "20260309", "--out", str(out)])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.maturity_qualification_bundle.v1"
    assert data["policy_id"] == "rugo.maturity_qualification_policy.v1"
    assert data["lts_policy_id"] == "rugo.lts_declaration_policy.v1"
    assert data["total_failures"] == 0
    assert data["qualification_pass"] is True
    assert data["lts_declaration"]["schema"] == "rugo.lts_declaration_report.v1"
    assert data["lts_declaration"]["eligible"] is True


def test_maturity_qualification_v1_detects_insufficient_release_history(tmp_path: Path):
    out = tmp_path / "maturity-qualification-v1-fail.json"
    rc = maturity.main(
        [
            "--qualified-release-count",
            "2",
            "--min-qualified-releases",
            "3",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["qualification_pass"] is False
    assert data["total_failures"] >= 1
    assert data["lts_declaration"]["eligible"] is False
    failed = {entry["name"] for entry in data["checks"] if entry["pass"] is False}
    assert "qualified_release_window" in failed
    failed_lts = {
        entry["name"] for entry in data["lts_declaration"]["criteria"] if entry["pass"] is False
    }
    assert "minimum_qualified_releases" in failed_lts
