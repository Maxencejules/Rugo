"""M25 PR-1: userspace service model and init contract v2 doc contracts."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_m25_pr1_artifacts_exist():
    required = [
        "docs/M25_EXECUTION_BACKLOG.md",
        "docs/runtime/service_model_v2.md",
        "docs/runtime/init_contract_v2.md",
        "tests/runtime/test_service_model_docs_v2.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M25 PR-1 artifact: {rel}"


def test_service_and_init_docs_declare_required_contract_tokens():
    service_doc = _read("docs/runtime/service_model_v2.md")
    init_doc = _read("docs/runtime/init_contract_v2.md")

    for token in [
        "Service Model ID: `rugo.service_model.v2`",
        "Dependency order schema: `rugo.service_dependency_order.v2`",
        "Lifecycle report schema: `rugo.service_lifecycle_report.v2`",
        "Restart report schema: `rugo.restart_policy_report.v2`",
        "Deterministic startup rule: topological dependency order, then lexical service",
        "Deterministic shutdown rule: reverse startup order.",
        "`never`",
        "`on-failure`",
        "`always`",
        "Maximum restart attempts per window: `3`.",
        "Restart window seconds: `60`.",
        "Backoff sequence seconds: `1, 2, 4`.",
        "Local gate: `make test-userspace-model-v2`",
    ]:
        assert token in service_doc

    for token in [
        "Init Contract ID: `rugo.init_contract.v2`",
        "Boot graph schema: `rugo.init_boot_graph.v2`",
        "Operational state schema: `rugo.init_operational_state.v2`",
        "Phase order: `bootstrap -> core -> services -> operational`.",
        "Required class: `critical`.",
        "Optional class: `best-effort`.",
        "Cycle policy: dependency cycles are release-blocking configuration errors.",
        "Failure policy: failure of a `critical` service blocks `operational`.",
        "Determinism rule: identical manifests must produce identical start/shutdown",
        "Boot-to-operational timeout budget: `45s`.",
    ]:
        assert token in init_doc


def test_m25_roadmap_anchor_declares_gate_name():
    roadmap = _read("docs/M21_M34_MATURITY_PARITY_ROADMAP.md")
    assert "test-userspace-model-v2" in roadmap
