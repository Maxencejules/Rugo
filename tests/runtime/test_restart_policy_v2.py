"""M25 PR-2: bounded restart policy semantics."""

from __future__ import annotations


MAX_RESTARTS = 3
WINDOW_SECONDS = 60
BACKOFF_SECONDS = [1, 2, 4]


def _should_restart(policy: str, exit_code: int, restart_count: int) -> bool:
    if restart_count >= MAX_RESTARTS:
        return False
    if policy == "never":
        return False
    if policy == "always":
        return True
    if policy == "on-failure":
        return exit_code != 0
    raise ValueError(f"unknown policy: {policy}")


def _simulate(policy: str, exits: list[int]) -> dict[str, object]:
    restart_count = 0
    delays: list[int] = []
    events: list[str] = []

    for exit_code in exits:
        events.append(f"exit:{exit_code}")
        if _should_restart(policy, exit_code, restart_count):
            delays.append(BACKOFF_SECONDS[min(restart_count, len(BACKOFF_SECONDS) - 1)])
            restart_count += 1
            events.append("restart")
            continue
        events.append("stop")
        break

    return {
        "schema": "rugo.restart_policy_report.v2",
        "policy": policy,
        "window_seconds": WINDOW_SECONDS,
        "max_restarts": MAX_RESTARTS,
        "restart_count": restart_count,
        "delays_seconds": delays,
        "bounded": restart_count <= MAX_RESTARTS,
        "events": events,
    }


def test_on_failure_restarts_until_cap_then_stops():
    report = _simulate("on-failure", exits=[2, 3, 4, 5, 6])
    assert report["schema"] == "rugo.restart_policy_report.v2"
    assert report["restart_count"] == 3
    assert report["delays_seconds"] == [1, 2, 4]
    assert report["bounded"] is True
    assert report["events"][-1] == "stop"


def test_always_restarts_even_on_clean_exit_until_cap():
    report = _simulate("always", exits=[0, 0, 0, 0])
    assert report["restart_count"] == 3
    assert report["delays_seconds"] == [1, 2, 4]
    assert report["events"][-1] == "stop"


def test_never_policy_does_not_restart():
    report = _simulate("never", exits=[9, 9, 9])
    assert report["restart_count"] == 0
    assert report["delays_seconds"] == []
    assert report["events"] == ["exit:9", "stop"]


def test_restart_policy_is_deterministic_for_same_inputs():
    first = _simulate("on-failure", exits=[1, 1, 0])
    second = _simulate("on-failure", exits=[1, 1, 0])
    assert first == second
