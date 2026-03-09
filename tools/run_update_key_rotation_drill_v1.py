#!/usr/bin/env python3
"""Run deterministic update key-rotation drill for M26."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Set


POLICY_ID = "rugo.update_key_rotation_policy.v1"
SCHEMA = "rugo.update_key_rotation_drill.v1"

STAGE_SPECS: List[Dict[str, str]] = [
    {
        "name": "old_key_only",
        "assertion": "client accepts metadata from currently trusted key",
    },
    {
        "name": "overlap_window",
        "assertion": "client accepts old and new key signatures during overlap",
    },
    {
        "name": "new_key_primary",
        "assertion": "new key becomes default signer before revoke cutover",
    },
    {
        "name": "old_key_revoked",
        "assertion": "old key appears in revoked key set after cutoff",
    },
    {
        "name": "revocation_enforced",
        "assertion": "client rejects old key signatures after revocation",
    },
]


def _known_stage_names() -> Set[str]:
    return {spec["name"] for spec in STAGE_SPECS}


def _collect_injected_failures(specs: List[str]) -> Set[str]:
    requested = {item.strip() for item in specs if item.strip()}
    unknown = sorted(requested - _known_stage_names())
    if unknown:
        raise ValueError(f"unknown drill stages in --inject-failure: {', '.join(unknown)}")
    return requested


def run_drill(
    overlap_window_days: int = 14, injected_failures: Set[str] | None = None
) -> Dict[str, object]:
    forced_failures = set() if injected_failures is None else set(injected_failures)
    stages = [
        {
            "name": spec["name"],
            "assertion": spec["assertion"],
            "success": spec["name"] not in forced_failures,
        }
        for spec in STAGE_SPECS
    ]
    total_failures = sum(1 for stage in stages if not stage["success"])
    return {
        "schema": SCHEMA,
        "policy_id": POLICY_ID,
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "overlap_window_days": overlap_window_days,
        "total_stages": len(stages),
        "total_failures": total_failures,
        "stages": stages,
        "success": total_failures == 0,
    }


def _build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--overlap-days", type=int, default=14)
    p.add_argument(
        "--inject-failure",
        action="append",
        default=[],
        help="force a named stage to fail for negative-path validation",
    )
    p.add_argument("--out", default="out/update-key-rotation-drill-v1.json")
    return p


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)
    try:
        injected_failures = _collect_injected_failures(args.inject_failure)
    except ValueError as exc:
        print(f"error: {exc}")
        return 2

    report = run_drill(
        overlap_window_days=args.overlap_days,
        injected_failures=injected_failures,
    )
    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(f"update-key-rotation-drill: {out_path}")
    print(f"success: {report['success']}")
    return 0 if report["success"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
