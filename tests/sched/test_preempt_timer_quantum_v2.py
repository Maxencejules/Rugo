"""M16 acceptance: deterministic timer preemption and quantum behavior."""

from pathlib import Path
import sys

sys.path.append(str(Path(__file__).resolve().parent))
from sched_v2_model import PRIORITY_NORMAL, SchedulerV2Model, longest_run_streak  # noqa: E402


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_quantum_enforces_deterministic_preemption():
    model = SchedulerV2Model(quantum_ticks=3)
    model.add_task("alpha", PRIORITY_NORMAL, total_work=64)
    model.add_task("beta", PRIORITY_NORMAL, total_work=64)
    model.run(30)

    assert longest_run_streak(model.timeline) <= 3
    assert model.preemptions >= 8
    assert model.timeline[:12] == [
        "alpha",
        "alpha",
        "alpha",
        "beta",
        "beta",
        "beta",
        "alpha",
        "alpha",
        "alpha",
        "beta",
        "beta",
        "beta",
    ]


def test_scheduling_policy_v2_doc_declares_quantum_contract():
    policy = _read("docs/abi/scheduling_policy_v2.md")
    for token in [
        "Scheduler policy identifier: `rugo.scheduling_policy.v2`",
        "Default quantum: 3 ticks.",
        "Preemption is timer-driven and mandatory at quantum boundary.",
        "Deterministic tie-breaker: lowest `tid`",
        "sys_sched_set(tid, class)",
        "timesvc` and",
    ]:
        assert token in policy
