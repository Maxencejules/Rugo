"""M32 PR-2: server profile conformance checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_conformance_suite_v1 as conformance  # noqa: E402


def _strip_timestamp(payload: dict) -> dict:
    stable = dict(payload)
    stable.pop("created_utc", None)
    return stable


def _server_profile(data: dict) -> dict:
    profiles = [p for p in data["profiles"] if p["profile_id"] == "server_v1"]
    assert len(profiles) == 1
    return profiles[0]


def test_server_profile_v1_deterministic_report():
    first = conformance.run_suite(
        seed=20260309,
        selected_profiles=["server_v1"],
        fixture=True,
    )
    second = conformance.run_suite(
        seed=20260309,
        selected_profiles=["server_v1"],
        fixture=True,
    )
    assert _strip_timestamp(first) == _strip_timestamp(second)


def test_server_profile_v1_qualification_pass(tmp_path: Path):
    out = tmp_path / "conformance-server-v1.json"
    rc = conformance.main(
        [
            "--seed",
            "20260309",
            "--fixture",
            "--profile",
            "server_v1",
            "--out",
            str(out),
        ]
    )
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.profile_conformance_report.v1"
    assert data["policy_id"] == "rugo.profile_conformance_policy.v1"
    assert data["checked_profiles"] == ["server_v1"]
    assert data["runtime_capture_digest"]
    assert data["gate_pass"] is True
    assert data["total_failures"] == 0

    profile = _server_profile(data)
    assert profile["profile_label"] == "server"
    assert profile["qualification_pass"] is True
    checks = {check["name"]: check["pass"] for check in profile["checks"]}
    assert checks["requirements_defined"] is True
    assert checks["requirements_all_pass"] is True


def test_server_profile_v1_rejects_requirement_failure(tmp_path: Path):
    out = tmp_path / "conformance-server-v1-fail.json"
    rc = conformance.main(
        [
            "--fixture",
            "--profile",
            "server_v1",
            "--inject-failure",
            "server_v1:service_restart_coverage_pct",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    profile = _server_profile(data)
    failures = [req for req in profile["requirements"] if req["pass"] is False]
    assert len(failures) == 1
    assert failures[0]["requirement_id"] == "service_restart_coverage_pct"
    assert data["gate_pass"] is False
    assert data["total_failures"] >= 1
