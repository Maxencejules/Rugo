#!/usr/bin/env python3
"""Extract the shipped stock-Go syscall wrapper interface.

This tool parses services/go_std/syscalls.asm and emits a machine-readable
report for the supported stock-Go userspace lane. The ABI gate uses it to
verify that the published syscall docs still match the userspace interface
that the repo actually ships.
"""

from __future__ import annotations

import argparse
import json
import re
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List


GLOBAL_RE = re.compile(r"^global\s+(main\.\w+)\s*$")
LABEL_RE = re.compile(r"^(main\.\w+):\s*$")
MOV_EAX_RE = re.compile(r"^\s*mov\s+eax,\s*(\d+)\s*$", re.IGNORECASE)
XOR_EAX_RE = re.compile(r"^\s*xor\s+eax,\s*eax\s*$", re.IGNORECASE)
INT80_RE = re.compile(r"^\s*int\s+0x80\s*$", re.IGNORECASE)

WRAPPER_TO_ABI: Dict[str, str] = {
    "main.sysDebugWrite": "sys_debug_write",
    "main.sysThreadSpawn": "sys_thread_spawn",
    "main.sysThreadExit": "sys_thread_exit",
    "main.sysYield": "sys_yield",
    "main.sysVmMap": "sys_vm_map",
    "main.sysVmUnmap": "sys_vm_unmap",
    "main.sysTimeNow": "sys_time_now",
}


def extract_wrappers(source_path: Path) -> List[Dict[str, object]]:
    """Return the shipped wrapper set from services/go_std/syscalls.asm."""
    lines = source_path.read_text(encoding="utf-8").splitlines()
    declared = set()
    wrappers: List[Dict[str, object]] = []
    current_wrapper = ""
    current_syscall_id: int | None = None

    for raw_line in lines:
        line = raw_line.strip()

        global_match = GLOBAL_RE.match(line)
        if global_match:
            declared.add(global_match.group(1))
            continue

        label_match = LABEL_RE.match(line)
        if label_match:
            label = label_match.group(1)
            current_wrapper = label if label in WRAPPER_TO_ABI else ""
            current_syscall_id = None
            continue

        if not current_wrapper:
            continue

        mov_match = MOV_EAX_RE.match(line)
        if mov_match:
            current_syscall_id = int(mov_match.group(1))
            continue

        if XOR_EAX_RE.match(line):
            current_syscall_id = 0
            continue

        if INT80_RE.match(line) and current_syscall_id is not None:
            wrappers.append(
                {
                    "wrapper": current_wrapper,
                    "id": current_syscall_id,
                    "name": WRAPPER_TO_ABI[current_wrapper],
                }
            )
            current_wrapper = ""
            current_syscall_id = None

    missing = sorted(name for name in WRAPPER_TO_ABI if name not in declared)
    if missing:
        raise RuntimeError(
            "Missing expected stock-Go wrapper declaration(s): "
            + ", ".join(missing)
        )

    wrappers.sort(key=lambda entry: int(entry["id"]))
    return wrappers


def extract_syscalls(source_path: Path) -> Dict[int, str]:
    """Return {syscall_id: canonical_name} for the shipped stock-Go lane."""
    return {
        int(entry["id"]): str(entry["name"])
        for entry in extract_wrappers(source_path)
    }


def build_report(source_path: Path) -> Dict[str, object]:
    wrappers = extract_wrappers(source_path)
    return {
        "schema": "rugo.gostd_syscall_interface.v1",
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "source_file": str(source_path),
        "total_wrappers": len(wrappers),
        "syscalls": wrappers,
        "syscalls_by_id": {
            str(entry["id"]): entry["name"] for entry in wrappers
        },
        "wrappers_by_symbol": {
            entry["wrapper"]: {"id": entry["id"], "name": entry["name"]}
            for entry in wrappers
        },
    }


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--source",
        default="services/go_std/syscalls.asm",
        help="Path to the stock-Go syscall wrapper source file.",
    )
    parser.add_argument("--out", default="out/gostd-syscall-interface.json")
    return parser


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)
    source_path = Path(args.source)
    if not source_path.is_file():
        print(f"ERROR: source file not found: {source_path}")
        return 1

    report = build_report(source_path)
    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    print(f"gostd-syscall-interface: {out_path}")
    print(f"total_wrappers: {report['total_wrappers']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
