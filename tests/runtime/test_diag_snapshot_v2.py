"""M29 PR-2: deterministic diagnostic snapshot v2 checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import collect_diagnostic_snapshot_v2 as diag_tool  # noqa: E402
import collect_trace_bundle_v2 as trace_tool  # noqa: E402


def _strip_timestamp(payload: dict) -> dict:
    stable = dict(payload)
    stable.pop("created_utc", None)
    return stable


def test_diag_snapshot_v2_is_seed_deterministic():
    first = diag_tool.collect_snapshot(seed=20260309)
    second = diag_tool.collect_snapshot(seed=20260309)
    assert _strip_timestamp(first) == _strip_timestamp(second)


def test_diag_snapshot_v2_schema_and_trace_link(tmp_path: Path):
    trace_out = tmp_path / "trace-bundle-v2.json"
    diag_out = tmp_path / "diagnostic-snapshot-v2.json"

    assert trace_tool.main(["--seed", "20260309", "--out", str(trace_out)]) == 0
    rc = diag_tool.main(
        [
            "--seed",
            "20260309",
            "--trace-bundle",
            str(trace_out),
            "--out",
            str(diag_out),
        ]
    )
    assert rc == 0

    data = json.loads(diag_out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.diagnostic_snapshot.v2"
    assert data["contract_id"] == "rugo.observability_contract.v2"
    assert data["trace_reference"]["attached"] is True
    assert data["trace_reference"]["schema"] == "rugo.trace_bundle.v2"
    assert data["unhealthy_checks"] == 0
    assert data["gate_pass"] is True


def test_diag_snapshot_v2_detects_unhealthy_check(tmp_path: Path):
    out = tmp_path / "diagnostic-snapshot-v2.json"
    rc = diag_tool.main(
        [
            "--seed",
            "20260309",
            "--inject-unhealthy-check",
            "network_service",
            "--max-unhealthy-checks",
            "0",
            "--out",
            str(out),
        ]
    )
    assert rc == 1
    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.diagnostic_snapshot.v2"
    assert data["unhealthy_checks"] >= 1
    assert data["gate_pass"] is False
