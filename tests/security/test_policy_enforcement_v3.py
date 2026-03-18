"""M28 PR-2: cross-policy enforcement checks for hardening and response tools."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_security_attack_suite_v3 as attack_suite  # noqa: E402
import run_security_fuzz_v2 as fuzz_tool  # noqa: E402
import security_advisory_lint_v1 as advisory_lint  # noqa: E402
import security_embargo_drill_v1 as embargo_tool  # noqa: E402


def test_policy_identifiers_align_across_reports(tmp_path: Path):
    attack_out = tmp_path / "security-attack-suite-v3.json"
    fuzz_out = tmp_path / "security-fuzz-v2.json"
    advisory_out = tmp_path / "security-advisory-lint-v1.json"
    embargo_out = tmp_path / "security-embargo-drill-v1.json"

    assert attack_suite.main(["--seed", "20260309", "--fixture", "--out", str(attack_out)]) == 0
    assert (
        fuzz_tool.main(
            [
                "--seed",
                "20260309",
                "--iterations",
                "800",
                "--cases",
                "5",
                "--out",
                str(fuzz_out),
            ]
        )
        == 0
    )
    assert advisory_lint.main(["--out", str(advisory_out)]) == 0
    assert embargo_tool.main(["--out", str(embargo_out)]) == 0

    attack_data = json.loads(attack_out.read_text(encoding="utf-8"))
    fuzz_data = json.loads(fuzz_out.read_text(encoding="utf-8"))
    advisory_data = json.loads(advisory_out.read_text(encoding="utf-8"))
    embargo_data = json.loads(embargo_out.read_text(encoding="utf-8"))

    assert attack_data["profile_id"] == "rugo.security_hardening_profile.v3"
    assert attack_data["threat_model_id"] == "rugo.security_threat_model.v2"
    assert attack_data["runtime_capture_digest"]
    assert fuzz_data["profile_id"] == "rugo.security_hardening_profile.v3"
    assert advisory_data["policy_id"] == "rugo.security_advisory_policy.v1"
    assert embargo_data["policy_id"] == "rugo.vulnerability_response_policy.v1"
    assert embargo_data["meets_sla"] is True


def test_advisory_lint_rejects_invalid_payload(tmp_path: Path):
    advisory = {
        "advisory_id": "RUGO-2026-NEG-1",
        "severity": "urgent",
        "affected_versions": [],
        "fixed_versions": [],
        "published_utc": "2026-03-09",
    }
    advisory_path = tmp_path / "bad-advisory.json"
    advisory_path.write_text(json.dumps(advisory, indent=2) + "\n", encoding="utf-8")

    out = tmp_path / "security-advisory-lint-v1.json"
    rc = advisory_lint.main(["--advisory", str(advisory_path), "--out", str(out)])
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["valid"] is False
    assert "missing_field:cve_ids" in data["errors"]
    assert "invalid_severity:urgent" in data["errors"]
    assert "invalid_affected_versions" in data["errors"]
    assert "invalid_fixed_versions" in data["errors"]
    assert "invalid_published_utc" in data["errors"]


def test_embargo_drill_detects_injected_failure(tmp_path: Path):
    out = tmp_path / "security-embargo-drill-v1.json"
    rc = embargo_tool.main(
        [
            "--inject-failure",
            "triage_complete",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.security_embargo_drill_report.v1"
    assert data["total_failures"] >= 1
    assert data["success"] is False
    assert any(
        step["name"] == "triage_complete" and step["success"] is False
        for step in data["steps"]
    )
