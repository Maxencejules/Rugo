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
        "tests/runtime/test_perf_budget_docs_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M24 PR-1 artifact: {rel}"


def test_perf_docs_declare_required_contract_tokens():
    budget_doc = _read("docs/runtime/performance_budget_v1.md")
    policy_doc = _read("docs/runtime/benchmark_policy_v1.md")

    for token in [
        "Budget ID: `rugo.performance_budget.v1`",
        "Baseline schema: `rugo.perf_baseline.v1`",
        "Regression schema: `rugo.perf_regression_report.v1`",
        "`syscall_spam`",
        "`ipc_loop`",
        "`blk_loop`",
        "`pressure_shm`",
        "`thread_spawn`",
        "`vm_map`",
        "make test-perf-regression-v1",
    ]:
        assert token in budget_doc

    for token in [
        "Policy ID: `rugo.benchmark_policy.v1`",
        "Primary owner: Runtime maintainers.",
        "Secondary owner: Release engineering maintainers.",
        "Default seed: `20260309`.",
        "Minimum iterations per class: `1200`.",
        "Gate outcome: `total_violations` must be `0`.",
        "tools/run_perf_baseline_v1.py",
        "tools/check_perf_regression_v1.py",
    ]:
        assert token in policy_doc
