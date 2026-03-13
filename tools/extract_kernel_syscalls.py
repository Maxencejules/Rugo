#!/usr/bin/env python3
"""Extract the syscall dispatch table from the kernel Rust source.

This tool parses kernel_rs/src/lib.rs to discover all syscall IDs and their
handler function names.  It produces a JSON report that downstream tools
(check_abi_diff_v3.py, tests) can compare against the ABI docs to ensure
the published syscall surface matches the actual kernel implementation.

Usage:
  python tools/extract_kernel_syscalls.py --out out/kernel-syscall-table.json
"""

from __future__ import annotations

import argparse
import json
import re
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List

# Matches lines like:  0  => sys_debug_write(arg1, arg2),
# or:                   0  => { *frame.add(14) = sys_debug_write(arg1, arg2); }
# or:                   2  => { r4_exit_and_switch(frame, 0); }
# or:                   9  => { sys_ipc_recv_r4(frame, arg1, arg2, arg3); }
MATCH_ARM_RE = re.compile(
    r"^\s*(\d+)\s*=>\s*\{?\s*(?:\*frame\.add\(\d+\)\s*=\s*)?(\w+)\(",
)

# Canonical name mapping: strip version/dispatch suffixes to get the ABI name.
# E.g. sys_thread_spawn_m3 -> sys_thread_spawn,  r4_exit_and_switch -> sys_thread_exit
_HANDLER_TO_ABI: Dict[str, str] = {
    "sys_thread_spawn_m3": "sys_thread_spawn",
    "sys_thread_spawn_r4": "sys_thread_spawn",
    "sys_yield_m3": "sys_yield",
    "r4_yield_and_switch": "sys_yield",
    "r4_exit_and_switch": "sys_thread_exit",
    "sys_thread_exit_m3": "sys_thread_exit",
    "sys_vm_map_m3": "sys_vm_map",
    "sys_vm_unmap_m3": "sys_vm_unmap",
    "sys_ipc_endpoint_create_r4": "sys_ipc_endpoint_create",
    "sys_shm_create_r4": "sys_shm_create",
    "sys_shm_map_r4": "sys_shm_map",
    "sys_shm_unmap_r4": "sys_shm_unmap",
    "sys_ipc_send_r4": "sys_ipc_send",
    "sys_ipc_recv_r4": "sys_ipc_recv",
    "sys_svc_register_r4": "sys_svc_register",
    "sys_svc_lookup_r4": "sys_svc_lookup",
    "sys_open_v1": "sys_open",
    "sys_read_v1": "sys_read",
    "sys_write_v1": "sys_write",
    "sys_close_v1": "sys_close",
    "sys_wait_v1": "sys_wait",
    "sys_wait_r4": "sys_wait",
    "sys_poll_v1": "sys_poll",
    "sys_fd_rights_get_v1": "sys_fd_rights_get",
    "sys_fd_rights_reduce_v1": "sys_fd_rights_reduce",
    "sys_fd_rights_transfer_v1": "sys_fd_rights_transfer",
    "sys_sec_profile_set_v1": "sys_sec_profile_set",
    "sys_fsync_v1": "sys_fsync",
    "sys_proc_info_r4": "sys_proc_info",
    "sys_sched_set_r4": "sys_sched_set",
    "sys_socket_open_r4": "sys_socket_open",
    "sys_socket_bind_r4": "sys_socket_bind",
    "sys_socket_listen_r4": "sys_socket_listen",
    "sys_socket_connect_r4": "sys_socket_connect",
    "sys_socket_accept_r4": "sys_socket_accept",
    "sys_socket_send_r4": "sys_socket_send",
    "sys_socket_recv_r4": "sys_socket_recv",
    "sys_socket_close_r4": "sys_socket_close",
    "sys_net_if_config_r4": "sys_net_if_config",
    "sys_net_route_add_r4": "sys_net_route_add",
    "sys_isolation_config_r4": "sys_isolation_config",
}


def _normalize_handler(handler: str) -> str:
    """Map a handler function name to the canonical ABI syscall name."""
    if handler in _HANDLER_TO_ABI:
        return _HANDLER_TO_ABI[handler]
    # Strip common suffixes if not in the explicit map.
    for suffix in ("_v1", "_v2", "_v3", "_m3", "_r4"):
        if handler.endswith(suffix):
            return handler[: -len(suffix)]
    return handler


def extract_syscalls(source_path: Path) -> Dict[int, str]:
    """Return {syscall_id: canonical_name} from the kernel source."""
    text = source_path.read_text(encoding="utf-8")
    by_id: Dict[int, str] = {}

    for line in text.splitlines():
        # Skip wildcard arms and qemu_exit arms.
        stripped = line.strip()
        if stripped.startswith("_") or "qemu_exit" in stripped:
            continue

        m = MATCH_ARM_RE.match(stripped)
        if not m:
            continue

        syscall_id = int(m.group(1))
        handler = m.group(2)

        # Skip internal/test-only IDs (98 = qemu_exit).
        if syscall_id == 98:
            continue

        canonical = _normalize_handler(handler)
        if syscall_id not in by_id:
            by_id[syscall_id] = canonical

    return by_id


def build_report(source_path: Path) -> Dict[str, object]:
    by_id = extract_syscalls(source_path)
    entries = [
        {"id": sid, "name": name} for sid, name in sorted(by_id.items())
    ]
    return {
        "schema": "rugo.kernel_syscall_table.v1",
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "source_file": str(source_path),
        "total_syscalls": len(entries),
        "syscalls": entries,
        "syscalls_by_id": {str(sid): name for sid, name in sorted(by_id.items())},
    }


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--source",
        default="kernel_rs/src/lib.rs",
        help="Path to the kernel Rust source file.",
    )
    parser.add_argument("--out", default="out/kernel-syscall-table.json")
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

    print(f"kernel-syscall-table: {out_path}")
    print(f"total_syscalls: {report['total_syscalls']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
