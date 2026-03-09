"""M21 PR-1: ABI stability v3 docs and policy contract checks."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_m21_pr1_artifacts_exist():
    required = [
        "docs/M21_EXECUTION_BACKLOG.md",
        "docs/abi/syscall_v3.md",
        "docs/runtime/abi_stability_policy_v2.md",
        "docs/runtime/deprecation_window_policy_v1.md",
        "tests/runtime/test_abi_docs_v3.py",
        "tests/runtime/test_abi_window_v3.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M21 PR-1 artifact: {rel}"


def test_v3_docs_and_policies_declare_required_contract_tokens():
    syscall_doc = _read("docs/abi/syscall_v3.md")
    abi_policy = _read("docs/runtime/abi_stability_policy_v2.md")
    deprecation_policy = _read("docs/runtime/deprecation_window_policy_v1.md")

    for token in [
        "Syscall ABI identifier: `rugo.syscall_abi.v3`.",
        "Freeze window: `v3.x`.",
        "No syscall ID renumbering is allowed in `v3.x`.",
        "Breaking changes require all of:",
        "| 27 | `sys_sec_profile_set` | required | active |",
    ]:
        assert token in syscall_doc

    for token in [
        "Policy ID: `rugo.runtime_abi_policy.v2`",
        "Stability window:",
        "backward-compatible for at least three",
        "tools/check_abi_diff_v3.py",
        "tools/check_syscall_compat_v3.py",
        "make test-abi-stability-v3",
    ]:
        assert token in abi_policy

    for token in [
        "Policy ID: `rugo.deprecation_window_policy.v1`",
        "Minimum tagged-release window: 3.",
        "Minimum calendar window: 180 days.",
        "Removal within the same ABI line (`v3.x`) is forbidden.",
    ]:
        assert token in deprecation_policy
