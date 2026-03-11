#!/usr/bin/env python3
"""Capture a deterministic display frame for M48."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import struct
from typing import List, Sequence
import zlib

import run_display_runtime_v1 as runtime


SCHEMA = runtime.FRAME_CAPTURE_SCHEMA
DEFAULT_SEED = runtime.DEFAULT_SEED
CAPTURE_SIZE = (320, 180)


def _noise(seed: int, key: str) -> int:
    digest = hashlib.sha256(f"{seed}|{key}".encode("utf-8")).hexdigest()
    return int(digest[:8], 16)


def _png_chunk(kind: bytes, payload: bytes) -> bytes:
    return (
        struct.pack(">I", len(payload))
        + kind
        + payload
        + struct.pack(">I", zlib.crc32(kind + payload) & 0xFFFFFFFF)
    )


def _encode_png(width: int, height: int, rows: Sequence[bytes]) -> bytes:
    raw = b"".join(b"\x00" + row for row in rows)
    header = struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0)
    return (
        b"\x89PNG\r\n\x1a\n"
        + _png_chunk(b"IHDR", header)
        + _png_chunk(b"IDAT", zlib.compress(raw, level=9))
        + _png_chunk(b"IEND", b"")
    )


def _mix(
    base: tuple[int, int, int],
    accent: tuple[int, int, int],
    weight: float,
) -> tuple[int, int, int]:
    return tuple(
        int(round(base[idx] * (1.0 - weight) + accent[idx] * weight))
        for idx in range(3)
    )


def _build_rows(seed: int, active_path: str) -> tuple[int, int, List[bytes]]:
    width, height = CAPTURE_SIZE
    if active_path == runtime.PRIMARY_DISPLAY_CLASS:
        base = (12, 34, 54)
        accent = (64, 176, 192)
        stripe = (220, 244, 248)
    else:
        base = (54, 46, 32)
        accent = (212, 152, 74)
        stripe = (248, 238, 214)

    rows: List[bytes] = []
    banner_height = 28
    scanout_top = 42
    scanout_bottom = height - 28
    buffer_top = height - 22
    accent_shift = _noise(seed, f"{active_path}|accent_shift") % 7
    focus_x = 40 + (_noise(seed, f"{active_path}|focus_x") % 220)
    focus_y = 56 + (_noise(seed, f"{active_path}|focus_y") % 70)

    for y in range(height):
        row = bytearray()
        for x in range(width):
            if y < banner_height:
                weight = min(1.0, (x + accent_shift * 5) / float(width + 20))
                color = _mix(accent, stripe, weight * 0.35)
            elif buffer_top <= y:
                slot = min(3, x // 80)
                slot_weight = 0.2 + (slot * 0.18)
                color = _mix(base, accent, slot_weight)
            elif scanout_top <= y < scanout_bottom and 28 <= x < width - 28:
                x_weight = x / float(width - 1)
                y_weight = (y - scanout_top) / float(scanout_bottom - scanout_top - 1)
                blend = min(1.0, 0.22 + (x_weight * 0.45) + (y_weight * 0.18))
                color = _mix(base, accent, blend)
                if ((x + y + accent_shift) % 24) == 0:
                    color = stripe
                if abs(x - focus_x) <= 10 and abs(y - focus_y) <= 10:
                    color = (248, 248, 248)
            else:
                weight = ((x // 12) % 3) * 0.08
                color = _mix(base, accent, weight)

            row.extend(color)
        rows.append(bytes(row))

    return width, height, rows


def capture_frame(
    seed: int,
    out_png: Path,
    runtime_failures: set[str] | None = None,
    force_fallback: bool = False,
) -> dict:
    runtime_report = runtime.run_display_runtime(
        seed=seed,
        injected_failures=runtime_failures,
        max_failures=0,
        force_fallback=force_fallback,
    )

    width, height, rows = _build_rows(seed, runtime_report["active_runtime_path"])
    png_bytes = _encode_png(width, height, rows)

    out_png.parent.mkdir(parents=True, exist_ok=True)
    out_png.write_bytes(png_bytes)

    png_sha256 = hashlib.sha256(png_bytes).hexdigest()
    manifest = {
        "schema": SCHEMA,
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "runtime_schema": runtime_report["schema"],
        "runtime_digest": runtime_report["digest"],
        "runtime_gate_pass": runtime_report["gate_pass"],
        "active_runtime_path": runtime_report["active_runtime_path"],
        "active_runtime_driver": runtime_report["active_runtime_driver"],
        "buffer_contract_id": runtime_report["buffer_contract_id"],
        "width": width,
        "height": height,
        "pixel_format": "rgb24",
        "png_sha256": png_sha256,
        "capture_pass": runtime_report["gate_pass"] and runtime_report["capture"]["checks_pass"],
        "artifact_refs": {
            "png_path": str(out_png),
            "runtime_report": runtime_report["artifact_refs"]["runtime_report"],
            "manifest_path": str(out_png.with_suffix(".json")),
        },
    }
    manifest["gate_pass"] = manifest["capture_pass"]
    return manifest


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--seed", type=int, default=DEFAULT_SEED)
    parser.add_argument(
        "--inject-runtime-failure",
        action="append",
        default=[],
        help="force a display runtime check to fail before capture",
    )
    parser.add_argument(
        "--force-fallback",
        action="store_true",
        help="capture the efifb fallback path",
    )
    parser.add_argument("--out", default="out/display-frame-v1.png")
    return parser


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)
    try:
        runtime_failures = runtime.normalize_failures(args.inject_runtime_failure)
    except ValueError as exc:
        print(f"error: {exc}")
        return 2

    out_png = Path(args.out)
    manifest = capture_frame(
        seed=args.seed,
        out_png=out_png,
        runtime_failures=runtime_failures,
        force_fallback=args.force_fallback,
    )
    manifest_path = out_png.with_suffix(".json")
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")

    print(f"display-frame: {out_png}")
    print(f"display-frame-manifest: {manifest_path}")
    print(f"active_runtime_path: {manifest['active_runtime_path']}")
    print(f"gate_pass: {manifest['gate_pass']}")
    return 0 if manifest["gate_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
