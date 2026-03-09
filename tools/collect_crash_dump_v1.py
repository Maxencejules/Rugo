#!/usr/bin/env python3
"""Generate deterministic crash dump artifact for postmortem tests."""

from __future__ import annotations

import argparse
import hashlib
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List


SCHEMA = "rugo.crash_dump.v1"
CONTRACT_ID = "rugo.crash_dump_contract.v1"
PLAYBOOK_ID = "rugo.postmortem_triage_playbook.v1"
SYMBOL_MAP_ID = "rugo.kernel_symbol_map.v1"


def _dump_id(panic_code: int, kernel_build_id: str) -> str:
    digest = hashlib.sha256(f"{panic_code}|{kernel_build_id}".encode("utf-8")).hexdigest()
    return f"dump-{digest[:12]}"


def build_dump(
    panic_code: int,
    panic_reason: str = "kernel_panic",
    kernel_build_id: str = "rugo-kernel-2026.03.09",
) -> Dict[str, object]:
    return {
        "schema": SCHEMA,
        "contract_id": CONTRACT_ID,
        "triage_playbook_id": PLAYBOOK_ID,
        "symbol_map_id": SYMBOL_MAP_ID,
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "dump_id": _dump_id(panic_code=panic_code, kernel_build_id=kernel_build_id),
        "panic_code": panic_code,
        "panic_reason": panic_reason,
        "release_channel": "stable",
        "kernel_build_id": kernel_build_id,
        "registers": {
            "rip": "0xffffffff80001000",
            "rsp": "0xffffffff8100ff00",
            "rbp": "0xffffffff8100ff30",
        },
        "stack_frames": [
            {"ip": "0xffffffff80001000", "offset": 0},
            {"ip": "0xffffffff80002000", "offset": 24},
            {"ip": "0xffffffff80003000", "offset": 56},
        ],
    }


def _build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--panic-code", type=int, default=13)
    p.add_argument("--panic-reason", default="kernel_panic")
    p.add_argument("--kernel-build-id", default="rugo-kernel-2026.03.09")
    p.add_argument("--out", default="out/crash-dump-v1.json")
    return p


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)
    dump = build_dump(
        panic_code=args.panic_code,
        panic_reason=args.panic_reason,
        kernel_build_id=args.kernel_build_id,
    )
    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(dump, indent=2) + "\n", encoding="utf-8")
    print(f"crash-dump: {out_path}")
    print(f"panic_code: {dump['panic_code']}")
    print(f"dump_id: {dump['dump_id']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
