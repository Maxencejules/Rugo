"""Live native-driver evidence for the NVMe runtime lane."""

from __future__ import annotations

from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_native_driver_live_v1 as live_driver  # noqa: E402


def test_native_driver_live_v1_collects_real_nvme_probe_and_rw():
    payload = live_driver.collect_report()

    assert payload["status"] == "pass"
    assert payload["cpu"] == "qemu64,+x2apic"
    assert payload["disk_device"].startswith("nvme,")
    assert all(row["present"] for row in payload["marker_rows"])

    summary = payload["nvme_summary"]
    assert summary["nsid"] == 1
    assert summary["lba_bytes"] == 512
    assert summary["block_count"] >= 1024
    assert summary["depth"] >= 1
    assert summary["irq_hits"] >= 1
