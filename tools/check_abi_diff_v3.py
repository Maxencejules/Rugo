#!/usr/bin/env python3
"""Diff syscall ABI docs for M21 compatibility enforcement."""

from __future__ import annotations

import argparse
import json
import re
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List


SYSCALL_ROW_RE = re.compile(r"^\|\s*(\d+)\s*\|\s*`([^`]+)`\s*\|")
CONTRACT_ID_RE = re.compile(r"Syscall ABI identifier:\s*`([^`]+)`")


def parse_syscall_doc(path: Path) -> Dict[str, object]:
    text = path.read_text(encoding="utf-8")
    contract_match = CONTRACT_ID_RE.search(text)
    contract_id = contract_match.group(1) if contract_match else "unknown"

    by_id: Dict[int, str] = {}
    duplicate_ids: List[Dict[str, object]] = []
    duplicate_symbols: List[Dict[str, object]] = []

    for line in text.splitlines():
        match = SYSCALL_ROW_RE.match(line.strip())
        if not match:
            continue
        syscall_id = int(match.group(1))
        symbol = match.group(2).strip()
        if syscall_id in by_id and by_id[syscall_id] != symbol:
            duplicate_ids.append(
                {
                    "id": syscall_id,
                    "first_symbol": by_id[syscall_id],
                    "second_symbol": symbol,
                }
            )
            continue
        by_id[syscall_id] = symbol

    by_symbol: Dict[str, int] = {}
    for syscall_id, symbol in sorted(by_id.items()):
        if symbol in by_symbol and by_symbol[symbol] != syscall_id:
            duplicate_symbols.append(
                {
                    "symbol": symbol,
                    "first_id": by_symbol[symbol],
                    "second_id": syscall_id,
                }
            )
            continue
        by_symbol[symbol] = syscall_id

    return {
        "path": str(path),
        "contract_id": contract_id,
        "syscalls_by_id": by_id,
        "syscalls_by_symbol": by_symbol,
        "duplicate_ids": duplicate_ids,
        "duplicate_symbols": duplicate_symbols,
    }


def build_diff_report(base_path: Path, candidate_path: Path) -> Dict[str, object]:
    base = parse_syscall_doc(base_path)
    candidate = parse_syscall_doc(candidate_path)

    base_by_id = base["syscalls_by_id"]
    candidate_by_id = candidate["syscalls_by_id"]
    base_by_symbol = base["syscalls_by_symbol"]
    candidate_by_symbol = candidate["syscalls_by_symbol"]

    removed = [
        {"id": syscall_id, "symbol": base_by_id[syscall_id]}
        for syscall_id in sorted(base_by_id.keys() - candidate_by_id.keys())
    ]
    added = [
        {"id": syscall_id, "symbol": candidate_by_id[syscall_id]}
        for syscall_id in sorted(candidate_by_id.keys() - base_by_id.keys())
    ]
    renamed = [
        {
            "id": syscall_id,
            "from_symbol": base_by_id[syscall_id],
            "to_symbol": candidate_by_id[syscall_id],
        }
        for syscall_id in sorted(base_by_id.keys() & candidate_by_id.keys())
        if base_by_id[syscall_id] != candidate_by_id[syscall_id]
    ]
    renumbered = [
        {
            "symbol": symbol,
            "from_id": base_by_symbol[symbol],
            "to_id": candidate_by_symbol[symbol],
        }
        for symbol in sorted(base_by_symbol.keys() & candidate_by_symbol.keys())
        if base_by_symbol[symbol] != candidate_by_symbol[symbol]
    ]

    issues: List[Dict[str, object]] = []
    for item in removed:
        issues.append({"kind": "removed", **item})
    for item in renamed:
        issues.append({"kind": "renamed", **item})
    for item in renumbered:
        issues.append({"kind": "renumbered", **item})
    for item in base["duplicate_ids"]:
        issues.append({"kind": "base_duplicate_id", **item})
    for item in candidate["duplicate_ids"]:
        issues.append({"kind": "candidate_duplicate_id", **item})
    for item in base["duplicate_symbols"]:
        issues.append({"kind": "base_duplicate_symbol", **item})
    for item in candidate["duplicate_symbols"]:
        issues.append({"kind": "candidate_duplicate_symbol", **item})

    return {
        "schema": "rugo.abi_diff_report.v3",
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "base_contract_id": base["contract_id"],
        "candidate_contract_id": candidate["contract_id"],
        "base_doc": str(base_path),
        "candidate_doc": str(candidate_path),
        "base_total_syscalls": len(base_by_id),
        "candidate_total_syscalls": len(candidate_by_id),
        "added": added,
        "removed": removed,
        "renamed": renamed,
        "renumbered": renumbered,
        "breaking_changes": issues,
        "breaking_change_count": len(issues),
        "requires_version_bump": len(issues) > 0,
    }


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", default="docs/abi/syscall_v2.md")
    parser.add_argument("--candidate", default="docs/abi/syscall_v3.md")
    parser.add_argument("--out", default="out/abi-diff-v3.json")
    parser.add_argument(
        "--allow-breaking",
        action="store_true",
        help="Return success even when breaking changes are detected.",
    )
    return parser


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)
    report = build_diff_report(Path(args.base), Path(args.candidate))
    report["allow_breaking"] = args.allow_breaking
    report["gate_pass"] = report["breaking_change_count"] == 0 or args.allow_breaking

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    print(f"abi-diff-report: {out_path}")
    print(f"breaking_change_count: {report['breaking_change_count']}")
    return 0 if report["gate_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())

