"""M26 PR-2: repository policy v3 tooling checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import repo_policy_check_v3 as policy  # noqa: E402


def test_repo_policy_v3_report_schema_and_pass(tmp_path: Path):
    out = tmp_path / "repo-policy-v3.json"
    rc = policy.main(["--out", str(out), "--max-failures", "0"])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.repo_policy_report.v3"
    assert data["policy_id"] == "rugo.repository_policy.v3"
    assert data["total_checks"] >= 5
    assert data["total_failures"] == 0
    assert data["meets_target"] is True


def test_repo_policy_v3_detects_injected_failure(tmp_path: Path):
    out = tmp_path / "repo-policy-v3.json"
    rc = policy.main(
        [
            "--inject-failure",
            "metadata_expiry_window",
            "--max-failures",
            "0",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.repo_policy_report.v3"
    assert data["total_failures"] >= 1
    assert data["meets_target"] is False
    assert any(
        check["name"] == "metadata_expiry_window" and check["passed"] is False
        for check in data["checks"]
    )

