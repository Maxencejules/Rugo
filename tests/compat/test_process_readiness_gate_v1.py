"""M41 aggregate gate: process/readiness parity v1 wiring and closure checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_compat_surface_campaign_v2 as campaign  # noqa: E402
import run_posix_gap_report_v2 as gap  # noqa: E402


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_process_readiness_gate_v1_wiring_and_artifacts(tmp_path: Path):
    required = [
        "docs/M41_EXECUTION_BACKLOG.md",
        "docs/abi/compat_profile_v5.md",
        "docs/runtime/syscall_coverage_matrix_v4.md",
        "docs/abi/process_model_v4.md",
        "docs/abi/readiness_io_model_v1.md",
        "tools/run_compat_surface_campaign_v2.py",
        "tools/run_posix_gap_report_v2.py",
        "tests/compat/test_compat_docs_v5.py",
        "tests/compat/test_fork_clone_surface_v1.py",
        "tests/compat/test_epoll_surface_v1.py",
        "tests/compat/test_process_model_v4.py",
        "tests/compat/test_deferred_surface_behavior_v2.py",
        "tests/compat/test_posix_gap_closure_gate_v2.py",
        "tests/compat/test_process_readiness_gate_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M41 artifact: {rel}"

    roadmap = _read("docs/M40_M44_GENERAL_PURPOSE_PARITY_ROADMAP.md")
    makefile = _read("Makefile")
    ci = _read(".github/workflows/ci.yml")
    backlog = _read("docs/M41_EXECUTION_BACKLOG.md")
    milestones = _read("MILESTONES.md")
    status = _read("docs/STATUS.md")
    readme = _read("README.md")

    assert "test-process-readiness-parity-v1" in roadmap
    assert "test-posix-gap-closure-v2" in roadmap

    assert "test-process-readiness-parity-v1" in makefile
    for entry in [
        "tools/run_compat_surface_campaign_v2.py --out $(OUT)/compat-surface-v2.json",
        "$(MAKE) test-posix-gap-closure-v2",
        "tests/compat/test_compat_docs_v5.py",
        "tests/compat/test_fork_clone_surface_v1.py",
        "tests/compat/test_epoll_surface_v1.py",
        "tests/compat/test_process_model_v4.py",
        "tests/compat/test_deferred_surface_behavior_v2.py",
        "tests/compat/test_process_readiness_gate_v1.py",
    ]:
        assert entry in makefile
    assert "pytest-process-readiness-parity-v1.xml" in makefile
    assert "pytest-posix-gap-closure-v2.xml" in makefile

    assert "Process readiness parity v1 gate" in ci
    assert "make test-process-readiness-parity-v1" in ci
    assert "process-readiness-parity-v1-artifacts" in ci
    assert "out/pytest-process-readiness-parity-v1.xml" in ci
    assert "out/compat-surface-v2.json" in ci

    assert "POSIX gap closure v2 gate" in ci
    assert "make test-posix-gap-closure-v2" in ci
    assert "posix-gap-closure-v2-artifacts" in ci
    assert "out/pytest-posix-gap-closure-v2.xml" in ci
    assert "out/posix-gap-report-v2.json" in ci

    assert "Status: done" in backlog
    assert (
        "| M41 | Process + Readiness Compatibility Closure v1 | n/a | done |"
        in milestones
    )
    assert (
        "| **M41** Process + Readiness Compatibility Closure v1 | n/a | done |"
        in status
    )
    assert "M0-M41: done" in readme
    assert "M41 execution backlog (completed)" in readme

    surface_out = tmp_path / "compat-surface-v2.json"
    gap_out = tmp_path / "posix-gap-report-v2.json"

    assert campaign.main(["--seed", "20260310", "--out", str(surface_out)]) == 0
    assert gap.main(["--seed", "20260310", "--out", str(gap_out)]) == 0

    surface_data = json.loads(surface_out.read_text(encoding="utf-8"))
    gap_data = json.loads(gap_out.read_text(encoding="utf-8"))

    assert surface_data["schema"] == "rugo.compat_surface_campaign_report.v2"
    assert surface_data["profile_id"] == "rugo.compat_profile.v5"
    assert surface_data["gate_pass"] is True
    assert surface_data["total_failures"] == 0

    assert gap_data["schema"] == "rugo.posix_gap_report.v2"
    assert gap_data["profile_id"] == "rugo.compat_profile.v5"
    assert gap_data["gate_pass"] is True
    assert gap_data["total_failures"] == 0
