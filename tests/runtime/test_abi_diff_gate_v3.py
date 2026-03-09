"""M21 PR-2: ABI diff gate tool behavior."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import check_abi_diff_v3 as abi_diff  # noqa: E402


def _write(path: Path, text: str) -> None:
    path.write_text(text, encoding="utf-8")


def test_abi_diff_v3_repo_baseline_is_non_breaking(tmp_path: Path):
    out = tmp_path / "abi-diff-v3.json"
    rc = abi_diff.main(["--out", str(out)])
    assert rc == 0
    assert out.is_file()

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.abi_diff_report.v3"
    assert data["base_contract_id"] == "rugo.syscall_abi.v2"
    assert data["candidate_contract_id"] == "rugo.syscall_abi.v3"
    assert data["breaking_change_count"] == 0
    assert data["gate_pass"] is True


def test_abi_diff_v3_detects_renumbered_breaking_changes(tmp_path: Path):
    base = tmp_path / "syscall_v2.md"
    candidate = tmp_path / "syscall_v3.md"
    out = tmp_path / "abi-diff-v3.json"

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
                "| 1 | `sys_thread_exit` |",
                "| 2 | `sys_thread_spawn` |",
                "",
            ]
        ),
    )

    rc = abi_diff.main(
        [
            "--base",
            str(base),
            "--candidate",
            str(candidate),
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["breaking_change_count"] >= 1
    kinds = {issue["kind"] for issue in data["breaking_changes"]}
    assert "renamed" in kinds or "renumbered" in kinds
    assert data["gate_pass"] is False

