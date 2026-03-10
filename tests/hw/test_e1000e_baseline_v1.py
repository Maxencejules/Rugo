"""M46 PR-2: deterministic e1000e baseline checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_baremetal_io_baseline_v1 as baseline  # noqa: E402


def _coverage_entry(data: dict, device: str) -> dict:
    rows = [entry for entry in data["device_class_coverage"] if entry["device"] == device]
    assert len(rows) == 1
    return rows[0]


def _driver_row(data: dict, driver: str) -> dict:
    rows = [entry for entry in data["driver_lifecycle"] if entry["driver"] == driver]
    assert len(rows) == 1
    return rows[0]


def _tier_profile(data: dict, profile_id: str) -> dict:
    rows = [entry for entry in data["tier2_profiles"] if entry["profile_id"] == profile_id]
    assert len(rows) == 1
    return rows[0]


def _out_path(name: str) -> Path:
    path = ROOT / "out" / "pytest-m46" / name
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        path.unlink()
    return path


def test_e1000e_baseline_v1_schema_and_pass():
    out = _out_path("baremetal-io-v1-e1000e.json")
    rc = baseline.main(["--seed", "20260310", "--out", str(out)])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.baremetal_io_baseline.v1"
    assert data["profile_id"] == "rugo.baremetal_io_profile.v1"
    assert data["gate_pass"] is True
    assert data["summary"]["network"]["pass"] is True
    assert data["wired_nic"]["e1000e"]["status"] == "pass"
    assert _coverage_entry(data, "e1000e")["status"] == "pass"
    row = _driver_row(data, "e1000e")
    assert row["status"] == "pass"
    assert "link_ready" in row["states_observed"]
    assert _tier_profile(data, "intel_q470_e1000e_xhci")["status"] == "pass"


def test_e1000e_baseline_v1_detects_udp_regression():
    out = _out_path("baremetal-io-v1-e1000e-fail.json")
    rc = baseline.main(
        [
            "--inject-failure",
            "e1000e_udp_echo",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    assert data["summary"]["network"]["failures"] >= 1
    assert data["wired_nic"]["e1000e"]["status"] == "fail"
    assert _coverage_entry(data, "e1000e")["status"] == "fail"
    assert _driver_row(data, "e1000e")["status"] == "fail"
    assert _tier_profile(data, "intel_q470_e1000e_xhci")["status"] == "fail"
