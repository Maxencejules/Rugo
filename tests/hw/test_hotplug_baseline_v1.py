"""M23 acceptance: hotplug baseline diagnostics contract."""

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import collect_hw_diagnostics_v3 as diagnostics  # noqa: E402


def test_hotplug_baseline_v1(tmp_path: Path):
    out = tmp_path / "hw-diagnostics-v3.json"
    rc = diagnostics.main(
        ["--seed", "20260306", "--hotplug-events", "16", "--out", str(out)]
    )
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    hotplug = data["hotplug_baseline"]

    assert data["schema"] == "rugo.hw_matrix_evidence.v3"
    assert hotplug["events_target"] == 16
    assert hotplug["events_completed"] == 16
    assert hotplug["failures"] == 0
    assert hotplug["max_settle_ms"] <= hotplug["settle_budget_ms"]
    assert hotplug["status"] == "pass"
