"""M28 PR-2: deterministic attack-suite v3 tooling checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_security_attack_suite_v3 as attack_suite  # noqa: E402


def _strip_timestamp(payload: dict) -> dict:
    stable = dict(payload)
    stable.pop("created_utc", None)
    return stable


def test_attack_suite_v3_is_seed_deterministic():
    first = attack_suite.run_suite(seed=20260309, fixture=True)
    second = attack_suite.run_suite(seed=20260309, fixture=True)
    assert _strip_timestamp(first) == _strip_timestamp(second)
    assert first["runtime_capture_digest"]


def test_attack_suite_v3_report_schema_and_pass(tmp_path: Path):
    out = tmp_path / "security-attack-suite-v3.json"
    rc = attack_suite.main(
        [
            "--seed",
            "20260309",
            "--fixture",
            "--max-failures",
            "0",
            "--out",
            str(out),
        ]
    )
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.security_attack_suite_report.v3"
    assert data["suite_id"] == "rugo.security_attack_suite.v3"
    assert data["profile_id"] == "rugo.security_hardening_profile.v3"
    assert data["threat_model_id"] == "rugo.security_threat_model.v2"
    assert data["runtime_capture_digest"]
    assert data["hardening_defaults"]["defaults_enforced"] is True
    assert data["total_cases"] >= 5
    assert data["total_failures"] == 0
    assert data["gate_pass"] is True


def test_attack_suite_v3_detects_injected_failure(tmp_path: Path):
    out = tmp_path / "security-attack-suite-v3.json"
    rc = attack_suite.main(
        [
            "--seed",
            "20260309",
            "--fixture",
            "--inject-failure",
            "syscall_filter_bypass",
            "--max-failures",
            "0",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.security_attack_suite_report.v3"
    assert data["total_failures"] >= 1
    assert data["gate_pass"] is False
    assert any(
        case["name"] == "syscall_filter_bypass" and case["pass"] is False
        for case in data["cases"]
    )
