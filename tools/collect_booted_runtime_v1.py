#!/usr/bin/env python3
"""Collect boot-backed runtime capture for the default release image."""

from __future__ import annotations

import argparse
from pathlib import Path
from typing import List

import runtime_capture_common_v1 as runtime_capture


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--image", default=runtime_capture.DEFAULT_RELEASE_IMAGE_PATH)
    parser.add_argument("--kernel", default=runtime_capture.DEFAULT_KERNEL_PATH)
    parser.add_argument(
        "--panic-image",
        default=runtime_capture.DEFAULT_PANIC_IMAGE_PATH,
    )
    parser.add_argument("--machine", default=runtime_capture.DEFAULT_MACHINE)
    parser.add_argument(
        "--timeout-seconds",
        type=float,
        default=runtime_capture.DEFAULT_TIMEOUT_SECONDS,
    )
    parser.add_argument(
        "--fixture",
        action="store_true",
        help="emit the deterministic built-in boot fixture instead of booting QEMU",
    )
    parser.add_argument("--out", default="out/booted-runtime-v1.json")
    return parser


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)
    if args.timeout_seconds <= 0:
        print("error: --timeout-seconds must be > 0")
        return 2

    if args.fixture:
        report = runtime_capture.build_fixture_capture(
            image_path=args.image,
            kernel_path=args.kernel,
            panic_image_path=args.panic_image,
        )
    else:
        try:
            report = runtime_capture.collect_booted_runtime(
                image_path=args.image,
                kernel_path=args.kernel,
                panic_image_path=args.panic_image,
                machine=args.machine,
                timeout_seconds=args.timeout_seconds,
            )
        except (FileNotFoundError, RuntimeError, TimeoutError) as exc:
            print(f"error: {exc}")
            return 1

    out_path = Path(args.out)
    runtime_capture.write_json(out_path, report)
    print(f"booted-runtime-capture: {out_path}")
    print(f"capture_mode: {report['capture_mode']}")
    print(f"image_path: {report['image_path']}")
    print(f"boots: {len(report['boots'])}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
