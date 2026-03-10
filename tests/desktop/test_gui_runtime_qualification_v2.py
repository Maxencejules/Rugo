"""M44 PR-2: runtime-qualified GUI app matrix checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_real_gui_app_matrix_v2 as matrix  # noqa: E402


def _strip_timestamp(payload: dict) -> dict:
    stable = dict(payload)
    stable.pop("created_utc", None)
    return stable


def test_real_gui_app_matrix_v2_is_seed_deterministic():
    first = matrix.run_matrix(seed=20260310)
    second = matrix.run_matrix(seed=20260310)
    assert _strip_timestamp(first) == _strip_timestamp(second)


def test_real_gui_app_matrix_v2_schema_and_gate_pass(tmp_path: Path):
    out = tmp_path / "real-gui-matrix-v2.json"
    rc = matrix.main(["--seed", "20260310", "--out", str(out)])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.real_gui_app_matrix_report.v2"
    assert data["profile_id"] == "rugo.desktop_profile.v2"
    assert data["tier_schema"] == "rugo.app_compat_tiers.v2"
    assert data["gate"] == "test-real-ecosystem-desktop-v2"
    assert data["gate_pass"] is True
    assert data["total_failures"] == 0
    assert data["classes"]["productivity"]["meets_threshold"] is True
    assert data["classes"]["media"]["meets_threshold"] is True
    assert data["classes"]["utility"]["meets_threshold"] is True
    assert data["provenance"]["signed_provenance_ratio"] >= 1.0
    assert data["provenance"]["runtime_trace_coverage_ratio"] >= 1.0
    assert data["provenance"]["runtime_source_ratio"] >= 1.0
    assert data["provenance"]["reproducible_ratio"] >= 1.0


def test_real_gui_app_matrix_v2_detects_unsigned_provenance(tmp_path: Path):
    out = tmp_path / "real-gui-matrix-v2-fail.json"
    rc = matrix.main(
        [
            "--inject-unsigned",
            "productivity-runtime-00",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    assert data["total_failures"] >= 1
    assert data["classes"]["productivity"]["meets_threshold"] is False
    assert any(issue["reason"] == "unsigned_provenance" for issue in data["issues"])


def test_real_gui_app_matrix_v2_rejects_unknown_case_id(tmp_path: Path):
    out = tmp_path / "real-gui-matrix-v2-error.json"
    rc = matrix.main(
        [
            "--inject-launch-failure",
            "unknown-case-42",
            "--out",
            str(out),
        ]
    )
    assert rc == 2
    assert not out.exists()
