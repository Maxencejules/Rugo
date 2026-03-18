"""M28 PR-1: hardening profile v3 and threat model v2 doc contracts."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_m28_pr1_artifacts_exist():
    required = [
        "docs/M28_EXECUTION_BACKLOG.md",
        "docs/security/hardening_profile_v3.md",
        "docs/security/threat_model_v2.md",
        "docs/security/vulnerability_response_policy_v1.md",
        "docs/security/security_advisory_policy_v1.md",
        "tests/security/test_hardening_docs_v3.py",
        "tests/security/test_vuln_response_docs_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M28 PR-1 artifact: {rel}"


def test_hardening_docs_declare_required_contract_tokens():
    hardening_doc = _read("docs/security/hardening_profile_v3.md")
    threat_doc = _read("docs/security/threat_model_v2.md")

    for token in [
        "Hardening profile ID: `rugo.security_hardening_profile.v3`",
        "Threat model reference: `rugo.security_threat_model.v2`",
        "Attack suite report schema: `rugo.security_attack_suite_report.v3`",
        "Fuzz report schema: `rugo.security_fuzz_report.v2`",
        "Control ID: `SEC-HARD-V3-A1`",
        "Control ID: `SEC-HARD-V3-B2`",
        "Runtime evidence artifact: `out/booted-runtime-v1.json`",
        "`GOSH: recv deny`",
        "`GOSH: reg deny`",
        "`GOSH: spawn deny`",
        "Local gate: `make test-security-hardening-v3`",
        "Sub-gate: `make test-vuln-response-v1`",
    ]:
        assert token in hardening_doc

    for token in [
        "Threat model ID: `rugo.security_threat_model.v2`",
        "Hardening profile linkage: `rugo.security_hardening_profile.v3`",
        "Attack scenario `syscall_filter_bypass`",
        "Attack scenario `capability_rights_escalation`",
        "Attack scenario `unsigned_advisory_publish`",
        "Maximum closure SLA hours: `120`.",
        "total_failures == 0",
        "total_violations == 0",
    ]:
        assert token in threat_doc


def test_m28_roadmap_anchor_declares_gate_names():
    roadmap = _read("docs/M21_M34_MATURITY_PARITY_ROADMAP.md")
    assert "test-security-hardening-v3" in roadmap
    assert "test-vuln-response-v1" in roadmap
