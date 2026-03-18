"""M34 aggregate gate: maturity qualification wiring and closure checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_maturity_qualification_v1 as maturity_tool  # noqa: E402


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_maturity_gate_v1_wiring_and_artifacts(tmp_path: Path):
    required = [
        "docs/M34_EXECUTION_BACKLOG.md",
        "docs/build/maturity_qualification_v1.md",
        "docs/build/lts_declaration_policy_v1.md",
        "tools/run_maturity_qualification_v1.py",
        "tests/build/test_maturity_docs_v1.py",
        "tests/build/test_maturity_qualification_v1.py",
        "tests/build/test_lts_policy_v1.py",
        "tests/build/test_lts_surface_scope_v1.py",
        "tests/build/test_maturity_security_response_drill_v1.py",
        "tests/build/test_maturity_supply_chain_continuity_v1.py",
        "tests/build/test_maturity_rollout_safety_v1.py",
        "tests/build/test_maturity_gate_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M34 artifact: {rel}"

    roadmap = _read("docs/M21_M34_MATURITY_PARITY_ROADMAP.md")
    makefile = _read("Makefile")
    ci = _read(".github/workflows/ci.yml")
    backlog = _read("docs/M34_EXECUTION_BACKLOG.md")
    milestones = _read("MILESTONES.md")
    status = _read("docs/STATUS.md")
    readme = _read("README.md")

    assert "test-maturity-qual-v1" in roadmap

    assert "test-maturity-qual-v1" in makefile
    for entry in [
        "tools/collect_booted_runtime_v1.py --image $(OUT)/os-go.iso --kernel $(OUT)/kernel-go.elf --panic-image $(OUT)/os-panic.iso --out $(OUT)/booted-runtime-v1.json",
        "tools/run_maturity_qualification_v1.py --seed 20260309 --runtime-capture $(OUT)/booted-runtime-v1.json --out $(OUT)/maturity-qualification-v1.json",
        "tests/build/test_maturity_docs_v1.py",
        "tests/build/test_maturity_qualification_v1.py",
        "tests/build/test_lts_policy_v1.py",
        "tests/build/test_lts_surface_scope_v1.py",
        "tests/build/test_maturity_security_response_drill_v1.py",
        "tests/build/test_maturity_supply_chain_continuity_v1.py",
        "tests/build/test_maturity_rollout_safety_v1.py",
        "tests/build/test_maturity_gate_v1.py",
    ]:
        assert entry in makefile
    assert "pytest-maturity-qual-v1.xml" in makefile

    assert "Maturity qualification v1 gate" in ci
    assert "make test-maturity-qual-v1" in ci
    assert "maturity-qualification-v1-artifacts" in ci
    assert "out/pytest-maturity-qual-v1.xml" in ci
    assert "out/booted-runtime-v1.json" in ci
    assert "out/pkg-rebuild-v3.json" in ci
    assert "out/maturity-qualification-v1.json" in ci

    assert "Status: done" in backlog
    assert "M34" in milestones
    assert "M34" in status
    assert "docs/architecture/SOURCE_MAP.md" in readme
    assert "docs/archive/README.md" in readme

    report_out = tmp_path / "maturity-qualification-v1.json"
    assert maturity_tool.main(["--seed", "20260309", "--fixture", "--out", str(report_out)]) == 0
    report = json.loads(report_out.read_text(encoding="utf-8"))
    assert report["schema"] == "rugo.maturity_qualification_bundle.v1"
    assert report["policy_id"] == "rugo.maturity_qualification_policy.v1"
    assert report["qualification_pass"] is True
    assert report["total_failures"] == 0
    assert report["evidence_artifacts"]["runtime_capture"].endswith("booted-runtime-v1.json")
    assert report["evidence_artifacts"]["pkg_rebuild"].endswith("pkg-rebuild-v3.json")
    assert report["lts_declaration"]["policy_id"] == "rugo.lts_declaration_policy.v1"
    assert report["lts_declaration"]["eligible"] is True
