"""M25 PR-2: deterministic service lifecycle semantics."""

from __future__ import annotations


SERVICE_MANIFEST = {
    "logger": {"deps": [], "class": "critical"},
    "devmgr": {"deps": ["logger"], "class": "critical"},
    "netd": {"deps": ["devmgr"], "class": "critical"},
    "pkgd": {"deps": ["devmgr"], "class": "best-effort"},
    "sshd": {"deps": ["netd"], "class": "best-effort"},
}


def _start_order(manifest: dict[str, dict[str, object]]) -> list[str]:
    pending = set(manifest.keys())
    started: list[str] = []

    while pending:
        ready = sorted(
            name
            for name in pending
            if all(dep in started for dep in manifest[name]["deps"])  # type: ignore[index]
        )
        if not ready:
            raise ValueError("dependency cycle detected")
        next_name = ready[0]
        started.append(next_name)
        pending.remove(next_name)

    return started


def _simulate_boot(fail_service: str | None = None) -> dict[str, object]:
    order = _start_order(SERVICE_MANIFEST)
    states = {name: "declared" for name in SERVICE_MANIFEST}
    timeline: list[str] = []

    for name in order:
        deps = SERVICE_MANIFEST[name]["deps"]
        if any(states[dep] != "ready" for dep in deps):
            states[name] = "blocked"
            timeline.append(f"blocked:{name}")
            continue

        states[name] = "starting"
        timeline.append(f"starting:{name}")

        if fail_service == name:
            states[name] = "failed"
            timeline.append(f"failed:{name}")
            continue

        states[name] = "running"
        timeline.append(f"running:{name}")
        states[name] = "ready"
        timeline.append(f"ready:{name}")

    critical = [
        name
        for name, cfg in SERVICE_MANIFEST.items()
        if cfg["class"] == "critical"
    ]
    operational = all(states[name] == "ready" for name in critical)
    return {
        "schema": "rugo.service_lifecycle_report.v2",
        "start_order": order,
        "states": states,
        "timeline": timeline,
        "operational": operational,
    }


def _simulate_shutdown(start_order: list[str], states: dict[str, str]) -> list[str]:
    ready = [name for name in start_order if states[name] == "ready"]
    return list(reversed(ready))


def test_boot_to_operational_is_deterministic():
    first = _simulate_boot()
    second = _simulate_boot()

    assert first == second
    assert first["schema"] == "rugo.service_lifecycle_report.v2"
    assert first["start_order"] == ["logger", "devmgr", "netd", "pkgd", "sshd"]
    assert first["operational"] is True


def test_shutdown_order_is_reverse_of_ready_start_order():
    report = _simulate_boot()
    shutdown_order = _simulate_shutdown(report["start_order"], report["states"])
    assert shutdown_order == ["sshd", "pkgd", "netd", "devmgr", "logger"]


def test_critical_failure_blocks_operational_state_and_dependents():
    report = _simulate_boot(fail_service="devmgr")
    states = report["states"]

    assert report["operational"] is False
    assert states["devmgr"] == "failed"
    assert states["netd"] == "blocked"
    assert states["pkgd"] == "blocked"
    assert states["sshd"] == "blocked"


def test_optional_failure_does_not_block_operational_state():
    report = _simulate_boot(fail_service="pkgd")
    states = report["states"]

    assert report["operational"] is True
    assert states["logger"] == "ready"
    assert states["devmgr"] == "ready"
    assert states["netd"] == "ready"
    assert states["pkgd"] == "failed"
    assert states["sshd"] == "ready"
