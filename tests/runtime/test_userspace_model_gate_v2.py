"""M25 aggregate gate: userspace service model + init v2 wiring and closure."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_userspace_model_v2_gate_wiring_and_artifacts():
    required = [
        "docs/M25_EXECUTION_BACKLOG.md",
        "docs/runtime/service_model_v2.md",
        "docs/runtime/init_contract_v2.md",
        "tests/runtime/test_service_model_docs_v2.py",
        "tests/runtime/test_service_lifecycle_v2.py",
        "tests/runtime/test_service_dependency_order_v2.py",
        "tests/runtime/test_restart_policy_v2.py",
        "tests/runtime/test_userspace_model_gate_v2.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M25 artifact: {rel}"

    roadmap = _read("docs/M21_M34_MATURITY_PARITY_ROADMAP.md")
    makefile = _read("Makefile")
    ci = _read(".github/workflows/ci.yml")
    backlog = _read("docs/M25_EXECUTION_BACKLOG.md")
    milestones = _read("MILESTONES.md")
    status = _read("docs/STATUS.md")
    readme = _read("README.md")

    assert "test-userspace-model-v2" in roadmap
    assert "test-userspace-model-v2" in makefile
    for entry in [
        "tests/runtime/test_service_model_docs_v2.py",
        "tests/runtime/test_service_lifecycle_v2.py",
        "tests/runtime/test_service_dependency_order_v2.py",
        "tests/runtime/test_restart_policy_v2.py",
        "tests/runtime/test_userspace_model_gate_v2.py",
    ]:
        assert entry in makefile
    assert "pytest-userspace-model-v2.xml" in makefile

    assert "Userspace model v2 gate" in ci
    assert "make test-userspace-model-v2" in ci
    assert "userspace-model-v2-artifacts" in ci
    assert "out/pytest-userspace-model-v2.xml" in ci

    assert "Status: done" in backlog
    assert "M25" in milestones
    assert "M25" in status
    assert "M25" in readme
