#!/usr/bin/env python3
"""Collect live QEMU evidence for the native NVMe driver lane."""

from __future__ import annotations

import argparse
import re
import time
import uuid
from pathlib import Path
from typing import Dict, List

import runtime_capture_common_v1 as runtime_capture


SCHEMA = "rugo.native_driver_live.v1"
DEFAULT_IMAGE_PATH = "out/os-blk-native.iso"
DEFAULT_KERNEL_PATH = "out/kernel-blk-native.elf"
DEFAULT_MACHINE = "q35"
DEFAULT_CPU = "qemu64,+x2apic"
DEFAULT_DISK_DEVICE = "nvme,drive=disk0,serial=nvme0,logical_block_size=512"
REQUIRED_MARKERS = [
    "IRQ: vector bound vec=64",
    "BAR: map ok bar=0 bytes=65536",
    "DRV: bind driver=nvme",
    "FW: allow signed driver=nvme",
    "NVME: ready",
    "NVME: identify ok",
    "NVME: io queue ok",
    "BLK: rw ok",
]

IDENTIFY_RE = re.compile(r"NVME: identify ok nsid=(\d+) lba=(\d+) blocks=(\d+)")
IO_RE = re.compile(r"NVME: io queue ok depth=(\d+) irq_hits=(\d+)")


def _marker_rows(lines: List[Dict[str, object]]) -> List[Dict[str, object]]:
    rows: List[Dict[str, object]] = []
    for marker in REQUIRED_MARKERS:
        ts_ms = None
        for entry in lines:
            if marker in str(entry.get("line", "")):
                ts_ms = round(float(entry.get("ts_ms", 0.0)), 3)
                break
        rows.append(
            {
                "marker": marker,
                "present": ts_ms is not None,
                "ts_ms": ts_ms,
            }
        )
    return rows


def _extract_summary(lines: List[Dict[str, object]]) -> Dict[str, int]:
    identify: Dict[str, int] = {}
    io_queue: Dict[str, int] = {}
    for entry in lines:
        line = str(entry.get("line", ""))
        match = IDENTIFY_RE.search(line)
        if match:
            identify = {
                "nsid": int(match.group(1)),
                "lba_bytes": int(match.group(2)),
                "block_count": int(match.group(3)),
            }
        match = IO_RE.search(line)
        if match:
            io_queue = {
                "depth": int(match.group(1)),
                "irq_hits": int(match.group(2)),
            }
    return {**identify, **io_queue}


def collect_report(
    *,
    image_path: str = DEFAULT_IMAGE_PATH,
    kernel_path: str = DEFAULT_KERNEL_PATH,
    machine: str = DEFAULT_MACHINE,
    cpu: str = DEFAULT_CPU,
    timeout_seconds: float = runtime_capture.DEFAULT_TIMEOUT_SECONDS,
    disk_device: str = DEFAULT_DISK_DEVICE,
) -> Dict[str, object]:
    image = Path(image_path)
    kernel = Path(kernel_path)

    temp_root = image.resolve().parent
    temp_root.mkdir(parents=True, exist_ok=True)
    disk_path = temp_root / f"rugo-native-driver-live-{uuid.uuid4().hex}.img"
    try:
        exit_code, lines = runtime_capture.qemu_capture_lines(
            image_path=image,
            timeout_seconds=timeout_seconds,
            machine=machine,
            cpu=cpu,
            disk_path=disk_path,
            disk_device=disk_device,
            with_net=False,
        )
    finally:
        if disk_path.is_file():
            disk_path.unlink()

    markers = _marker_rows(lines)
    summary = _extract_summary(lines)
    status = (
        "pass"
        if exit_code in runtime_capture.QEMU_SUCCESS_EXIT_CODES
        and all(row["present"] for row in markers)
        else "fail"
    )
    return {
        "schema": SCHEMA,
        "created_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "image_path": runtime_capture.posix_path(image),
        "kernel_path": runtime_capture.posix_path(kernel),
        "image_digest": runtime_capture.maybe_sha256_file(image, "native-driver-image"),
        "kernel_digest": runtime_capture.maybe_sha256_file(kernel, "native-driver-kernel"),
        "machine": machine,
        "cpu": cpu,
        "disk_device": disk_device,
        "exit_code": exit_code,
        "required_markers": REQUIRED_MARKERS,
        "marker_rows": markers,
        "nvme_summary": summary,
        "serial_lines": lines,
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
    parser.add_argument("--out", default="out/native-driver-live-v1.json")
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
        )
    except (FileNotFoundError, RuntimeError, TimeoutError) as exc:
        print(f"error: {exc}")
        return 1

    out_path = Path(args.out)
    runtime_capture.write_json(out_path, payload)
    print(f"native-driver-live: {out_path}")
    print(f"status: {payload['status']}")
    print(f"exit_code: {payload['exit_code']}")
    return 0 if payload["status"] == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
