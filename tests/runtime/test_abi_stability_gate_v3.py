"""M21 aggregate gate: ABI/API stability v3 wiring and closure checks."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_abi_stability_v3_gate_wiring_and_artifacts():
    required = [
        "docs/M21_EXECUTION_BACKLOG.md",
        "docs/abi/syscall_v3.md",
        "docs/runtime/abi_stability_policy_v2.md",
        "docs/runtime/deprecation_window_policy_v1.md",
        "tools/check_abi_diff_v3.py",
        "tools/check_syscall_compat_v3.py",
        "tests/runtime/test_abi_docs_v3.py",
        "tests/runtime/test_abi_window_v3.py",
        "tests/runtime/test_abi_diff_gate_v3.py",
        "tests/compat/test_abi_compat_matrix_v3.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M21 artifact: {rel}"

    makefile = _read("Makefile")
    ci = _read(".github/workflows/ci.yml")
    backlog = _read("docs/M21_EXECUTION_BACKLOG.md")
    milestones = _read("MILESTONES.md")
    status = _read("docs/STATUS.md")

    assert "test-abi-stability-v3" in makefile
    for entry in [
        "tools/check_abi_diff_v3.py --out $(OUT)/abi-diff-v3.json",
        "tools/check_syscall_compat_v3.py --diff-report $(OUT)/abi-diff-v3.json --out $(OUT)/syscall-compat-v3.json",
        "tests/runtime/test_abi_docs_v3.py",
        "tests/runtime/test_abi_window_v3.py",
        "tests/runtime/test_abi_diff_gate_v3.py",
        "tests/compat/test_abi_compat_matrix_v3.py",
        "tests/runtime/test_abi_stability_gate_v3.py",
    ]:
        assert entry in makefile
    assert "pytest-abi-stability-v3.xml" in makefile

    assert "ABI stability v3 gate" in ci
    assert "make test-abi-stability-v3" in ci
    assert "abi-stability-v3-artifacts" in ci
    assert "out/pytest-abi-stability-v3.xml" in ci
    assert "out/abi-diff-v3.json" in ci
    assert "out/syscall-compat-v3.json" in ci

    assert "Status: done" in backlog
    assert "M21" in milestones
    assert "M21" in status

