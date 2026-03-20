#!/usr/bin/env python3
"""Generate the product-level alpha qualification report."""

from __future__ import annotations

import argparse
from pathlib import Path
from typing import List

import product_alpha_common_v1 as alpha
import runtime_capture_common_v1 as runtime_capture


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--seed", type=int, default=alpha.DEFAULT_SEED)
    parser.add_argument("--channel", default=alpha.DEFAULT_CHANNEL)
    parser.add_argument("--version", default=alpha.DEFAULT_VERSION)
    parser.add_argument("--build-sequence", type=int, default=alpha.DEFAULT_BUILD_SEQUENCE)
    parser.add_argument("--image", default=str(alpha.DEFAULT_RELEASE_IMAGE_PATH))
    parser.add_argument("--kernel", default=str(alpha.DEFAULT_KERNEL_PATH))
    parser.add_argument("--panic-image", default=str(alpha.DEFAULT_PANIC_IMAGE_PATH))
    parser.add_argument("--machine", default=alpha.DEFAULT_MACHINE)
    parser.add_argument("--cpu", default=alpha.DEFAULT_CPU)
    parser.add_argument("--disk-device", default=alpha.DEFAULT_DISK_DEVICE)
    parser.add_argument("--net-device", default=alpha.DEFAULT_NET_DEVICE)
    parser.add_argument(
        "--timeout-seconds",
        type=float,
        default=runtime_capture.DEFAULT_TIMEOUT_SECONDS,
    )
    parser.add_argument(
        "--runtime-capture",
        default="",
        help="existing runtime capture to reuse instead of booting QEMU",
    )
    parser.add_argument(
        "--runtime-capture-out",
        default=str(alpha.DEFAULT_RUNTIME_CAPTURE_PATH),
        help="path to write the collected or fixture runtime capture",
    )
    parser.add_argument(
        "--artifact-dir",
        default=str(alpha.DEFAULT_ARTIFACT_DIR),
        help="directory for generated alpha artifacts",
    )
    parser.add_argument(
        "--supporting-dir",
        default=str(alpha.DEFAULT_SUPPORTING_DIR),
        help="directory for optional supporting reports",
    )
    parser.add_argument(
        "--emit-supporting-reports",
        action="store_true",
        help="write the underlying X3/X4 supporting reports into --supporting-dir",
    )
    parser.add_argument(
        "--fixture",
        action="store_true",
        help="use the deterministic built-in alpha fixture instead of booting QEMU",
    )
    parser.add_argument(
        "--inject-failure",
        action="append",
        default=[],
        help="force a high-level alpha check to fail by check_id",
    )
    parser.add_argument("--out", default="out/product-alpha-v1.json")
    return parser


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)
    if args.timeout_seconds <= 0:
        print("error: --timeout-seconds must be > 0")
        return 2
    if args.build_sequence <= 0:
        print("error: --build-sequence must be > 0")
        return 2

    try:
        injected_failures = alpha.normalize_failures(args.inject_failure)
    except ValueError as exc:
        print(f"error: {exc}")
        return 2

    artifact_dir = Path(args.artifact_dir)
    supporting_dir = Path(args.supporting_dir)
    paths = alpha.build_paths(
        artifact_dir=artifact_dir,
        supporting_dir=supporting_dir,
        report_path=Path(args.out),
        runtime_capture_path=Path(args.runtime_capture_out),
    )

    try:
        capture = alpha.load_runtime_capture(
            runtime_capture_path=args.runtime_capture,
            fixture=args.fixture,
            image_path=args.image,
            kernel_path=args.kernel,
            panic_image_path=args.panic_image,
            machine=args.machine,
            cpu=args.cpu,
            timeout_seconds=args.timeout_seconds,
            disk_device=args.disk_device,
            net_device=args.net_device,
        )
    except (FileNotFoundError, RuntimeError, TimeoutError, KeyError, ValueError) as exc:
        print(f"error: {exc}")
        return 1

    runtime_capture.write_json(paths.runtime_capture, capture)

    try:
        reports = alpha.collect_reports(
            seed=args.seed,
            capture=capture,
            image_path=args.image,
            kernel_path=args.kernel,
            panic_image_path=args.panic_image,
            fixture=args.fixture,
            channel=args.channel,
            version=args.version,
            build_sequence=args.build_sequence,
            emit_supporting_reports=args.emit_supporting_reports,
            paths=paths,
        )
    except (FileNotFoundError, RuntimeError, TimeoutError, KeyError, ValueError) as exc:
        print(f"error: {exc}")
        return 1

    report = alpha.build_report(
        seed=args.seed,
        capture=capture,
        reports=reports,
        image_path=args.image,
        kernel_path=args.kernel,
        panic_image_path=args.panic_image,
        paths=paths,
        injected_failures=injected_failures,
    )
    runtime_capture.write_json(paths.report, report)

    print(f"product-alpha-report: {paths.report}")
    print(f"runtime-capture: {paths.runtime_capture}")
    print(f"capture_mode: {capture['capture_mode']}")
    print(f"total_failures: {report['total_failures']}")
    print(f"gate_pass: {report['gate_pass']}")
    return 0 if report["gate_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
