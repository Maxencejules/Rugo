#!/usr/bin/env python3
"""Generate the shared X4 desktop profile runtime-backed report."""

from __future__ import annotations

import argparse
from pathlib import Path
from typing import List

import runtime_capture_common_v1 as runtime_capture
import x4_desktop_runtime_common_v1 as x4_runtime


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--seed", type=int, default=x4_runtime.DEFAULT_SEED)
    parser.add_argument(
        "--runtime-capture",
        help="existing desktop-profile runtime capture to use instead of booting QEMU",
    )
    parser.add_argument(
        "--runtime-capture-out",
        default=str(x4_runtime.DEFAULT_RUNTIME_CAPTURE_PATH),
        help="path to write the collected or fixture runtime capture",
    )
    parser.add_argument(
        "--image",
        default=str(x4_runtime.DEFAULT_RELEASE_IMAGE_PATH),
        help="desktop-profile boot image used for live runtime capture",
    )
    parser.add_argument(
        "--kernel",
        default=str(x4_runtime.DEFAULT_KERNEL_PATH),
        help="desktop-profile kernel image used for live runtime capture",
    )
    parser.add_argument(
        "--panic-image",
        default=str(x4_runtime.DEFAULT_PANIC_IMAGE_PATH),
        help="panic image path recorded in the capture provenance",
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
        help="emit the built-in desktop-profile fixture instead of booting QEMU",
    )
    parser.add_argument(
        "--inject-failure",
        action="append",
        default=[],
        help="force an X4 runtime check to fail by check_id",
    )
    parser.add_argument(
        "--emit-supporting-reports",
        action="store_true",
        help="write supporting reports into the selected output directory",
    )
    parser.add_argument(
        "--supporting-dir",
        default="out",
        help="directory for emitted supporting reports",
    )
    parser.add_argument("--out", default="out/desktop-profile-runtime-v1.json")
    return parser


def _load_or_collect_capture(args: argparse.Namespace) -> dict:
    if args.fixture:
        return x4_runtime.build_fixture_capture(
            image_path=args.image,
            kernel_path=args.kernel,
            panic_image_path=args.panic_image,
        )
    if args.runtime_capture:
        path = Path(args.runtime_capture)
        if not path.is_file():
            raise FileNotFoundError(f"runtime capture not found: {path}")
        return runtime_capture.read_json(path)
    return x4_runtime.collect_runtime_capture(
        image_path=args.image,
        kernel_path=args.kernel,
        panic_image_path=args.panic_image,
        machine=args.machine,
        timeout_seconds=args.timeout_seconds,
    )


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)
    if args.timeout_seconds <= 0:
        print("error: --timeout-seconds must be > 0")
        return 2
    try:
        injected_failures = x4_runtime.normalize_failures(args.inject_failure)
    except ValueError as exc:
        print(f"error: {exc}")
        return 2

    try:
        capture = _load_or_collect_capture(args)
    except (FileNotFoundError, RuntimeError, TimeoutError, KeyError, ValueError) as exc:
        print(f"error: {exc}")
        return 1

    runtime_capture_path = Path(args.runtime_capture_out)
    runtime_capture.write_json(runtime_capture_path, capture)

    reports = x4_runtime.collect_source_reports(seed=args.seed)
    if args.emit_supporting_reports:
        x4_runtime.write_supporting_reports(reports, base_dir=args.supporting_dir)

    report = x4_runtime.build_report(
        seed=args.seed,
        capture=capture,
        reports=reports,
        injected_failures=injected_failures,
    )

    out_path = Path(args.out)
    runtime_capture.write_json(out_path, report)
    print(f"desktop-profile-runtime-report: {out_path}")
    print(f"runtime-capture: {runtime_capture_path}")
    print(f"capture_mode: {capture['capture_mode']}")
    print(f"total_failures: {report['total_failures']}")
    print(f"gate_pass: {report['gate_pass']}")
    return 0 if report["gate_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
