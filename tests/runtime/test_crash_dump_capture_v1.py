"""M29 PR-2: deterministic crash dump capture checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import collect_crash_dump_v1 as collector  # noqa: E402


def _strip_timestamp(payload: dict) -> dict:
    stable = dict(payload)
    stable.pop("created_utc", None)
    return stable


def test_crash_dump_build_dump_is_deterministic_except_timestamp():
    first = collector.build_dump(
        panic_code=13,
        panic_reason="kernel_panic",
        release_image_digest="digest",
        panic_trace_digest="trace",
    )
    second = collector.build_dump(
        panic_code=13,
        panic_reason="kernel_panic",
        release_image_digest="digest",
        panic_trace_digest="trace",
    )
    assert _strip_timestamp(first) == _strip_timestamp(second)


def test_crash_dump_capture_v1_schema(tmp_path: Path):
    out = tmp_path / "crash-dump-v1.json"
    rc = collector.main(
        [
            "--fixture",
            "--panic-code",
            "0x4d",
            "--panic-reason",
            "test_fault",
            "--kernel-build-id",
            "rugo-kernel-2026.03.09",
            "--out",
            str(out),
        ]
    )
    assert rc == 0
    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.crash_dump.v1"
    assert data["contract_id"] == "rugo.crash_dump_contract.v1"
    assert data["triage_playbook_id"] == "rugo.postmortem_triage_playbook.v1"
    assert data["symbol_map_id"] == "rugo.kernel_symbol_map.v1"
    assert data["panic_code"] == 77
    assert data["panic_reason"] == "test_fault"
    assert data["release_channel"] == "stable"
    assert data["kernel_build_id"] == "rugo-kernel-2026.03.09"
    assert data["runtime_provenance"]["release_image_path"] == "out/os-go.iso"
    assert data["runtime_provenance"]["panic_image_path"] == "out/os-panic.iso"
    assert data["runtime_provenance"]["panic_trace_id"]
    assert data["dump_id"].startswith("dump-")
    assert set(["rip", "rsp", "rbp"]).issubset(set(data["registers"].keys()))
    assert len(data["stack_frames"]) >= 1
