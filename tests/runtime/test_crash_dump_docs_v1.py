"""M29 PR-1: crash dump contract and postmortem playbook doc checks."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_m29_crash_doc_artifacts_exist():
    required = [
        "docs/M29_EXECUTION_BACKLOG.md",
        "docs/runtime/crash_dump_contract_v1.md",
        "docs/runtime/postmortem_triage_playbook_v1.md",
        "tools/collect_crash_dump_v1.py",
        "tools/symbolize_crash_dump_v1.py",
        "tests/runtime/test_crash_dump_docs_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M29 crash artifact: {rel}"


def test_crash_docs_declare_required_contract_tokens():
    contract = _read("docs/runtime/crash_dump_contract_v1.md")
    playbook = _read("docs/runtime/postmortem_triage_playbook_v1.md")

    for token in [
        "Crash dump contract ID: `rugo.crash_dump_contract.v1`",
        "Dump schema: `rugo.crash_dump.v1`",
        "Symbolized schema: `rugo.crash_dump_symbolized.v1`",
        "Symbol map ID: `rugo.kernel_symbol_map.v1`",
        "Triage playbook linkage: `rugo.postmortem_triage_playbook.v1`",
        "Register set with `rip`, `rsp`, and `rbp`.",
        "Release identity with `kernel_build_id` and `release_channel`.",
        "Local sub-gate: `make test-crash-dump-v1`",
        "Parent gate: `make test-observability-v2`",
    ]:
        assert token in contract

    for token in [
        "Playbook ID: `rugo.postmortem_triage_playbook.v1`",
        "Triage bundle schema: `rugo.postmortem_triage_bundle.v1`",
        "Crash dump input schema: `rugo.crash_dump.v1`",
        "Symbolized input schema: `rugo.crash_dump_symbolized.v1`",
        "Classify incident (`panic`, `deadlock`, `oom`, `data_corruption`).",
        "Initial triage SLA: `4` hours.",
        "Root-cause classification SLA: `24` hours.",
        "Sub-gate: `make test-crash-dump-v1`",
    ]:
        assert token in playbook


def test_m29_roadmap_anchor_declares_crash_gate():
    roadmap = _read("docs/M21_M34_MATURITY_PARITY_ROADMAP.md")
    assert "test-crash-dump-v1" in roadmap
