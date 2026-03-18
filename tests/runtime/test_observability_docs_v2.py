"""M29 PR-1: observability contract v2 doc checks."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_m29_observability_pr1_artifacts_exist():
    required = [
        "docs/M29_EXECUTION_BACKLOG.md",
        "docs/runtime/observability_contract_v2.md",
        "docs/runtime/crash_dump_contract_v1.md",
        "docs/runtime/postmortem_triage_playbook_v1.md",
        "tools/collect_booted_runtime_v1.py",
        "tests/runtime/test_booted_runtime_capture_v1.py",
        "tests/runtime/test_observability_docs_v2.py",
        "tests/runtime/test_crash_dump_docs_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M29 PR-1 artifact: {rel}"


def test_observability_doc_declares_required_contract_tokens():
    observability_doc = _read("docs/runtime/observability_contract_v2.md")

    for token in [
        "Observability contract ID: `rugo.observability_contract.v2`",
        "Booted runtime schema: `rugo.booted_runtime_capture.v1`",
        "Trace bundle schema: `rugo.trace_bundle.v2`",
        "Diagnostic snapshot schema: `rugo.diagnostic_snapshot.v2`",
        "Crash dump schema: `rugo.crash_dump.v1`",
        "Symbolized crash schema: `rugo.crash_dump_symbolized.v1`",
        "Postmortem triage playbook: `rugo.postmortem_triage_playbook.v1`",
        "Structured log stream for `goinit`, `gosvcm`, `timesvc`, `diagsvc`, `shell`,",
        "Diagnostic health checks for `service_manager`, `memory_pressure`,",
        "Cross-layer identifiers: `build_id`, `trace_id`, `boot_id`, and",
        "Trace window seconds: `300`.",
        "Maximum allowed trace errors: `0`.",
        "Maximum allowed dropped trace spans: `0`.",
        "Maximum allowed unhealthy diagnostic checks: `0`.",
        "Maximum allowed unresolved crash frames: `0`.",
        "tools/collect_booted_runtime_v1.py",
        "Local gate: `make test-observability-v2`",
        "Sub-gate: `make test-crash-dump-v1`",
        "`Observability v2 gate`",
        "`Crash dump v1 gate`",
    ]:
        assert token in observability_doc


def test_m29_roadmap_anchor_declares_observability_gates():
    roadmap = _read("docs/M21_M34_MATURITY_PARITY_ROADMAP.md")
    assert "test-observability-v2" in roadmap
    assert "test-crash-dump-v1" in roadmap
