"""M50 PR-2: deterministic window z-order checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_window_system_runtime_v1 as runtime  # noqa: E402


def _out_path(name: str) -> Path:
    path = ROOT / "out" / "pytest-m50" / name
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        path.unlink()
    return path


def _strip_timestamp(payload: dict) -> dict:
    stable = dict(payload)
    stable.pop("created_utc", None)
    return stable


def test_window_zorder_v1_deterministic_report():
    first = runtime.run_window_system_runtime(seed=20260311)
    second = runtime.run_window_system_runtime(seed=20260311)
    assert _strip_timestamp(first) == _strip_timestamp(second)


def test_window_zorder_v1_schema_and_focus_alignment():
    out = _out_path("window-system-v1-zorder.json")
    rc = runtime.main(["--seed", "20260311", "--out", str(out)])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    stack = data["z_order"]["stack"]

    assert data["window_manager_contract_id"] == "rugo.window_manager_contract.v2"
    assert [row["window_id"] for row in stack] == [
        "desktop.shell.workspace",
        "files.panel",
        "settings.panel",
    ]
    assert data["z_order"]["topmost_focusable_window"] == "settings.panel"
    assert data["z_order"]["focus_owner"] == "settings.panel"
    assert data["z_order"]["ordering_violations"] == 0
    assert data["z_order"]["focus_alignment_pass"] is True
    assert data["summary"]["z_order"]["pass"] is True


def test_window_zorder_v1_detects_focus_alignment_regression():
    out = _out_path("window-system-v1-zorder-fail.json")
    rc = runtime.main(
        [
            "--inject-failure",
            "focus_z_order_alignment",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    assert "focus_z_order_alignment" in data["failures"]
    assert data["summary"]["z_order"]["failures"] >= 1
    assert data["z_order"]["topmost_focusable_window"] == "settings.panel"
    assert data["z_order"]["focus_owner"] == "files.panel"
    assert data["z_order"]["focus_alignment_pass"] is False
