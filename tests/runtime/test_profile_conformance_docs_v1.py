"""M32 PR-1: profile conformance v1 doc contract checks."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_m32_pr1_profile_conformance_artifacts_exist():
    required = [
        "docs/M32_EXECUTION_BACKLOG.md",
        "docs/runtime/profile_conformance_v1.md",
        "tools/run_conformance_suite_v1.py",
        "tests/runtime/test_profile_conformance_docs_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M32 PR-1 artifact: {rel}"


def test_profile_conformance_v1_doc_declares_required_tokens():
    doc = _read("docs/runtime/profile_conformance_v1.md")
    for token in [
        "Conformance policy ID: `rugo.profile_conformance_policy.v1`",
        "Conformance report schema: `rugo.profile_conformance_report.v1`",
        "Profile requirement schema: `rugo.profile_requirement_set.v1`",
        "`server_v1`",
        "`developer_v1`",
        "`appliance_v1`",
        "`service_restart_coverage_pct` must be `>= 95`.",
        "`interactive_shell_latency_ms_p95` must be `<= 200`.",
        "`package_build_success_rate_pct` must be `>= 98`.",
        "`read_only_runtime_pct` must be `>= 99`.",
        "Runtime capture artifact: `out/booted-runtime-v1.json`",
        "Release attestation artifact: `out/release-attestation-verification-v1.json`",
        "Package rebuild artifact: `out/pkg-rebuild-v3.json`",
        "Conformance suite tool: `tools/run_conformance_suite_v1.py`",
        "Local gate: `make test-conformance-v1`",
        "CI gate: `Conformance v1 gate`",
        "Release artifact: `out/conformance-v1.json`",
    ]:
        assert token in doc


def test_m32_roadmap_anchor_declares_conformance_gate():
    roadmap = _read("docs/M21_M34_MATURITY_PARITY_ROADMAP.md")
    assert "test-conformance-v1" in roadmap
