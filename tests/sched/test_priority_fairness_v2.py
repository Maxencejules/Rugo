"""M16 acceptance: priority fairness contract for scheduler v2."""

from pathlib import Path
import sys

sys.path.append(str(Path(__file__).resolve().parent))
from sched_v2_model import (  # noqa: E402
    PRIORITY_HIGH,
    PRIORITY_LOW,
    PRIORITY_NORMAL,
    PRIORITY_WEIGHTS,
    SchedulerV2Model,
)


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_equal_priority_tasks_converge_to_near_equal_service():
    model = SchedulerV2Model(quantum_ticks=3)
    model.add_task("n1", PRIORITY_NORMAL, total_work=400)
    model.add_task("n2", PRIORITY_NORMAL, total_work=400)
    model.add_task("n3", PRIORITY_NORMAL, total_work=400)
    model.run(180)

    report = model.report()
    ticks = [stats["executed_ticks"] for stats in report["per_task"].values()]
    assert max(ticks) - min(ticks) <= model.quantum_ticks


def test_weighted_priority_fairness_bias():
    model = SchedulerV2Model(quantum_ticks=3)
    model.add_task("hi", PRIORITY_HIGH, total_work=500)
    model.add_task("lo", PRIORITY_LOW, total_work=500)
    model.run(240)

    report = model.report()["per_task"]
    hi = report["hi"]["executed_ticks"]
    lo = report["lo"]["executed_ticks"]

    # High-priority task must receive materially more service than low.
    assert hi > lo * 2

    hi_norm = hi / PRIORITY_WEIGHTS[PRIORITY_HIGH]
    lo_norm = lo / PRIORITY_WEIGHTS[PRIORITY_LOW]
    # Weighted fairness keeps normalized service within a bounded distance.
    assert abs(hi_norm - lo_norm) <= 20


def test_scheduling_policy_v2_doc_declares_fairness_rules():
    policy = _read("docs/abi/scheduling_policy_v2.md")
    for token in [
        "Weighted fairness is enforced by priority classes:",
        "- `high`: weight `3`",
        "- `normal`: weight `2`",
        "- `low`: weight `1`",
        "Lower-priority tasks may receive less service, but starvation is disallowed.",
    ]:
        assert token in policy
