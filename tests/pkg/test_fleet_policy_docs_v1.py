"""M33 PR-1: fleet update and health policy doc contract checks."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_m33_pr1_fleet_policy_artifacts_exist():
    required = [
        "docs/M33_EXECUTION_BACKLOG.md",
        "docs/pkg/fleet_update_policy_v1.md",
        "docs/runtime/fleet_health_policy_v1.md",
        "tests/pkg/test_fleet_policy_docs_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M33 PR-1 artifact: {rel}"


def test_fleet_update_policy_v1_doc_declares_required_tokens():
    doc = _read("docs/pkg/fleet_update_policy_v1.md")
    for token in [
        "Fleet update policy ID: `rugo.fleet_update_policy.v1`",
        "Fleet update simulation schema: `rugo.fleet_update_sim_report.v1`",
        "Rollout simulation schema: `rugo.canary_rollout_report.v1`",
        "Abort drill schema: `rugo.rollout_abort_drill_report.v1`",
        "Group promotion requires success rate meeting or exceeding `0.98`.",
        "runtime control plane mode: `runtime_lab`",
        "runtime capture artifact: `out/booted-runtime-v1.json`",
        "Fleet operations pass only when `max_failures = 0`.",
        "Local gate: `make test-fleet-ops-v1`",
        "CI gate: `Fleet ops v1 gate`",
        "CI sub-gate: `Fleet rollout safety v1 gate`",
    ]:
        assert token in doc


def test_fleet_health_policy_v1_doc_declares_required_tokens():
    doc = _read("docs/runtime/fleet_health_policy_v1.md")
    for token in [
        "Fleet health policy ID: `rugo.fleet_health_policy.v1`",
        "Fleet health simulation schema: `rugo.fleet_health_report.v1`",
        "maximum fleet degraded ratio: `0.05`",
        "maximum fleet error rate: `0.02`",
        "maximum canary latency p95: `200 ms`",
        "runtime capture artifact: `out/booted-runtime-v1.json`",
        "Fleet ops gate: `make test-fleet-ops-v1`",
        "Rollout safety sub-gate: `make test-fleet-rollout-safety-v1`",
    ]:
        assert token in doc


def test_m33_roadmap_anchor_declares_fleet_gates():
    roadmap = _read("docs/M21_M34_MATURITY_PARITY_ROADMAP.md")
    assert "test-fleet-ops-v1" in roadmap
    assert "test-fleet-rollout-safety-v1" in roadmap
