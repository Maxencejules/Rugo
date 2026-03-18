"""M32 PR-2: appliance profile runtime qualification checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_conformance_suite_v1 as conformance  # noqa: E402


def _appliance_profile(data: dict) -> dict:
    profiles = [p for p in data["profiles"] if p["profile_id"] == "appliance_v1"]
    assert len(profiles) == 1
    return profiles[0]


def test_appliance_profile_v1_qualification_pass(tmp_path: Path):
    out = tmp_path / "conformance-appliance-v1.json"
    rc = conformance.main(
        [
            "--fixture",
            "--profile",
            "appliance_v1",
            "--out",
            str(out),
        ]
    )
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    profile = _appliance_profile(data)
    assert data["runtime_capture_digest"]
    assert profile["qualification_pass"] is True
    requirements = {req["requirement_id"]: req for req in profile["requirements"]}
    assert requirements["immutable_rootfs_enforced"]["observed"] == 1
    assert requirements["read_only_runtime_pct"]["observed"] >= 99
    assert requirements["boot_to_service_seconds_p95"]["observed"] <= 45
    assert requirements["remote_mgmt_surface_minimized"]["observed"] == 1


def test_appliance_profile_v1_rejects_rootfs_regression(tmp_path: Path):
    out = tmp_path / "conformance-appliance-v1-fail.json"
    rc = conformance.main(
        [
            "--fixture",
            "--profile",
            "appliance_v1",
            "--inject-failure",
            "appliance_v1:immutable_rootfs_enforced",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    profile = _appliance_profile(data)
    failed = {req["requirement_id"] for req in profile["requirements"] if req["pass"] is False}
    assert "immutable_rootfs_enforced" in failed
