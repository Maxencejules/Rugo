"""M29 PR-2: deterministic trace bundle v2 checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import collect_trace_bundle_v2 as trace_tool  # noqa: E402


def _strip_timestamp(payload: dict) -> dict:
    stable = dict(payload)
    stable.pop("created_utc", None)
    return stable


def test_trace_bundle_v2_is_seed_deterministic():
    first = trace_tool.collect_trace_bundle(seed=20260309, window_seconds=300)
    second = trace_tool.collect_trace_bundle(seed=20260309, window_seconds=300)
    assert _strip_timestamp(first) == _strip_timestamp(second)


def test_trace_bundle_v2_schema_and_gate_pass(tmp_path: Path):
    out = tmp_path / "trace-bundle-v2.json"
    rc = trace_tool.main(
        [
            "--seed",
            "20260309",
            "--window-seconds",
            "300",
            "--out",
            str(out),
        ]
    )
    assert rc == 0
    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.trace_bundle.v2"
    assert data["contract_id"] == "rugo.observability_contract.v2"
    assert data["totals"]["total_services"] >= 4
    assert data["totals"]["total_errors"] == 0
    assert data["totals"]["total_dropped_spans"] == 0
    assert data["gate_pass"] is True


def test_trace_bundle_v2_detects_injected_error(tmp_path: Path):
    out = tmp_path / "trace-bundle-v2.json"
    rc = trace_tool.main(
        [
            "--seed",
            "20260309",
            "--inject-error",
            "svcman",
            "--max-errors",
            "0",
            "--out",
            str(out),
        ]
    )
    assert rc == 1
    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.trace_bundle.v2"
    assert data["totals"]["total_errors"] >= 1
    assert data["gate_pass"] is False
