"""M27 aggregate gate: app compatibility v3 wiring and closure."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_app_compat_matrix_v3 as matrix  # noqa: E402


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_app_compat_gate_v3_wiring_and_artifacts(tmp_path: Path):
    required = [
        "docs/M27_EXECUTION_BACKLOG.md",
        "docs/abi/compat_profile_v3.md",
        "docs/abi/app_compat_tiers_v1.md",
        "tools/run_app_compat_matrix_v3.py",
        "tests/compat/test_app_tier_docs_v1.py",
        "tests/compat/test_cli_suite_v3.py",
        "tests/compat/test_runtime_suite_v3.py",
        "tests/compat/test_service_suite_v3.py",
        "tests/compat/test_app_compat_gate_v3.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M27 artifact: {rel}"

    roadmap = _read("docs/M21_M34_MATURITY_PARITY_ROADMAP.md")
    makefile = _read("Makefile")
    ci = _read(".github/workflows/ci.yml")
    backlog = _read("docs/M27_EXECUTION_BACKLOG.md")
    milestones = _read("MILESTONES.md")
    status = _read("docs/STATUS.md")
    readme = _read("README.md")

    assert "test-app-compat-v3" in roadmap

    assert "test-app-compat-v3" in makefile
    for entry in [
        "tools/run_app_compat_matrix_v3.py --seed 20260309 --out $(OUT)/app-compat-matrix-v3.json",
        "tests/compat/test_app_tier_docs_v1.py",
        "tests/compat/test_cli_suite_v3.py",
        "tests/compat/test_runtime_suite_v3.py",
        "tests/compat/test_service_suite_v3.py",
        "tests/compat/test_app_compat_gate_v3.py",
    ]:
        assert entry in makefile
    assert "pytest-app-compat-v3.xml" in makefile

    assert "App compatibility v3 gate" in ci
    assert "make test-app-compat-v3" in ci
    assert "app-compat-v3-artifacts" in ci
    assert "out/pytest-app-compat-v3.xml" in ci
    assert "out/app-compat-matrix-v3.json" in ci

    assert "Status: done" in backlog
    assert "M27" in milestones
    assert "M27" in status
    assert "M27" in readme

    out = tmp_path / "app-compat-matrix-v3.json"
    assert matrix.main(["--seed", "20260309", "--out", str(out)]) == 0
    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.app_compat_matrix_report.v3"
    assert data["profile_id"] == "rugo.compat_profile.v3"
    assert data["tier_schema"] == "rugo.app_compat_tiers.v1"
    assert data["gate_pass"] is True
    assert data["classes"]["cli"]["meets_threshold"] is True
    assert data["classes"]["runtime"]["meets_threshold"] is True
    assert data["classes"]["service"]["meets_threshold"] is True
