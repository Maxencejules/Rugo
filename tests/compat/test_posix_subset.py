"""M8 PR-3 closure checks for Compatibility Profile v1."""

from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
COMPAT_DIR = REPO_ROOT / "tests" / "compat"
PROFILE_DOC = REPO_ROOT / "docs" / "abi" / "compat_profile_v1.md"


def test_required_profile_tests_exist():
    required = [
        COMPAT_DIR / "test_loader_contract.py",
        COMPAT_DIR / "test_process_lifecycle.py",
        COMPAT_DIR / "test_process_wait.py",
        COMPAT_DIR / "test_file_io_subset.py",
        COMPAT_DIR / "test_fd_table.py",
        COMPAT_DIR / "test_time_signal_subset.py",
        COMPAT_DIR / "test_socket_api_subset.py",
    ]
    for path in required:
        assert path.is_file(), f"missing compatibility coverage file: {path}"


def test_compat_profile_no_pr3_skeleton_markers():
    text = PROFILE_DOC.read_text(encoding="utf-8")
    assert "Conformance skeleton (PR-3 target)" not in text
    assert "M8 PR-3 will close remaining" not in text


def test_no_remaining_compat_todo_gate_usage():
    for path in sorted(COMPAT_DIR.glob("test_*.py")):
        if path.name == "test_posix_subset.py":
            continue
        text = path.read_text(encoding="utf-8")
        assert "compat_todo(" not in text, f"TODO gate remains in {path.name}"
