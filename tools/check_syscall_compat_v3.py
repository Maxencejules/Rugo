#!/usr/bin/env python3
"""Validate syscall compatibility obligations for the M21 v3 gate."""

from __future__ import annotations

import argparse
import json
import re
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Tuple

import check_abi_diff_v3 as abi_diff


TAGGED_WINDOW_RE = re.compile(r"Minimum tagged-release window:\s*(\d+)")
CALENDAR_WINDOW_RE = re.compile(r"Minimum calendar window:\s*(\d+)\s*days")
DEPRECATION_ROW_RE = re.compile(
    r"^\|\s*`([^`]+)`\s*\|\s*(active|deprecated|removed)\s*\|\s*([^|]+)\|\s*([^|]+)\|\s*([^|]+)\|",
    re.IGNORECASE,
)
RELEASE_RE = re.compile(r"^v(\d+)\.(\d+)$")


def _parse_release(tag: str) -> Tuple[int, int] | None:
    match = RELEASE_RE.match(tag.strip())
    if not match:
        return None
    return int(match.group(1)), int(match.group(2))


def _release_gap(old_release: str, new_release: str) -> int | None:
    old_parsed = _parse_release(old_release)
    new_parsed = _parse_release(new_release)
    if old_parsed is None or new_parsed is None:
        return None
    old_major, old_minor = old_parsed
    new_major, new_minor = new_parsed
    if new_major < old_major:
        return -1
    if new_major > old_major:
        # Cross-major removal is considered larger than any same-major window.
        return 10_000 + (new_major - old_major) * 100 + new_minor
    return new_minor - old_minor


def _extract_int(pattern: re.Pattern[str], text: str, default: int) -> int:
    match = pattern.search(text)
    if not match:
        return default
    return int(match.group(1))


def _collect_deprecations(syscall_doc_text: str) -> List[Dict[str, str]]:
    rows: List[Dict[str, str]] = []
    for line in syscall_doc_text.splitlines():
        match = DEPRECATION_ROW_RE.match(line.strip())
        if not match:
            continue
        rows.append(
            {
                "symbol": match.group(1).strip(),
                "state": match.group(2).strip().lower(),
                "deprecated_in": match.group(3).strip(),
                "earliest_removal": match.group(4).strip(),
                "replacement": match.group(5).strip(),
            }
        )
    return rows


