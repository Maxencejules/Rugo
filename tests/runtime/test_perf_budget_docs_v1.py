"""M24 PR-1: performance budget and benchmark policy v1 doc contracts."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_m24_pr1_artifacts_exist():
    required = [
        "docs/M24_EXECUTION_BACKLOG.md",
        "docs/runtime/performance_budget_v1.md",
        "docs/runtime/benchmark_policy_v1.md",
        "tools/collect_booted_runtime_v1.py",
        "tests/runtime/test_booted_runtime_capture_v1.py",
        "tests/runtime/test_perf_budget_docs_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M24 PR-1 artifact: {rel}"


def test_perf_docs_declare_required_contract_tokens():
    budget_doc = _read("docs/runtime/performance_budget_v1.md")
    policy_doc = _read("docs/runtime/benchmark_policy_v1.md")

    for token in [
        "Budget ID: `rugo.performance_budget.v1`",
        "Booted runtime schema: `rugo.booted_runtime_capture.v1`",
        "Baseline schema: `rugo.perf_baseline.v1`",
        "Regression schema: `rugo.perf_regression_report.v1`",
        "`cpu_service_cycle`",
        "`memory_diag_snapshot`",
        "`block_recovery_cycle`",
        "`network_roundtrip_cycle`",
        "`service_restart_cycle`",
        "`mixed_runtime_cycle`",
        "Default release image: `out/os-go.iso`",
        "Runtime capture artifact: `out/booted-runtime-v1.json`",
        "make test-perf-regression-v1",
    ]:
        assert token in budget_doc

    for token in [
        "Policy ID: `rugo.benchmark_policy.v1`",
        "Primary owner: Runtime maintainers.",
        "Secondary owner: Release engineering maintainers.",
        "Default release image: `out/os-go.iso`.",
        "Fixture seed: `20260318`.",
        "Minimum boots per capture: `2`.",
        "Runtime capture lane: `qemu`.",
        "Gate outcome: `total_violations` must be `0`.",
        "tools/collect_booted_runtime_v1.py",
        "tools/run_perf_baseline_v1.py",
        "tools/check_perf_regression_v1.py",
    ]:
        assert token in policy_doc
