"""M29 PR-2: deterministic diagnostic snapshot v2 checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import collect_booted_runtime_v1 as capture_tool  # noqa: E402
import collect_diagnostic_snapshot_v2 as diag_tool  # noqa: E402
import collect_trace_bundle_v2 as trace_tool  # noqa: E402


def _strip_timestamp(payload: dict) -> dict:
    stable = dict(payload)
    stable.pop("created_utc", None)
    return stable


def test_diag_snapshot_v2_is_seed_deterministic():
    capture = capture_tool.runtime_capture.build_fixture_capture()
    first = diag_tool.collect_snapshot(capture)
    second = diag_tool.collect_snapshot(capture)
    assert _strip_timestamp(first) == _strip_timestamp(second)


def test_diag_snapshot_v2_schema_and_trace_link(tmp_path: Path):
    capture_out = tmp_path / "booted-runtime-v1.json"
    trace_out = tmp_path / "trace-bundle-v2.json"
    diag_out = tmp_path / "diagnostic-snapshot-v2.json"

    assert capture_tool.main(["--fixture", "--out", str(capture_out)]) == 0
    assert trace_tool.main(["--runtime-capture", str(capture_out), "--out", str(trace_out)]) == 0
    rc = diag_tool.main(
        [
            "--runtime-capture",
            str(capture_out),
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
    assert data["release_image_path"] == "out/os-go.iso"
    assert data["trace_reference"]["attached"] is True
    assert data["trace_reference"]["schema"] == "rugo.trace_bundle.v2"
    assert data["unhealthy_checks"] == 0
    assert data["gate_pass"] is True


def test_diag_snapshot_v2_detects_unhealthy_check(tmp_path: Path):
    capture_out = tmp_path / "booted-runtime-v1.json"
    out = tmp_path / "diagnostic-snapshot-v2.json"
    assert capture_tool.main(["--fixture", "--out", str(capture_out)]) == 0
    rc = diag_tool.main(
        [
            "--runtime-capture",
            str(capture_out),
            "--inject-unhealthy-check",
            "network_stack",
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
