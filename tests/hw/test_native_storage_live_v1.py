"""Live native-storage evidence for the NVMe durability lane."""

from __future__ import annotations

from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_native_storage_live_v1 as live_storage  # noqa: E402


def test_native_storage_live_v1_collects_replay_and_flush_markers():
    payload = live_storage.collect_report()

    assert payload["status"] == "pass"
    assert payload["cpu"] == "qemu64,+x2apic"
    assert payload["disk_device"].startswith("nvme,")
    assert payload["durability_bridge"]["fsync_device_class"] == "nvme"
    assert payload["durability_bridge"]["required_markers"] == [
        "BLK: fua ok",
        "BLK: flush ordered",
    ]
    assert all(row["present"] for row in payload["cold_boot_markers"])
    assert all(row["present"] for row in payload["replay_boot_markers"])
    assert len(payload["capture"]["boots"]) == 2
