"""M28 runtime-backed hardening evidence checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import collect_booted_runtime_v1 as capture_tool  # noqa: E402
import run_security_attack_suite_v3 as attack_suite  # noqa: E402


def test_runtime_hardening_v3_binds_to_booted_capture(tmp_path: Path):
    capture_out = tmp_path / "booted-runtime-v1.json"
    report_out = tmp_path / "security-attack-suite-v3.json"

    assert capture_tool.main(["--fixture", "--out", str(capture_out)]) == 0
    assert (
        attack_suite.main(
            [
                "--runtime-capture",
                str(capture_out),
                "--out",
                str(report_out),
            ]
        )
        == 0
    )

    capture = json.loads(capture_out.read_text(encoding="utf-8"))
    report = json.loads(report_out.read_text(encoding="utf-8"))
    assert report["runtime_capture_digest"] == capture["digest"]
    assert report["hardening_defaults"]["defaults_enforced"] is True
    runtime_cases = [case for case in report["cases"] if case["case_type"] == "runtime_hardening"]
    assert {case["name"] for case in runtime_cases} == {
        "syscall_filter_bypass",
        "capability_rights_escalation",
    }
    assert all(case["pass"] is True for case in runtime_cases)


def test_runtime_hardening_v3_detects_missing_runtime_denial_marker(tmp_path: Path):
    capture_out = tmp_path / "booted-runtime-v1.json"
    mutated_capture = tmp_path / "booted-runtime-missing-denial.json"
    report_out = tmp_path / "security-attack-suite-v3-fail.json"

    assert capture_tool.main(["--fixture", "--out", str(capture_out)]) == 0
    capture = json.loads(capture_out.read_text(encoding="utf-8"))
    for boot in capture["boots"]:
        boot["serial_lines"] = [
            entry
            for entry in boot["serial_lines"]
            if "GOSH: spawn deny" not in entry["line"]
        ]
    mutated_capture.write_text(json.dumps(capture, indent=2) + "\n", encoding="utf-8")

    assert (
        attack_suite.main(
            [
                "--runtime-capture",
                str(mutated_capture),
                "--out",
                str(report_out),
            ]
        )
        == 1
    )

    report = json.loads(report_out.read_text(encoding="utf-8"))
    failed = {case["name"] for case in report["cases"] if case["pass"] is False}
    assert "syscall_filter_bypass" in failed
