"""M23 acceptance: driver lifecycle v3 contract and diagnostics evidence."""

from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import collect_hw_diagnostics_v3 as diagnostics  # noqa: E402


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def _strip_timestamp(payload: dict) -> dict:
    stable = dict(payload)
    stable.pop("created_utc", None)
    return stable


def test_driver_lifecycle_contract_v3_has_required_states():
    contract = _read("docs/hw/driver_lifecycle_contract_v3.md")
    for token in [
        "Schema identifier: `rugo.driver_lifecycle_report.v3`",
        "`probe_missing`",
        "`probe_found`",
        "`init_ready`",
        "`runtime_ok`",
        "`suspend_prepare`",
        "`resume_ok`",
        "`hotplug_add`",
        "`hotplug_remove`",
        "`error_recoverable`",
        "`error_fatal`",
    ]:
        assert token in contract


def test_driver_lifecycle_report_is_seed_deterministic():
    first = diagnostics.run_diagnostics(seed=20260306, suspend_cycles=12, hotplug_events=8)
    second = diagnostics.run_diagnostics(seed=20260306, suspend_cycles=12, hotplug_events=8)
    assert _strip_timestamp(first) == _strip_timestamp(second)


def test_driver_lifecycle_report_contains_required_runtime_paths():
    report = diagnostics.run_diagnostics(seed=20260306, suspend_cycles=12, hotplug_events=8)
    assert report["schema"] == "rugo.hw_matrix_evidence.v3"
    assert report["driver_contract_id"] == "rugo.driver_lifecycle_report.v3"
    assert report["gate_pass"] is True

    required_states = {
        "probe_found",
        "init_ready",
        "runtime_ok",
        "suspend_prepare",
        "resume_ok",
        "hotplug_add",
        "hotplug_remove",
    }
    for entry in report["driver_lifecycle"]:
        assert required_states.issubset(set(entry["states_observed"]))
        assert entry["init_failures"] == 0
        assert entry["runtime_errors"] == 0
        assert entry["status"] == "pass"
