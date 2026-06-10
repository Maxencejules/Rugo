"""Deterministic reference model for M16 scheduler/process semantics."""

from __future__ import annotations

from dataclasses import dataclass
from hashlib import sha256
import random


PRIORITY_HIGH = 0
PRIORITY_NORMAL = 1
PRIORITY_LOW = 2

PRIORITY_WEIGHTS = {
    PRIORITY_HIGH: 3,
    PRIORITY_NORMAL: 2,
    PRIORITY_LOW: 1,
}

# Virtual-runtime deltas per tick. Lower delta means more CPU share.
VRUNTIME_STEP = {
    PRIORITY_HIGH: 341,
    PRIORITY_NORMAL: 512,
    PRIORITY_LOW: 1024,
}


@dataclass
class TaskState:
    tid: int
    name: str
    priority: int
    total_work: int
    remaining_work: int
    vruntime: int = 0
    executed_ticks: int = 0
    dispatches: int = 0
    ready_wait: int = 0
    max_ready_wait: int = 0
    blocked_until: int | None = None
    exited: bool = False


class SchedulerV2Model:
    """Small deterministic scheduler model with weighted fairness."""

    def __init__(self, quantum_ticks: int = 3, starvation_limit: int = 350):
        if quantum_ticks <= 0:
            raise ValueError("quantum_ticks must be > 0")
        self.quantum_ticks = quantum_ticks
        self.starvation_limit = starvation_limit
        self.tasks: dict[int, TaskState] = {}
        self.current_tid: int | None = None
        self.slice_used = 0
        self.tick_count = 0
        self.idle_ticks = 0
        self.preemptions = 0
        self.voluntary_yields = 0
        self.timeline: list[str] = []

    def add_task(self, name: str, priority: int, total_work: int) -> int:
        if priority not in PRIORITY_WEIGHTS:
            raise ValueError("invalid priority")
        if total_work <= 0:
            raise ValueError("total_work must be > 0")

        tid = len(self.tasks) + 1
        self.tasks[tid] = TaskState(
            tid=tid,
            name=name,
            priority=priority,
            total_work=total_work,
            remaining_work=total_work,
        )
        return tid

    def _is_runnable(self, task: TaskState) -> bool:
        return not task.exited and task.blocked_until is None

    def _unblock_ready_tasks(self) -> None:
        for task in self.tasks.values():
            if task.blocked_until is not None and task.blocked_until <= self.tick_count:
                task.blocked_until = None

    def _pick_next_tid(self) -> int | None:
        runnable = [t for t in self.tasks.values() if self._is_runnable(t)]
        if not runnable:
            return None
        # Deterministic choice: min vruntime, then priority class, then tid.
        nxt = min(runnable, key=lambda t: (t.vruntime, t.priority, t.tid))
        return nxt.tid

    def force_yield(self) -> int:
        if self.current_tid is None:
            return -1
        self.current_tid = None
        self.slice_used = 0
        self.voluntary_yields += 1
        return 0

    def block_task(self, tid: int, block_ticks: int) -> int:
        task = self.tasks.get(tid)
        if task is None or task.exited or block_ticks <= 0:
            return -1
        task.blocked_until = self.tick_count + block_ticks
        task.ready_wait = 0
        if self.current_tid == tid:
            self.current_tid = None
            self.slice_used = 0
        return 0

    def tick(self) -> str:
        self._unblock_ready_tasks()

        if self.current_tid is None:
            self.current_tid = self._pick_next_tid()
            self.slice_used = 0
            if self.current_tid is not None:
                self.tasks[self.current_tid].dispatches += 1

        if self.current_tid is None:
            self.tick_count += 1
            self.idle_ticks += 1
            self.timeline.append("IDLE")
            return "IDLE"

        runnable_tids = [t.tid for t in self.tasks.values() if self._is_runnable(t)]
        for tid in runnable_tids:
            task = self.tasks[tid]
            if tid == self.current_tid:
                task.ready_wait = 0
                continue
            task.ready_wait += 1
            if task.ready_wait > task.max_ready_wait:
                task.max_ready_wait = task.ready_wait

        task = self.tasks[self.current_tid]
        task.executed_ticks += 1
        task.remaining_work -= 1
        task.vruntime += VRUNTIME_STEP[task.priority]

        self.slice_used += 1
        self.tick_count += 1
        self.timeline.append(task.name)

        if task.remaining_work <= 0:
            task.exited = True
            task.blocked_until = None
            self.current_tid = None
            self.slice_used = 0
        elif self.slice_used >= self.quantum_ticks:
            self.preemptions += 1
            self.current_tid = None
            self.slice_used = 0

        return task.name

    def run(self, ticks: int) -> None:
        for _ in range(max(0, ticks)):
            self.tick()

    def report(self) -> dict:
        digest_src = ",".join(self.timeline).encode("utf-8")
        per_task = {}
        for task in sorted(self.tasks.values(), key=lambda t: t.tid):
            per_task[task.name] = {
                "tid": task.tid,
                "priority": task.priority,
                "weight": PRIORITY_WEIGHTS[task.priority],
                "executed_ticks": task.executed_ticks,
                "dispatches": task.dispatches,
                "max_ready_wait": task.max_ready_wait,
                "vruntime": task.vruntime,
                "exited": task.exited,
            }
        return {
            "schema": "rugo.scheduler_soak_report.v2",
            "ticks": self.tick_count,
            "quantum_ticks": self.quantum_ticks,
            "preemptions": self.preemptions,
            "voluntary_yields": self.voluntary_yields,
            "idle_ticks": self.idle_ticks,
            "timeline_digest": sha256(digest_src).hexdigest(),
            "per_task": per_task,
        }


def longest_run_streak(timeline: list[str]) -> int:
    streak = 0
    longest = 0
    last = None
    for item in timeline:
        if item == last:
            streak += 1
        else:
            last = item
            streak = 1
        if streak > longest:
            longest = streak
    return longest


def run_scheduler_soak(
    seed: int,
    ticks: int = 1200,
    task_count: int = 9,
    quantum_ticks: int = 3,
) -> dict:
    """Run a seeded soak campaign and return a machine-readable report."""

    rng = random.Random(seed)
    model = SchedulerV2Model(quantum_ticks=quantum_ticks, starvation_limit=350)

    for idx in range(task_count):
        priority = rng.choices(
            [PRIORITY_HIGH, PRIORITY_NORMAL, PRIORITY_LOW],
            weights=[2, 3, 2],
            k=1,
        )[0]
        # Keep tasks runnable for the full campaign.
        total_work = ticks + 300 + rng.randint(0, 400)
        model.add_task(name=f"task-{idx + 1}", priority=priority, total_work=total_work)

    for _ in range(ticks):
        if model.current_tid is not None and rng.random() < 0.05:
            model.block_task(model.current_tid, block_ticks=rng.randint(1, 4))
        model.tick()
        if model.current_tid is not None and rng.random() < 0.02:
            model.force_yield()

    report = model.report()
    report["seed"] = seed

    anomalies = []
    for name, stats in report["per_task"].items():
        if stats["executed_ticks"] == 0:
            anomalies.append({"task": name, "reason": "never_executed"})
        if stats["max_ready_wait"] > model.starvation_limit:
            anomalies.append(
                {
                    "task": name,
                    "reason": "ready_wait_exceeded",
                    "max_ready_wait": stats["max_ready_wait"],
                    "limit": model.starvation_limit,
                }
            )

    report["anomalies"] = anomalies
    return report
