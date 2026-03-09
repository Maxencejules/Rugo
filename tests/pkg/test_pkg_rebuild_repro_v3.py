"""M26 PR-2: deterministic package rebuild verification tooling checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import pkg_rebuild_verify_v3 as rebuild  # noqa: E402


def _strip_timestamp(payload: dict) -> dict:
    stable = dict(payload)
    stable.pop("created_utc", None)
    return stable


def test_pkg_rebuild_v3_is_seed_deterministic():
    first = rebuild.run_rebuild(seed=20260309)
    second = rebuild.run_rebuild(seed=20260309)
    assert _strip_timestamp(first) == _strip_timestamp(second)


def test_pkg_rebuild_v3_report_schema_and_pass(tmp_path: Path):
    out = tmp_path / "pkg-rebuild-v3.json"
    rc = rebuild.main(["--seed", "20260309", "--out", str(out)])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.pkg_rebuild_report.v3"
    assert data["package_format_id"] == "rugo.pkg_format.v3"
    assert data["total_packages"] >= 3
    assert data["total_mismatches"] == 0
    assert data["verified"] is True
    assert data["meets_target"] is True


def test_pkg_rebuild_v3_detects_mismatch(tmp_path: Path):
    out = tmp_path / "pkg-rebuild-v3.json"
    rc = rebuild.main(
        [
            "--seed",
            "20260309",
            "--inject-mismatch",
            "svc-manager",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.pkg_rebuild_report.v3"
    assert data["total_mismatches"] >= 1
    assert data["verified"] is False
    assert any(
        pkg["name"] == "svc-manager" and pkg["match"] is False for pkg in data["packages"]
    )

