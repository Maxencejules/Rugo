"""M23 acceptance: suspend/resume baseline diagnostics contract."""

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import collect_hw_diagnostics_v3 as diagnostics  # noqa: E402


def test_suspend_resume_baseline_v1(tmp_path: Path):
    out = tmp_path / "hw-diagnostics-v3.json"
    rc = diagnostics.main(
        ["--seed", "20260306", "--suspend-cycles", "24", "--out", str(out)]
    )
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    suspend = data["suspend_resume"]

    assert data["schema"] == "rugo.hw_matrix_evidence.v3"
    assert suspend["cycles_target"] == 24
    assert suspend["cycles_completed"] == 24
    assert suspend["suspend_failures"] == 0
    assert suspend["resume_timeouts"] == 0
    assert suspend["max_resume_latency_ms"] <= suspend["resume_latency_budget_ms"]
    assert suspend["status"] == "pass"
