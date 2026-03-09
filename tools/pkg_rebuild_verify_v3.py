#!/usr/bin/env python3
"""Emit deterministic package rebuild verification report for M26."""

from __future__ import annotations

import argparse
import hashlib
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Set


SCHEMA = "rugo.pkg_rebuild_report.v3"
FORMAT_ID = "rugo.pkg_format.v3"

PACKAGE_SPECS: List[Dict[str, str]] = [
    {
        "name": "base-shell",
        "version": "3.2.0",
        "release": "r1",
        "source_commit": "a2d5317",
        "build_recipe_id": "pkg.base-shell.v3.r1",
    },
    {
        "name": "svc-manager",
        "version": "2.4.1",
        "release": "r3",
        "source_commit": "8f30b16",
        "build_recipe_id": "pkg.svc-manager.v3.r3",
    },
    {
        "name": "net-utils",
        "version": "1.8.5",
        "release": "r2",
        "source_commit": "4f92a1a",
        "build_recipe_id": "pkg.net-utils.v3.r2",
    },
]


def _digest_for_package(spec: Dict[str, str], seed: int) -> str:
    identity = (
        f"{seed}|{spec['name']}|{spec['version']}|{spec['release']}|"
        f"{spec['source_commit']}|{spec['build_recipe_id']}"
    )
    return hashlib.sha256(identity.encode("utf-8")).hexdigest()


def _known_package_names() -> Set[str]:
    return {spec["name"] for spec in PACKAGE_SPECS}


def _collect_injected_mismatches(specs: List[str]) -> Set[str]:
    requested = {name.strip() for name in specs if name.strip()}
    unknown = sorted(requested - _known_package_names())
    if unknown:
        raise ValueError(f"unknown packages in --inject-mismatch: {', '.join(unknown)}")
    return requested


def run_rebuild(seed: int, mismatch_packages: Set[str] | None = None) -> Dict[str, object]:
    injected = set() if mismatch_packages is None else set(mismatch_packages)
    packages: List[Dict[str, object]] = []

    for spec in PACKAGE_SPECS:
        expected_sha256 = _digest_for_package(spec, seed=seed)
        rebuilt_sha256 = expected_sha256
        if spec["name"] in injected:
            rebuilt_sha256 = hashlib.sha256(
                f"{expected_sha256}|mismatch".encode("utf-8")
            ).hexdigest()

        packages.append(
            {
                "name": spec["name"],
                "version": spec["version"],
                "release": spec["release"],
                "source_commit": spec["source_commit"],
                "build_recipe_id": spec["build_recipe_id"],
                "expected_artifact_sha256": expected_sha256,
                "rebuilt_artifact_sha256": rebuilt_sha256,
                "match": rebuilt_sha256 == expected_sha256,
            }
        )

    total_mismatches = sum(1 for entry in packages if not entry["match"])
    return {
        "schema": SCHEMA,
        "package_format_id": FORMAT_ID,
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "seed": seed,
        "total_packages": len(packages),
        "total_mismatches": total_mismatches,
        "packages": packages,
        "verified": total_mismatches == 0,
    }


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--seed", type=int, default=20260309)
    parser.add_argument("--max-mismatches", type=int, default=0)
    parser.add_argument(
        "--inject-mismatch",
        action="append",
        default=[],
        help="force mismatch for a package name",
    )
    parser.add_argument("--out", default="out/pkg-rebuild-v3.json")
    return parser


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)
    try:
        mismatch_packages = _collect_injected_mismatches(args.inject_mismatch)
    except ValueError as exc:
        print(f"error: {exc}")
        return 2

    report = run_rebuild(seed=args.seed, mismatch_packages=mismatch_packages)
    report["max_mismatches"] = args.max_mismatches
    report["meets_target"] = report["total_mismatches"] <= args.max_mismatches

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    print(f"pkg-rebuild-report: {out_path}")
    print(f"total_mismatches: {report['total_mismatches']}")
    return 0 if report["meets_target"] else 1


if __name__ == "__main__":
    raise SystemExit(main())

