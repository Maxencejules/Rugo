"""M28 aggregate gate: security hardening v3 wiring and closure checks."""

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
import collect_booted_runtime_v1 as capture_tool  # noqa: E402


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_security_hardening_gate_v3_wiring_and_artifacts(tmp_path: Path):
    required = [
        "docs/M28_EXECUTION_BACKLOG.md",
        "docs/security/hardening_profile_v3.md",
        "docs/security/threat_model_v2.md",
        "docs/security/vulnerability_response_policy_v1.md",
        "docs/security/security_advisory_policy_v1.md",
        "tools/collect_booted_runtime_v1.py",
        "tools/run_security_attack_suite_v3.py",
        "tools/run_security_fuzz_v2.py",
        "tools/security_advisory_lint_v1.py",
        "tools/security_embargo_drill_v1.py",
        "tests/security/test_hardening_docs_v3.py",
        "tests/security/test_attack_suite_v3.py",
        "tests/security/test_runtime_hardening_evidence_v3.py",
        "tests/security/test_fuzz_gate_v2.py",
        "tests/security/test_policy_enforcement_v3.py",
        "tests/security/test_vuln_response_docs_v1.py",
        "tests/security/test_vuln_triage_sla_v1.py",
        "tests/security/test_embargo_workflow_v1.py",
        "tests/security/test_advisory_schema_v1.py",
        "tests/security/test_vuln_response_gate_v1.py",
        "tests/security/test_security_hardening_gate_v3.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M28 artifact: {rel}"

    roadmap = _read("docs/M21_M34_MATURITY_PARITY_ROADMAP.md")
    makefile = _read("Makefile")
    ci = _read(".github/workflows/ci.yml")
    backlog = _read("docs/M28_EXECUTION_BACKLOG.md")
    milestones = _read("MILESTONES.md")
    status = _read("docs/STATUS.md")
    readme = _read("README.md")

    assert "test-security-hardening-v3" in roadmap
    assert "test-vuln-response-v1" in roadmap

    assert "test-security-hardening-v3" in makefile
    for entry in [
        "tools/collect_booted_runtime_v1.py --image $(OUT)/os-go.iso --kernel $(OUT)/kernel-go.elf --panic-image $(OUT)/os-panic.iso --out $(OUT)/booted-runtime-v1.json",
        "tools/run_security_attack_suite_v3.py --seed 20260309 --runtime-capture $(OUT)/booted-runtime-v1.json --out $(OUT)/security-attack-suite-v3.json",
        "tools/run_security_fuzz_v2.py --seed 20260309 --iterations 1600 --cases 5 --out $(OUT)/security-fuzz-v2.json",
        "tests/security/test_hardening_docs_v3.py",
        "tests/security/test_attack_suite_v3.py",
        "tests/security/test_runtime_hardening_evidence_v3.py",
        "tests/security/test_fuzz_gate_v2.py",
        "tests/security/test_policy_enforcement_v3.py",
        "tests/security/test_security_hardening_gate_v3.py",
        "tools/security_advisory_lint_v1.py --out $(OUT)/security-advisory-lint-v1.json",
        "tools/security_embargo_drill_v1.py --out $(OUT)/security-embargo-drill-v1.json",
        "tests/security/test_vuln_response_gate_v1.py",
    ]:
        assert entry in makefile
    assert "pytest-security-hardening-v3.xml" in makefile
    assert "pytest-vuln-response-v1.xml" in makefile

    assert "Security hardening v3 gate" in ci
    assert "make test-security-hardening-v3" in ci
    assert "security-hardening-v3-artifacts" in ci
    assert "out/pytest-security-hardening-v3.xml" in ci
    assert "out/booted-runtime-v1.json" in ci
    assert "out/security-attack-suite-v3.json" in ci
    assert "out/security-fuzz-v2.json" in ci

    assert "Vulnerability response v1 gate" in ci
    assert "make test-vuln-response-v1" in ci
    assert "vuln-response-v1-artifacts" in ci
    assert "out/pytest-vuln-response-v1.xml" in ci
    assert "out/security-advisory-lint-v1.json" in ci
    assert "out/security-embargo-drill-v1.json" in ci

    assert "Status: done" in backlog
    assert "M28" in milestones
    assert "M28" in status
    assert "docs/architecture/SOURCE_MAP.md" in readme
    assert "docs/archive/README.md" in readme

    capture_out = tmp_path / "booted-runtime-v1.json"
    attack_out = tmp_path / "security-attack-suite-v3.json"
    fuzz_out = tmp_path / "security-fuzz-v2.json"
    advisory_out = tmp_path / "security-advisory-lint-v1.json"
    embargo_out = tmp_path / "security-embargo-drill-v1.json"

    assert capture_tool.main(["--fixture", "--out", str(capture_out)]) == 0
    assert (
        attack_suite.main(
            [
                "--seed",
                "20260309",
                "--runtime-capture",
                str(capture_out),
                "--out",
                str(attack_out),
            ]
        )
        == 0
    )
    assert (
        fuzz_tool.main(
            [
                "--seed",
                "20260309",
                "--iterations",
                "1200",
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

    assert attack_data["schema"] == "rugo.security_attack_suite_report.v3"
    assert attack_data["gate_pass"] is True
    assert attack_data["runtime_capture_digest"]
    assert fuzz_data["schema"] == "rugo.security_fuzz_report.v2"
    assert fuzz_data["gate_pass"] is True
    assert advisory_data["schema"] == "rugo.security_advisory_lint_report.v1"
    assert advisory_data["valid"] is True
    assert embargo_data["schema"] == "rugo.security_embargo_drill_report.v1"
    assert embargo_data["success"] is True
