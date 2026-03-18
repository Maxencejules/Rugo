"""T2 foundation: booted runtime capture contract checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import collect_booted_runtime_v1 as collector  # noqa: E402
import runtime_capture_common_v1 as runtime_capture  # noqa: E402


def test_booted_runtime_capture_fixture_is_deterministic():
    first = runtime_capture.build_fixture_capture()
    second = runtime_capture.build_fixture_capture()
    assert first == second


def test_booted_runtime_capture_v1_schema_and_default_image_binding(tmp_path: Path):
    out = tmp_path / "booted-runtime-v1.json"
    rc = collector.main(["--fixture", "--out", str(out)])
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.booted_runtime_capture.v1"
    assert data["capture_mode"] == "fixture"
    assert data["image_path"] == "out/os-go.iso"
    assert data["kernel_path"] == "out/kernel-go.elf"
    assert data["panic_image_path"] == "out/os-panic.iso"
    assert data["execution_lane"] == "qemu"
    assert data["trace_id"].startswith("trace-qemu-")
    assert data["trace_digest"]
    assert data["boot_profiles"] == ["cold_boot", "replay_boot"]
    assert len(data["boots"]) == 2

    first_boot = data["boots"][0]
    second_boot = data["boots"][1]
    assert first_boot["boot_profile"] == "cold_boot"
    assert second_boot["boot_profile"] == "replay_boot"
    assert first_boot["serial_line_count"] > 0
    assert second_boot["serial_line_count"] > 0
    assert first_boot["process_snapshots"]
    assert first_boot["task_snapshots"]
    assert any("STORC4: journal staged" in row["line"] for row in first_boot["serial_lines"])
    assert any("RECOV: replay ok" in row["line"] for row in second_boot["serial_lines"])
    assert any("NETC4: reply ok" in row["line"] for row in second_boot["serial_lines"])
