"""M25 PR-2: deterministic dependency ordering semantics."""

from __future__ import annotations


def _resolve_start_order(manifest: dict[str, list[str]]) -> list[str]:
    pending = set(manifest.keys())
    started: list[str] = []

    for name, deps in manifest.items():
        for dep in deps:
            if dep not in manifest:
                raise ValueError(f"unknown dependency: {name} -> {dep}")

    while pending:
        ready = sorted(name for name in pending if all(dep in started for dep in manifest[name]))
        if not ready:
            raise ValueError("dependency cycle detected")
        next_name = ready[0]
        started.append(next_name)
        pending.remove(next_name)

    return started


def test_dependency_order_is_topological_then_lexical():
    manifest = {
        "logger": [],
        "auth": ["logger"],
        "cache": ["logger"],
        "db": ["logger"],
        "api": ["auth", "cache", "db"],
        "metrics": ["logger"],
    }

    assert _resolve_start_order(manifest) == [
        "logger",
        "auth",
        "cache",
        "db",
        "api",
        "metrics",
    ]


def test_shutdown_order_is_reverse_start_order():
    manifest = {
        "logger": [],
        "devmgr": ["logger"],
        "netd": ["devmgr"],
        "sshd": ["netd"],
    }
    start = _resolve_start_order(manifest)
    assert list(reversed(start)) == ["sshd", "netd", "devmgr", "logger"]


def test_dependency_cycle_is_rejected():
    manifest = {
        "alpha": ["beta"],
        "beta": ["alpha"],
    }
    try:
        _resolve_start_order(manifest)
    except ValueError as exc:
        assert "cycle" in str(exc)
    else:
        raise AssertionError("expected ValueError for dependency cycle")


def test_missing_dependency_is_rejected():
    manifest = {
        "svc-a": ["svc-missing"],
        "svc-b": [],
    }
    try:
        _resolve_start_order(manifest)
    except ValueError as exc:
        assert "unknown dependency" in str(exc)
    else:
        raise AssertionError("expected ValueError for unknown dependency")
