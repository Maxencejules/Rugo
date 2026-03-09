"""M21 PR-2: compatibility matrix and policy-check behavior for ABI v3."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import check_abi_diff_v3 as abi_diff  # noqa: E402
import check_syscall_compat_v3 as compat_check  # noqa: E402


def _write(path: Path, text: str) -> None:
    path.write_text(text, encoding="utf-8")


def test_syscall_compat_matrix_v3_repo_baseline(tmp_path: Path):
    diff_out = tmp_path / "abi-diff-v3.json"
    compat_out = tmp_path / "syscall-compat-v3.json"

    assert abi_diff.main(["--out", str(diff_out)]) == 0
    assert (
        compat_check.main(
            [
                "--diff-report",
                str(diff_out),
                "--out",
                str(compat_out),
            ]
        )
        == 0
    )

    data = json.loads(compat_out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.syscall_compat_report.v3"
    assert data["minimum_tagged_release_window"] == 3
    assert data["minimum_calendar_days"] == 180
    assert data["compat_matrix"][0]["direction"] == "backward"
    assert data["compat_matrix"][0]["compatible"] is True
    assert data["requires_explicit_migration"] is False
    assert data["policy_issues"] == []
    assert data["gate_pass"] is True


def test_syscall_compat_requires_explicit_actions_for_breaking_diff(tmp_path: Path):
    base = tmp_path / "syscall_v2.md"
    candidate = tmp_path / "syscall_v3.md"
    diff_out = tmp_path / "abi-diff-v3.json"
    compat_out = tmp_path / "syscall-compat-v3.json"
    migration_doc = tmp_path / "migration-v4.md"

    _write(
        base,
        "\n".join(
            [
                "# Syscall ABI v2",
                "Syscall ABI identifier: `rugo.syscall_abi.v2`.",
                "| ID | Name |",
                "|----|------|",
                "| 1 | `sys_thread_spawn` |",
                "| 2 | `sys_thread_exit` |",
                "",
            ]
        ),
    )
    _write(
        candidate,
        "\n".join(
            [
                "# Syscall ABI v3",
                "Syscall ABI identifier: `rugo.syscall_abi.v3`.",
                "| ID | Name |",
                "|----|------|",
                "| 1 | `sys_thread_spawn` |",
                "",
            ]
        ),
    )
    _write(migration_doc, "# Migration v4\n")

    assert (
        abi_diff.main(
            [
                "--base",
                str(base),
                "--candidate",
                str(candidate),
                "--allow-breaking",
                "--out",
                str(diff_out),
            ]
        )
        == 0
    )

    assert (
        compat_check.main(
            [
                "--base",
                str(base),
                "--candidate",
                str(candidate),
                "--diff-report",
                str(diff_out),
                "--out",
                str(compat_out),
            ]
        )
        == 1
    )

    assert (
        compat_check.main(
            [
                "--base",
                str(base),
                "--candidate",
                str(candidate),
                "--diff-report",
                str(diff_out),
                "--version-action",
                "major-abi-bump",
                "--migration-doc",
                str(migration_doc),
                "--out",
                str(compat_out),
            ]
        )
        == 0
    )

    data = json.loads(compat_out.read_text(encoding="utf-8"))
    assert data["requires_explicit_migration"] is True
    assert data["explicit_actions"]["version_action"] == "major-abi-bump"
    assert data["explicit_actions"]["migration_doc"] == str(migration_doc)
    assert data["gate_pass"] is True
