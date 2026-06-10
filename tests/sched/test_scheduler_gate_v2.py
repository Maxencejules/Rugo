"""M16 aggregate gate: process/scheduler v2 contract and gate wiring."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_process_scheduler_v2_gate_wiring_and_artifacts():
    required = [
        "docs/M16_EXECUTION_BACKLOG.md",
        "docs/abi/process_thread_model_v2.md",
        "docs/abi/scheduling_policy_v2.md",
        "tests/user/test_process_wait_kill_v2.py",
        "tests/user/test_signal_delivery_v2.py",
        "tests/runtime/test_process_scheduler_runtime_v2.py",
        "tests/sched/sched_v2_model.py",
        "tests/sched/test_preempt_timer_quantum_v2.py",
        "tests/sched/test_priority_fairness_v2.py",
        "tests/sched/test_scheduler_soak_v2.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M16 artifact: {rel}"

    makefile = _read("Makefile")
    ci = _read(".github/workflows/ci.yml")
    backlog = _read("docs/M16_EXECUTION_BACKLOG.md")
    milestones = _read("MILESTONES.md")
    status = _read("docs/STATUS.md")

    assert "test-process-scheduler-v2" in makefile
    assert "test-process-scheduler-v2: image-thread-spawn image-thread-exit image-yield image-user-fault image-go" in makefile
    for test_name in [
        "tests/sched/test_preempt_timer_quantum_v2.py",
        "tests/sched/test_priority_fairness_v2.py",
        "tests/sched/test_scheduler_soak_v2.py",
        "tests/user/test_process_wait_kill_v2.py",
        "tests/user/test_signal_delivery_v2.py",
        "tests/runtime/test_process_scheduler_runtime_v2.py",
        "tests/sched/test_scheduler_gate_v2.py",
    ]:
        assert test_name in makefile
    assert "pytest-process-scheduler-v2.xml" in makefile

    assert "Process scheduler v2 gate" in ci
    assert "make test-process-scheduler-v2" in ci
    assert "process-scheduler-v2-junit" in ci

    assert "Status: done" in backlog
    assert "M16" in milestones
    assert "M16" in status
