"""M34 PR-2: LTS declaration policy checks via maturity qualification bundle."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_maturity_qualification_v1 as maturity  # noqa: E402


def test_lts_policy_v1_artifacts_exist():
    required = [
        "docs/M34_EXECUTION_BACKLOG.md",
        "docs/build/lts_declaration_policy_v1.md",
        "tools/run_maturity_qualification_v1.py",
        "tests/build/test_lts_policy_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M34 PR-2 artifact: {rel}"


def test_lts_policy_v1_default_bundle_declares_eligible(tmp_path: Path):
    out = tmp_path / "maturity-qualification-v1.json"
    assert maturity.main(["--seed", "20260309", "--fixture", "--out", str(out)]) == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    lts = data["lts_declaration"]
    assert lts["schema"] == "rugo.lts_declaration_report.v1"
    assert lts["policy_id"] == "rugo.lts_declaration_policy.v1"
    assert lts["eligible"] is True
    assert lts["qualified_release_count"] >= lts["min_qualified_releases"]
    assert lts["lts_support_days"] >= lts["min_support_days"]
    assert lts["advisory_sla_breach_count"] == 0
    assert lts["supply_chain_drift_count"] == 0
    assert lts["supported_surface"]["supported_profiles"] == ["server_v1", "appliance_v1"]
    assert lts["supported_surface"]["non_lts_profiles"] == ["developer_v1"]


def test_lts_policy_v1_blocks_when_release_history_is_short(tmp_path: Path):
    out = tmp_path / "maturity-qualification-v1-short-history.json"
    assert (
        maturity.main(
            [
                "--qualified-release-count",
                "2",
                "--min-qualified-releases",
                "3",
                "--fixture",
                "--out",
                str(out),
            ]
        )
        == 1
    )

    data = json.loads(out.read_text(encoding="utf-8"))
    lts = data["lts_declaration"]
    assert lts["eligible"] is False
    failed = {entry["name"] for entry in lts["criteria"] if entry["pass"] is False}
    assert "minimum_qualified_releases" in failed
