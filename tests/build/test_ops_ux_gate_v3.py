"""M30 aggregate gate: installer/upgrade/recovery UX v3 wiring and closure."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_recovery_drill_v3 as recovery_tool  # noqa: E402
import run_upgrade_drill_v3 as upgrade_tool  # noqa: E402


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_ops_ux_gate_v3_wiring_and_artifacts(tmp_path: Path):
    required = [
        "docs/M30_EXECUTION_BACKLOG.md",
        "docs/build/installer_ux_v3.md",
        "docs/build/recovery_workflow_v3.md",
        "tools/run_upgrade_drill_v3.py",
        "tools/run_recovery_drill_v3.py",
        "tests/build/test_installer_ux_v3.py",
        "tests/build/test_upgrade_recovery_v3.py",
        "tests/build/test_rollback_safety_v3.py",
        "tests/build/test_ops_ux_gate_v3.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M30 artifact: {rel}"

    roadmap = _read("docs/M21_M34_MATURITY_PARITY_ROADMAP.md")
    makefile = _read("Makefile")
    ci = _read(".github/workflows/ci.yml")
    backlog = _read("docs/M30_EXECUTION_BACKLOG.md")
    milestones = _read("MILESTONES.md")
    status = _read("docs/STATUS.md")
    readme = _read("README.md")

    assert "test-ops-ux-v3" in roadmap

    assert "test-ops-ux-v3" in makefile
    for entry in [
        "tools/run_upgrade_drill_v3.py --seed 20260309 --out $(OUT)/upgrade-drill-v3.json",
        "tools/run_recovery_drill_v3.py --seed 20260309 --out $(OUT)/recovery-drill-v3.json",
        "tests/build/test_installer_ux_v3.py",
        "tests/build/test_upgrade_recovery_v3.py",
        "tests/build/test_rollback_safety_v3.py",
        "tests/build/test_ops_ux_gate_v3.py",
    ]:
        assert entry in makefile
    assert "pytest-ops-ux-v3.xml" in makefile

    assert "Ops UX v3 gate" in ci
    assert "make test-ops-ux-v3" in ci
    assert "ops-ux-v3-artifacts" in ci
    assert "out/pytest-ops-ux-v3.xml" in ci
    assert "out/upgrade-drill-v3.json" in ci
    assert "out/recovery-drill-v3.json" in ci

    assert "Status: done" in backlog
    assert "M30" in milestones
    assert "M30" in status
    assert "M30" in readme

    upgrade_out = tmp_path / "upgrade-drill-v3.json"
    recovery_out = tmp_path / "recovery-drill-v3.json"

    assert upgrade_tool.main(["--seed", "20260309", "--out", str(upgrade_out)]) == 0
    assert recovery_tool.main(["--seed", "20260309", "--out", str(recovery_out)]) == 0

    upgrade_data = json.loads(upgrade_out.read_text(encoding="utf-8"))
    recovery_data = json.loads(recovery_out.read_text(encoding="utf-8"))

    assert upgrade_data["schema"] == "rugo.upgrade_drill.v3"
    assert upgrade_data["gate_pass"] is True
    assert upgrade_data["total_failures"] == 0
    assert recovery_data["schema"] == "rugo.recovery_drill.v3"
    assert recovery_data["gate_pass"] is True
    assert recovery_data["total_failures"] == 0
