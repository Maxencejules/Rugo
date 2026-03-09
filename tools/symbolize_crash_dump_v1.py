#!/usr/bin/env python3
"""Symbolize a crash dump artifact using deterministic symbol mapping."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List


SCHEMA = "rugo.crash_dump_symbolized.v1"
SYMBOL_MAP_ID = "rugo.kernel_symbol_map.v1"
PLAYBOOK_ID = "rugo.postmortem_triage_playbook.v1"

SYMBOLS = {
    "0xffffffff80001000": "kernel::panic_entry",
    "0xffffffff80002000": "kernel::scheduler::tick",
    "0xffffffff80003000": "kernel::syscall::dispatch",
}


def symbolize(dump: Dict[str, object]) -> Dict[str, object]:
    frames = []
    resolved = 0
    unresolved = 0

    for frame in dump.get("stack_frames", []):
        ip = str(frame.get("ip"))
        symbol = SYMBOLS.get(ip, "unknown")
        if symbol == "unknown":
            unresolved += 1
        else:
            resolved += 1
        frames.append(
            {
                "ip": ip,
                "symbol": symbol,
                "offset": frame.get("offset", 0),
            }
        )

    return {
        "schema": SCHEMA,
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "source_schema": dump.get("schema", "unknown"),
        "contract_id": dump.get("contract_id", "unknown"),
        "symbol_map_id": dump.get("symbol_map_id", SYMBOL_MAP_ID),
        "triage_playbook_id": dump.get("triage_playbook_id", PLAYBOOK_ID),
        "panic_code": dump.get("panic_code", -1),
        "panic_reason": dump.get("panic_reason", ""),
        "dump_id": dump.get("dump_id", ""),
        "frames": frames,
        "resolved_frames": resolved,
        "unresolved_frames": unresolved,
    }


def _build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--dump", default="")
    p.add_argument("--max-unresolved", type=int, default=0)
    p.add_argument("--out", default="out/crash-dump-symbolized-v1.json")
    return p


def _default_dump() -> Dict[str, object]:
    return {
        "schema": "rugo.crash_dump.v1",
        "contract_id": "rugo.crash_dump_contract.v1",
        "symbol_map_id": "rugo.kernel_symbol_map.v1",
        "triage_playbook_id": "rugo.postmortem_triage_playbook.v1",
        "panic_code": 13,
        "panic_reason": "kernel_panic",
        "stack_frames": [
            {"ip": "0xffffffff80001000", "offset": 0},
            {"ip": "0xffffffff80002000", "offset": 24},
            {"ip": "0xffffffff80003000", "offset": 56},
        ],
    }


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)
    if args.dump:
        dump = json.loads(Path(args.dump).read_text(encoding="utf-8"))
    else:
        dump = _default_dump()
    report = symbolize(dump)
    report["max_unresolved"] = args.max_unresolved
    report["all_frames_symbolized"] = report["unresolved_frames"] == 0
    report["gate_pass"] = report["unresolved_frames"] <= args.max_unresolved

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(f"crash-dump-symbolized: {out_path}")
    print(f"frames: {len(report['frames'])}")
    print(f"unresolved_frames: {report['unresolved_frames']}")
    return 0 if report["gate_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