def _require_tokens(
    text: str,
    tokens: List[str],
    label: str,
    issues: List[Dict[str, object]],
) -> None:
    for token in tokens:
        if token not in text:
            issues.append(
                {"kind": "missing_policy_token", "source": label, "token": token}
            )


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", default="docs/abi/syscall_v2.md")
    parser.add_argument("--candidate", default="docs/abi/syscall_v3.md")
    parser.add_argument("--abi-policy", default="docs/runtime/abi_stability_policy_v2.md")
    parser.add_argument(
        "--deprecation-policy",
        default="docs/runtime/deprecation_window_policy_v1.md",
    )
    parser.add_argument("--diff-report", default="")
    parser.add_argument(
        "--version-action",
        choices=["none", "major-abi-bump"],
        default="none",
    )
    parser.add_argument("--migration-doc", default="")
    parser.add_argument("--out", default="out/syscall-compat-v3.json")
    return parser


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)

    base_path = Path(args.base)
    candidate_path = Path(args.candidate)
    abi_policy_path = Path(args.abi_policy)
    deprecation_policy_path = Path(args.deprecation_policy)

    if args.diff_report:
        diff_payload = json.loads(Path(args.diff_report).read_text(encoding="utf-8"))
    else:
        diff_payload = abi_diff.build_diff_report(base_path, candidate_path)

    abi_policy_text = abi_policy_path.read_text(encoding="utf-8")
    deprecation_policy_text = deprecation_policy_path.read_text(encoding="utf-8")
    candidate_text = candidate_path.read_text(encoding="utf-8")

    issues: List[Dict[str, object]] = []

    _require_tokens(
        abi_policy_text,
        [
            "Policy ID: `rugo.runtime_abi_policy.v2`",
            "Stability window:",
            "tools/check_abi_diff_v3.py",
            "tools/check_syscall_compat_v3.py",
            "make test-abi-stability-v3",
        ],
        "abi_stability_policy_v2",
        issues,
    )
    _require_tokens(
        deprecation_policy_text,
        [
            "Policy ID: `rugo.deprecation_window_policy.v1`",
            "Minimum tagged-release window:",
            "Minimum calendar window:",
        ],
        "deprecation_window_policy_v1",
        issues,
    )

    min_tagged_window = _extract_int(TAGGED_WINDOW_RE, deprecation_policy_text, 3)
    min_calendar_days = _extract_int(CALENDAR_WINDOW_RE, deprecation_policy_text, 180)

    deprecations = _collect_deprecations(candidate_text)
    for row in deprecations:
        if row["state"] not in {"deprecated", "removed"}:
            continue
        if row["replacement"].lower() in {"", "n/a", "none", "-"}:
            issues.append(
                {
                    "kind": "missing_replacement",
                    "symbol": row["symbol"],
                }
            )
        release_gap = _release_gap(row["deprecated_in"], row["earliest_removal"])
        if release_gap is None:
            issues.append(
                {
                    "kind": "invalid_release_tag",
                    "symbol": row["symbol"],
                    "deprecated_in": row["deprecated_in"],
                    "earliest_removal": row["earliest_removal"],
                }
            )
            continue
        if release_gap < min_tagged_window:
            issues.append(
                {
                    "kind": "window_too_short",
                    "symbol": row["symbol"],
                    "required_window": min_tagged_window,
                    "actual_window": release_gap,
                }
            )

    breaking_change_count = int(diff_payload.get("breaking_change_count", 0))
    requires_explicit_migration = breaking_change_count > 0
    if requires_explicit_migration:
        if args.version_action != "major-abi-bump":
            issues.append(
                {
                    "kind": "missing_major_bump_action",
                    "required_version_action": "major-abi-bump",
                }
            )
        if not args.migration_doc:
            issues.append({"kind": "missing_migration_doc"})
        elif not Path(args.migration_doc).is_file():
            issues.append(
                {"kind": "migration_doc_not_found", "path": args.migration_doc}
            )

    gate_pass = len(issues) == 0
    report = {
        "schema": "rugo.syscall_compat_report.v3",
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "base_contract_id": diff_payload.get("base_contract_id", "unknown"),
        "candidate_contract_id": diff_payload.get("candidate_contract_id", "unknown"),
        "minimum_tagged_release_window": min_tagged_window,
        "minimum_calendar_days": min_calendar_days,
        "compat_matrix": [
            {
                "direction": "backward",
                "from": diff_payload.get("base_contract_id", "unknown"),
                "to": diff_payload.get("candidate_contract_id", "unknown"),
                "compatible": breaking_change_count == 0,
                "breaking_change_count": breaking_change_count,
            },
            {
                "direction": "forward",
                "from": diff_payload.get("candidate_contract_id", "unknown"),
                "to": diff_payload.get("base_contract_id", "unknown"),
                "compatible": False,
                "reason": "forward compatibility across major ABI lines is not guaranteed",
            },
        ],
        "deprecation_registry": {
            "entries": deprecations,
            "entry_count": len(deprecations),
        },
        "requires_explicit_migration": requires_explicit_migration,
        "explicit_actions": {
            "version_action": args.version_action,
            "migration_doc": args.migration_doc,
        },
        "policy_issues": issues,
        "gate_pass": gate_pass,
    }

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    print(f"syscall-compat-report: {out_path}")
    print(f"policy_issues: {len(issues)}")
    return 0 if gate_pass else 1


if __name__ == "__main__":
    raise SystemExit(main())

