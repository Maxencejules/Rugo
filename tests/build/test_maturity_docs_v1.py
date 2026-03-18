"""M34 PR-1: maturity qualification and LTS declaration doc contract checks."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_m34_pr1_maturity_policy_artifacts_exist():
    required = [
        "docs/M34_EXECUTION_BACKLOG.md",
        "docs/build/maturity_qualification_v1.md",
        "docs/build/lts_declaration_policy_v1.md",
        "tests/build/test_maturity_docs_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M34 PR-1 artifact: {rel}"


def test_maturity_qualification_v1_doc_declares_required_tokens():
    doc = _read("docs/build/maturity_qualification_v1.md")
    for token in [
        "Maturity qualification policy ID: `rugo.maturity_qualification_policy.v1`",
        "Qualification bundle schema: `rugo.maturity_qualification_bundle.v1`",
        "LTS declaration policy ID: `rugo.lts_declaration_policy.v1`",
        "LTS declaration schema: `rugo.lts_declaration_report.v1`",
        "minimum qualified release count: `3`",
        "minimum support window for LTS baseline: `730 days`",
        "runtime capture evidence artifact: `out/booted-runtime-v1.json`",
        "package rebuild evidence artifact: `out/pkg-rebuild-v3.json`",
        "`server_v1` and `appliance_v1` are the only LTS-qualified profiles.",
        "`developer_v1` remains outside the LTS declaration surface.",
        "`make test-vuln-response-v1`",
        "`make test-supply-chain-revalidation-v1`",
        "`make test-fleet-rollout-safety-v1`",
        "`make test-conformance-v1`",
        "`make test-release-lifecycle-v2`",
        "all cross-domain evidence checks must pass with `max_failures = 0`",
        "Final local gate: `make test-maturity-qual-v1`",
        "Final CI gate: `Maturity qualification v1 gate`",
    ]:
        assert token in doc


def test_lts_declaration_policy_v1_doc_declares_required_tokens():
    doc = _read("docs/build/lts_declaration_policy_v1.md")
    for token in [
        "LTS declaration policy ID: `rugo.lts_declaration_policy.v1`",
        "Declaration report schema: `rugo.lts_declaration_report.v1`",
        "Qualification dependency schema: `rugo.maturity_qualification_bundle.v1`",
        "minimum qualified releases: `3`",
        "minimum support window: `730 days`",
        "maximum advisory SLA breach count: `0`",
        "supply-chain drift tolerance: `0`",
        "LTS target scope: `qemu-q35-default-lane`",
        "LTS-qualified profiles: `server_v1`, `appliance_v1`",
        "Non-LTS profile: `developer_v1`",
        "Maturity gate: `make test-maturity-qual-v1`",
        "CI gate: `Maturity qualification v1 gate`",
    ]:
        assert token in doc


def test_m34_roadmap_anchor_declares_maturity_gate():
    roadmap = _read("docs/M21_M34_MATURITY_PARITY_ROADMAP.md")
    assert "test-maturity-qual-v1" in roadmap
