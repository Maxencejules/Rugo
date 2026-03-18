#!/usr/bin/env python3
"""Collect crash dump artifact with boot-backed panic provenance."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
from typing import Dict, List

import runtime_capture_common_v1 as runtime_capture


SCHEMA = "rugo.crash_dump.v1"
CONTRACT_ID = "rugo.crash_dump_contract.v1"
PLAYBOOK_ID = "rugo.postmortem_triage_playbook.v1"
SYMBOL_MAP_ID = "rugo.kernel_symbol_map.v1"


def _dump_id(panic_code: int, kernel_build_id: str, panic_trace_id: str) -> str:
    digest = hashlib.sha256(
        f"{panic_code}|{kernel_build_id}|{panic_trace_id}".encode("utf-8")
    ).hexdigest()
    return f"dump-{digest[:12]}"


def build_dump(
    panic_code: int,
    panic_reason: str = "kernel_panic",
    kernel_build_id: str = "rugo-kernel-booted-default",
    *,
    release_channel: str = "stable",
    release_image_path: str = runtime_capture.DEFAULT_RELEASE_IMAGE_PATH,
    release_image_digest: str = "",
    panic_image_path: str = runtime_capture.DEFAULT_PANIC_IMAGE_PATH,
    panic_image_digest: str = "",
    panic_boot_id: str = "",
    panic_trace_id: str = "trace-panic-fixture",
    panic_trace_digest: str = "",
    serial_digest: str = "",
    serial_lines: List[Dict[str, object]] | None = None,
    capture_mode: str = "fixture",
) -> Dict[str, object]:
    excerpt = [] if serial_lines is None else [str(entry.get("line", "")) for entry in serial_lines[:8]]
    stable_payload = {
        "schema": SCHEMA,
        "panic_code": panic_code,
        "kernel_build_id": kernel_build_id,
        "panic_trace_id": panic_trace_id,
        "serial_digest": serial_digest,
        "release_image_digest": release_image_digest,
    }
    digest = runtime_capture.stable_digest(stable_payload)
    return {
        "schema": SCHEMA,
        "contract_id": CONTRACT_ID,
        "triage_playbook_id": PLAYBOOK_ID,
        "symbol_map_id": SYMBOL_MAP_ID,
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "dump_id": _dump_id(
            panic_code=panic_code,
            kernel_build_id=kernel_build_id,
            panic_trace_id=panic_trace_id,
        ),
        "panic_code": panic_code,
        "panic_reason": panic_reason,
        "release_channel": release_channel,
        "kernel_build_id": kernel_build_id,
        "registers": {
            "rip": "0xffffffff80001000",
            "rsp": "0xffffffff8100ff00",
            "rbp": "0xffffffff8100ff30",
        },
        "stack_frames": [
            {"ip": "0xffffffff80001000", "offset": 0},
            {"ip": "0xffffffff80002000", "offset": 24},
            {"ip": "0xffffffff80003000", "offset": 56},
        ],
        "runtime_provenance": {
            "capture_mode": capture_mode,
            "release_image_path": release_image_path,
            "release_image_digest": release_image_digest,
            "panic_image_path": panic_image_path,
            "panic_image_digest": panic_image_digest,
            "panic_boot_id": panic_boot_id,
            "panic_trace_id": panic_trace_id,
            "panic_trace_digest": panic_trace_digest,
            "serial_digest": serial_digest,
            "serial_excerpt": excerpt,
        },
        "digest": digest,
    }


def _kernel_build_id(capture: Dict[str, object], override: str) -> str:
    if override:
        return override
    kernel_digest = str(capture.get("kernel_digest", "unknown"))
    return f"rugo-kernel-{kernel_digest[:12]}"


def _build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--release-image", default=runtime_capture.DEFAULT_RELEASE_IMAGE_PATH)
    p.add_argument("--kernel", default=runtime_capture.DEFAULT_KERNEL_PATH)
    p.add_argument("--panic-image", default=runtime_capture.DEFAULT_PANIC_IMAGE_PATH)
    p.add_argument("--panic-code", type=lambda value: int(value, 0), default=None)
    p.add_argument("--panic-reason", default="kernel_panic")
    p.add_argument("--kernel-build-id", default="")
    p.add_argument(
        "--fixture",
        action="store_true",
        help="use the deterministic built-in panic fixture instead of booting QEMU",
    )
    p.add_argument("--out", default="out/crash-dump-v1.json")
    return p


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)
    if args.fixture:
        capture = runtime_capture.build_panic_fixture(
            release_image_path=args.release_image,
            kernel_path=args.kernel,
            panic_image_path=args.panic_image,
        )
    else:
        try:
            capture = runtime_capture.collect_panic_capture(
                release_image_path=args.release_image,
                kernel_path=args.kernel,
                panic_image_path=args.panic_image,
            )
        except (FileNotFoundError, RuntimeError, TimeoutError) as exc:
            print(f"error: {exc}")
            return 1

    panic_code = int(args.panic_code) if args.panic_code is not None else int(capture["panic_code"])
    dump = build_dump(
        panic_code=panic_code,
        panic_reason=args.panic_reason,
        kernel_build_id=_kernel_build_id(capture, args.kernel_build_id),
        release_image_path=str(capture["release_image_path"]),
        release_image_digest=str(capture["release_image_digest"]),
        panic_image_path=str(capture["panic_image_path"]),
        panic_image_digest=str(capture["panic_image_digest"]),
        panic_boot_id=str(capture["panic_boot_id"]),
        panic_trace_id=str(capture["panic_trace_id"]),
        panic_trace_digest=str(capture["panic_trace_digest"]),
        serial_digest=str(capture["serial_digest"]),
        serial_lines=list(capture.get("serial_lines", [])),
        capture_mode=str(capture["capture_mode"]),
    )

    out_path = Path(args.out)
    runtime_capture.write_json(out_path, dump)
    print(f"crash-dump: {out_path}")
    print(f"panic_code: {dump['panic_code']}")
    print(f"dump_id: {dump['dump_id']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
