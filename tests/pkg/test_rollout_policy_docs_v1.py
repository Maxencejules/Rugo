"""M33 PR-1: staged rollout and canary SLO doc contract checks."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_m33_pr1_rollout_policy_artifacts_exist():
    required = [
        "docs/M33_EXECUTION_BACKLOG.md",
        "docs/pkg/staged_rollout_policy_v1.md",
        "docs/runtime/canary_slo_policy_v1.md",
        "tools/run_canary_rollout_sim_v1.py",
        "tools/run_rollout_abort_drill_v1.py",
        "tests/pkg/test_rollout_policy_docs_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M33 PR-1 artifact: {rel}"


def test_staged_rollout_policy_v1_doc_declares_required_tokens():
    doc = _read("docs/pkg/staged_rollout_policy_v1.md")
    for token in [
        "Staged rollout policy ID: `rugo.staged_rollout_policy.v1`",
        "Report schema: `rugo.canary_rollout_report.v1`",
        "Abort drill schema: `rugo.rollout_abort_drill_report.v1`",
        "Stage: `canary`",
        "blast-radius budget: `1%`",
        "Stage: `small_batch`",
        "blast-radius budget: `10%`",
        "Stage: `broad`",
        "maximum canary error rate: `0.02`",
        "maximum canary latency p95: `200 ms`",
        "runtime control plane mode: `runtime_lab`",
        "runtime capture artifact: `out/booted-runtime-v1.json`",
        "Local sub-gate: `make test-fleet-rollout-safety-v1`",
        "Parent gate: `make test-fleet-ops-v1`",
    ]:
        assert token in doc


def test_canary_slo_policy_v1_doc_declares_required_tokens():
    doc = _read("docs/runtime/canary_slo_policy_v1.md")
    for token in [
        "Canary SLO policy ID: `rugo.canary_slo_policy.v1`",
        "Abort drill schema: `rugo.rollout_abort_drill_report.v1`",
        "`slo_error_rate_threshold`",
        "`slo_latency_p95_ms_threshold`",
        "`observed_error_rate`",
        "`observed_latency_p95_ms`",
        "`auto_halt`",
        "`rollback_triggered`",
        "`halt_reason`",
        "`recovery_actions`",
        "Any SLO breach is gate-blocking.",
        "Local sub-gate: `make test-fleet-rollout-safety-v1`",
    ]:
        assert token in doc
