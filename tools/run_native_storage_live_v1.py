#!/usr/bin/env python3
"""Collect live QEMU evidence for the native NVMe storage durability lane."""

from __future__ import annotations

import argparse
import time
from pathlib import Path
from typing import Dict, List

import runtime_capture_common_v1 as runtime_capture


SCHEMA = "rugo.native_storage_live.v1"
DEFAULT_IMAGE_PATH = "out/os-go-native.iso"
DEFAULT_KERNEL_PATH = "out/kernel-go-native.elf"
DEFAULT_MACHINE = "q35"
DEFAULT_CPU = "qemu64,+x2apic"
DEFAULT_DISK_DEVICE = "nvme,drive=disk0,serial=nvme0,logical_block_size=512"
DEFAULT_NET_DEVICE = "virtio-net-pci,netdev=n0,disable-modern=on"

COLD_BOOT_MARKERS = [
    "STORC4: block ready driver=nvme",
    "NETC4: nic ready",
    "GOSH: diag ok",
    "STORC4: journal staged",
    "GOINIT: ready",
]

REPLAY_BOOT_MARKERS = [
    "STORC4: block ready driver=nvme",
    "RECOV: replay ok",
    "STORC4: state ok",
    "BLK: fua ok",
    "BLK: flush ordered",
    "STORC4: fsync ok",
    "GOINIT: ready",
]


def _boot_by_profile(capture: Dict[str, object], profile: str) -> Dict[str, object]:
    for boot in runtime_capture.iter_boots(capture):
        if boot.get("boot_profile") == profile:
            return boot
    raise KeyError(f"missing boot profile: {profile}")


def _marker_rows(boot: Dict[str, object], markers: List[str]) -> List[Dict[str, object]]:
    rows: List[Dict[str, object]] = []
    for marker in markers:
        rows.append(
            {
                "marker": marker,
                "present": runtime_capture.find_first_line_ts(boot, marker) is not None,
                "ts_ms": runtime_capture.find_first_line_ts(boot, marker),
            }
        )
    return rows


def collect_report(
    *,
    image_path: str = DEFAULT_IMAGE_PATH,
    kernel_path: str = DEFAULT_KERNEL_PATH,
    machine: str = DEFAULT_MACHINE,
    cpu: str = DEFAULT_CPU,
    timeout_seconds: float = runtime_capture.DEFAULT_TIMEOUT_SECONDS,
    disk_device: str = DEFAULT_DISK_DEVICE,
    net_device: str = DEFAULT_NET_DEVICE,
) -> Dict[str, object]:
    capture = runtime_capture.collect_booted_runtime(
        image_path=image_path,
        kernel_path=kernel_path,
        machine=machine,
        cpu=cpu,
        timeout_seconds=timeout_seconds,
        disk_device=disk_device,
        net_device=net_device,
    )
    cold_boot = _boot_by_profile(capture, "cold_boot")
    replay_boot = _boot_by_profile(capture, "replay_boot")
    cold_rows = _marker_rows(cold_boot, COLD_BOOT_MARKERS)
    replay_rows = _marker_rows(replay_boot, REPLAY_BOOT_MARKERS)
    status = (
        "pass"
        if all(row["present"] for row in cold_rows)
        and all(row["present"] for row in replay_rows)
        else "fail"
    )
    return {
        "schema": SCHEMA,
        "created_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "image_path": runtime_capture.posix_path(Path(image_path)),
        "kernel_path": runtime_capture.posix_path(Path(kernel_path)),
        "machine": machine,
        "cpu": cpu,
        "disk_device": disk_device,
        "net_device": net_device,
        "cold_boot_markers": cold_rows,
        "replay_boot_markers": replay_rows,
        "durability_bridge": {
            "fsync_device_class": "nvme",
            "required_markers": ["BLK: fua ok", "BLK: flush ordered"],
            "recovery_marker": "RECOV: replay ok",
            "block_ready_marker": "STORC4: block ready driver=nvme",
        },
        "capture": capture,
        "status": status,
    }


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--image", default=DEFAULT_IMAGE_PATH)
    parser.add_argument("--kernel", default=DEFAULT_KERNEL_PATH)
    parser.add_argument("--machine", default=DEFAULT_MACHINE)
    parser.add_argument("--cpu", default=DEFAULT_CPU)
    parser.add_argument(
        "--timeout-seconds",
        type=float,
        default=runtime_capture.DEFAULT_TIMEOUT_SECONDS,
    )
    parser.add_argument("--disk-device", default=DEFAULT_DISK_DEVICE)
    parser.add_argument("--net-device", default=DEFAULT_NET_DEVICE)
    parser.add_argument("--out", default="out/native-storage-live-v1.json")
    return parser


def main() -> int:
    args = _build_parser().parse_args()
    if args.timeout_seconds <= 0:
        print("error: --timeout-seconds must be > 0")
        return 2
    try:
        payload = collect_report(
            image_path=args.image,
            kernel_path=args.kernel,
            machine=args.machine,
            cpu=args.cpu,
            timeout_seconds=args.timeout_seconds,
            disk_device=args.disk_device,
            net_device=args.net_device,
        )
    except (FileNotFoundError, RuntimeError, TimeoutError, KeyError) as exc:
        print(f"error: {exc}")
        return 1

    out_path = Path(args.out)
    runtime_capture.write_json(out_path, payload)
    print(f"native-storage-live: {out_path}")
    print(f"status: {payload['status']}")
    return 0 if payload["status"] == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
