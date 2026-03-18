"""M32 PR-2: developer profile conformance checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_conformance_suite_v1 as conformance  # noqa: E402


def _dev_profile(data: dict) -> dict:
    profiles = [p for p in data["profiles"] if p["profile_id"] == "developer_v1"]
    assert len(profiles) == 1
    return profiles[0]


def test_developer_profile_v1_qualification_pass(tmp_path: Path):
    out = tmp_path / "conformance-developer-v1.json"
    rc = conformance.main(
        [
            "--seed",
            "20260309",
            "--fixture",
            "--profile",
            "developer_v1",
            "--out",
            str(out),
        ]
    )
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.profile_conformance_report.v1"
    assert data["checked_profiles"] == ["developer_v1"]
    assert data["runtime_capture_digest"]
    assert data["gate_pass"] is True
    profile = _dev_profile(data)
    assert profile["qualification_pass"] is True
    assert profile["total_failures"] == 0
    requirements = {req["requirement_id"]: req for req in profile["requirements"]}
    assert requirements["interactive_shell_latency_ms_p95"]["observed"] <= 200


def test_developer_profile_v1_rejects_build_success_regression(tmp_path: Path):
    out = tmp_path / "conformance-developer-v1-fail.json"
    rc = conformance.main(
        [
            "--fixture",
            "--profile",
            "developer_v1",
            "--inject-failure",
            "developer_v1:package_build_success_rate_pct",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    profile = _dev_profile(data)
    failures = {req["requirement_id"] for req in profile["requirements"] if not req["pass"]}
    assert "package_build_success_rate_pct" in failures
    assert data["gate_pass"] is False
