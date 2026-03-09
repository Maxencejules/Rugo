"""M22 aggregate gate: kernel reliability v1 wiring and closure checks."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_kernel_reliability_v1_gate_wiring_and_artifacts():
    required = [
        "docs/M22_EXECUTION_BACKLOG.md",
        "docs/runtime/kernel_reliability_model_v1.md",
        "tools/run_kernel_soak_v1.py",
        "tools/run_fault_campaign_kernel_v1.py",
        "tests/stress/test_kernel_soak_24h_v1.py",
        "tests/stress/test_fault_injection_matrix_v1.py",
        "tests/stress/test_reliability_artifact_schema_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M22 artifact: {rel}"

    makefile = _read("Makefile")
    ci = _read(".github/workflows/ci.yml")
    backlog = _read("docs/M22_EXECUTION_BACKLOG.md")
    milestones = _read("MILESTONES.md")
    status = _read("docs/STATUS.md")

    assert "test-kernel-reliability-v1" in makefile
    for entry in [
        "tools/run_kernel_soak_v1.py --seed 20260306 --out $(OUT)/kernel-soak-v1.json",
        "tools/run_fault_campaign_kernel_v1.py --seed 20260306 --out $(OUT)/kernel-fault-campaign-v1.json",
        "tests/stress/test_kernel_soak_24h_v1.py",
        "tests/stress/test_fault_injection_matrix_v1.py",
        "tests/stress/test_reliability_artifact_schema_v1.py",
        "tests/stress/test_kernel_reliability_gate_v1.py",
    ]:
        assert entry in makefile
    assert "pytest-kernel-reliability-v1.xml" in makefile

    assert "Kernel reliability v1 gate" in ci
    assert "make test-kernel-reliability-v1" in ci
    assert "kernel-reliability-v1-artifacts" in ci
    assert "out/pytest-kernel-reliability-v1.xml" in ci
    assert "out/kernel-soak-v1.json" in ci
    assert "out/kernel-fault-campaign-v1.json" in ci

    assert "Status: done" in backlog
    assert "M22" in milestones
    assert "M22" in status

