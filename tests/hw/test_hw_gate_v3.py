"""M23 aggregate gate: hardware matrix v3 wiring and closure checks."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_hw_matrix_v3_gate_wiring_and_artifacts():
    required = [
        "docs/M23_EXECUTION_BACKLOG.md",
        "docs/hw/support_matrix_v3.md",
        "docs/hw/driver_lifecycle_contract_v3.md",
        "tools/collect_hw_diagnostics_v3.py",
        "tests/hw/test_hardware_matrix_v3.py",
        "tests/hw/test_driver_lifecycle_v3.py",
        "tests/hw/test_suspend_resume_v1.py",
        "tests/hw/test_hotplug_baseline_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M23 artifact: {rel}"

    makefile = _read("Makefile")
    ci = _read(".github/workflows/ci.yml")
    backlog = _read("docs/M23_EXECUTION_BACKLOG.md")
    milestones = _read("MILESTONES.md")
    status = _read("docs/STATUS.md")
    readme = _read("README.md")

    assert "test-hw-matrix-v3" in makefile
    for entry in [
        "tools/collect_hw_diagnostics_v3.py --seed 20260306 --out $(OUT)/hw-diagnostics-v3.json",
        "tests/hw/test_hardware_matrix_v3.py",
        "tests/hw/test_driver_lifecycle_v3.py",
        "tests/hw/test_suspend_resume_v1.py",
        "tests/hw/test_hotplug_baseline_v1.py",
        "tests/hw/test_hw_gate_v3.py",
    ]:
        assert entry in makefile
    assert "pytest-hw-matrix-v3.xml" in makefile

    assert "Hardware matrix v3 gate" in ci
    assert "make test-hw-matrix-v3" in ci
    assert "hw-matrix-v3-artifacts" in ci
    assert "out/pytest-hw-matrix-v3.xml" in ci
    assert "out/hw-diagnostics-v3.json" in ci

    assert "Status: done" in backlog
    assert "M23" in milestones
    assert "M23" in status
    assert "M23" in readme
