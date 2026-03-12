#!/usr/bin/env python3
"""Run deterministic native storage diagnostics checks for M54."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
from typing import Dict, List, Sequence, Set

import run_hw_matrix_v7 as matrix_v7


SCHEMA = "rugo.native_storage_diagnostics_report.v1"
CONTRACT_ID = "rugo.nvme_ahci_contract.v1"
PARENT_NATIVE_DRIVER_CONTRACT_ID = "rugo.native_driver_contract.v1"
BLOCK_FLUSH_CONTRACT_ID = "rugo.block_flush_contract.v1"
DEFAULT_SEED = 20260312


CHECK_IDS = {
    "source_matrix_green",
    "nvme_ready",
    "nvme_identify_ok",
    "nvme_admin_queue_ok",
    "nvme_io_queue_ok",
    "nvme_reset_recover",
    "nvme_power_state_bounded",
    "ahci_port_up",
    "ahci_rw_ok",
    "ahci_flush_ok",
    "ahci_reset_recover",
    "blk_fua_ok",
    "blk_flush_ordered",
    "negative_namespace_missing",
    "negative_port_absent",
}


def known_checks() -> Set[str]:
    return set(CHECK_IDS)


def _noise(seed: int, key: str) -> int:
    digest = hashlib.sha256(f"{seed}|{key}".encode("utf-8")).hexdigest()
    return int(digest[:8], 16)


def _metric(seed: int, key: str, base: int, spread: int) -> int:
    return base + (_noise(seed, key) % spread)


def _check_row(
    check_id: str,
    domain: str,
    operator: str,
    threshold: float | bool | str,
    observed: float | bool | str,
) -> Dict[str, object]:
    if operator == "eq":
        passed = observed == threshold
    elif operator == "max":
        passed = float(observed) <= float(threshold)
    elif operator == "min":
        passed = float(observed) >= float(threshold)
    else:
        raise ValueError(f"unsupported operator: {operator}")
    return {
        "check_id": check_id,
        "domain": domain,
        "operator": operator,
        "threshold": threshold,
        "observed": observed,
        "pass": passed,
    }


def _domain_summary(checks: List[Dict[str, object]], domain: str) -> Dict[str, object]:
    scoped = [entry for entry in checks if entry["domain"] == domain]
    failures = [entry for entry in scoped if entry["pass"] is False]
    return {
        "checks": len(scoped),
        "failures": len(failures),
        "pass": len(failures) == 0,
    }


def normalize_failures(values: Sequence[str]) -> Set[str]:
    failures = {value.strip() for value in values if value.strip()}
    unknown = sorted(failures - known_checks())
    if unknown:
        raise ValueError(f"unknown check ids in --inject-failure: {', '.join(unknown)}")
    return failures


def _event(
    event_id: str,
    driver: str,
    phase: str,
    marker: str,
    status: str,
    details: Dict[str, object],
) -> Dict[str, object]:
    return {
        "event_id": event_id,
        "driver": driver,
        "phase": phase,
        "marker": marker,
        "status": status,
        "details": details,
    }


def run_diagnostics(
    seed: int,
    injected_failures: Set[str] | None = None,
    max_failures: int = 0,
) -> Dict[str, object]:
    failures = set() if injected_failures is None else set(injected_failures)

    matrix_report = matrix_v7.run_matrix(seed=seed, max_failures=0)
    source_green = bool(matrix_report["gate_pass"]) and "source_matrix_green" not in failures

    nvme_ready = "nvme_ready" not in failures
    nvme_identify = "nvme_identify_ok" not in failures
    nvme_admin_queue = "nvme_admin_queue_ok" not in failures
    nvme_io_queue = "nvme_io_queue_ok" not in failures
    nvme_reset = "nvme_reset_recover" not in failures
    nvme_power = "nvme_power_state_bounded" not in failures
    ahci_port = "ahci_port_up" not in failures
    ahci_rw = "ahci_rw_ok" not in failures
    ahci_flush = "ahci_flush_ok" not in failures
    ahci_reset = "ahci_reset_recover" not in failures
    blk_fua = "blk_fua_ok" not in failures
    blk_flush = "blk_flush_ordered" not in failures
    negative_namespace = "negative_namespace_missing" not in failures
    negative_port = "negative_port_absent" not in failures

    checks: List[Dict[str, object]] = [
        _check_row("source_matrix_green", "source", "eq", True, source_green),
        _check_row("nvme_ready", "nvme", "eq", True, source_green and nvme_ready),
        _check_row("nvme_identify_ok", "nvme", "eq", True, source_green and nvme_identify),
        _check_row(
            "nvme_admin_queue_ok",
            "nvme",
            "eq",
            True,
            source_green and nvme_admin_queue,
        ),
        _check_row("nvme_io_queue_ok", "nvme", "eq", True, source_green and nvme_io_queue),
        _check_row("nvme_reset_recover", "nvme", "eq", True, source_green and nvme_reset),
        _check_row(
            "nvme_power_state_bounded",
            "nvme",
            "eq",
            True,
            source_green and nvme_power,
        ),
        _check_row("ahci_port_up", "ahci", "eq", True, source_green and ahci_port),
        _check_row("ahci_rw_ok", "ahci", "eq", True, source_green and ahci_rw),
        _check_row("ahci_flush_ok", "ahci", "eq", True, source_green and ahci_flush),
        _check_row("ahci_reset_recover", "ahci", "eq", True, source_green and ahci_reset),
        _check_row("blk_fua_ok", "flush", "eq", True, source_green and blk_fua),
        _check_row("blk_flush_ordered", "flush", "eq", True, source_green and blk_flush),
        _check_row(
            "negative_namespace_missing",
            "negative_path",
            "eq",
            True,
            negative_namespace,
        ),
        _check_row("negative_port_absent", "negative_path", "eq", True, negative_port),
    ]

    nvme_status = all(
        [
            source_green,
            nvme_ready,
            nvme_identify,
            nvme_admin_queue,
            nvme_io_queue,
            nvme_reset,
            nvme_power,
        ]
    )
    ahci_status = all([source_green, ahci_port, ahci_rw, ahci_flush, ahci_reset])
    flush_status = source_green and blk_fua and blk_flush

    controllers = [
        {
            "driver": "nvme",
            "machine": "q35",
            "profile": "tier0-emulated",
            "transport": "pcie",
            "irq_mode": "msix",
            "admin_queue_depth": 32,
            "io_queue_depth": 64,
            "power_state_window": ["ps0", "ps1", "ps3"],
            "markers": [
                "NVME: ready" if nvme_ready and source_green else "NVME: namespace missing",
                "NVME: identify ok" if nvme_identify and source_green else "NVME: namespace missing",
                "NVME: io queue ok" if nvme_io_queue and source_green else "BLK: flush timeout",
                "NVME: reset recover" if nvme_reset and source_green else "BLK: flush timeout",
                "BLK: fua ok" if blk_fua and source_green else "BLK: flush timeout",
            ],
            "namespaces": [
                {
                    "nsid": 1,
                    "size_gib": 64,
                    "lba_bytes": 4096,
                    "fua_supported": True,
                    "status": "pass" if nvme_status else "fail",
                }
            ],
            "status": "pass" if nvme_status else "fail",
        },
        {
            "driver": "ahci",
            "machine": "pc/i440fx",
            "profile": "tier1-emulated",
            "transport": "sata",
            "irq_mode": "msi",
            "ncq_depth": 32,
            "markers": [
                "AHCI: port up" if ahci_port and source_green else "AHCI: port absent",
                "AHCI: rw ok" if ahci_rw and source_green else "BLK: flush timeout",
                "AHCI: flush ok" if ahci_flush and source_green else "BLK: flush timeout",
                "BLK: flush ordered" if blk_flush and source_green else "BLK: flush timeout",
            ],
            "ports": [
                {
                    "port": 0,
                    "link_state": "up" if ahci_port and source_green else "absent",
                    "device_model": "sata-ssd-shadow",
                    "ncq_depth": 32,
                    "status": "pass" if ahci_status else "fail",
                }
            ],
            "status": "pass" if ahci_status else "fail",
        },
    ]

    queue_audits = [
        {
            "audit_id": "nvme_admin_identify",
            "driver": "nvme",
            "queue_kind": "admin",
            "depth": 32,
            "completion_p95_ms": _metric(seed, "nvme_admin_identify", 2, 3),
            "irq_mode": "msix",
            "marker": "NVME: identify ok" if nvme_identify and source_green else "NVME: namespace missing",
            "status": "pass" if source_green and nvme_identify and nvme_admin_queue else "fail",
        },
        {
            "audit_id": "nvme_io_submission",
            "driver": "nvme",
            "queue_kind": "io",
            "depth": 64,
            "completion_p95_ms": _metric(seed, "nvme_io_submission", 3, 4),
            "irq_mode": "msix",
            "fua_supported": True,
            "marker": "NVME: io queue ok" if nvme_io_queue and source_green else "BLK: flush timeout",
            "status": "pass" if source_green and nvme_io_queue else "fail",
        },
        {
            "audit_id": "ahci_dma_rw",
            "driver": "ahci",
            "queue_kind": "dma",
            "depth": 32,
            "completion_p95_ms": _metric(seed, "ahci_dma_rw", 4, 4),
            "irq_mode": "msi",
            "marker": "AHCI: rw ok" if ahci_rw and source_green else "BLK: flush timeout",
            "status": "pass" if source_green and ahci_rw else "fail",
        },
    ]

    flush_audits = [
        {
            "audit_id": "nvme_fua_write",
            "driver": "nvme",
            "device_class": "nvme",
            "command": "write+fua",
            "latency_ms": _metric(seed, "nvme_fua_write", 2, 3),
            "data_durable": source_green and blk_fua,
            "metadata_durable": source_green and blk_fua,
            "marker": "BLK: fua ok" if blk_fua and source_green else "BLK: flush timeout",
            "status": "pass" if source_green and blk_fua else "fail",
        },
        {
            "audit_id": "nvme_fsync_bridge",
            "driver": "nvme",
            "device_class": "nvme",
            "command": "fsync",
            "latency_ms": _metric(seed, "nvme_fsync_bridge", 3, 3),
            "data_durable": source_green and blk_fua,
            "metadata_durable": source_green and blk_fua,
            "marker": "BLK: fua ok" if blk_fua and source_green else "BLK: flush timeout",
            "status": "pass" if source_green and blk_fua else "fail",
        },
        {
            "audit_id": "ahci_cache_flush",
            "driver": "ahci",
            "device_class": "ahci",
            "command": "cache_flush",
            "latency_ms": _metric(seed, "ahci_cache_flush", 4, 4),
            "data_durable": source_green and ahci_flush,
            "metadata_durable": source_green and ahci_flush,
            "marker": "AHCI: flush ok" if ahci_flush and source_green else "BLK: flush timeout",
            "status": "pass" if source_green and ahci_flush else "fail",
        },
        {
            "audit_id": "ahci_fsync_bridge",
            "driver": "ahci",
            "device_class": "ahci",
            "command": "fsync",
            "latency_ms": _metric(seed, "ahci_fsync_bridge", 5, 3),
            "data_durable": source_green and blk_flush,
            "metadata_durable": source_green and blk_flush,
            "marker": "BLK: flush ordered" if blk_flush and source_green else "BLK: flush timeout",
            "status": "pass" if source_green and blk_flush else "fail",
        },
    ]

    negative_paths = {
        "nvme_missing_namespace": {
            "marker": "NVME: namespace missing",
            "deterministic": negative_namespace,
            "status": "pass" if negative_namespace else "fail",
        },
        "ahci_port_absent": {
            "marker": "AHCI: port absent",
            "deterministic": negative_port,
            "status": "pass" if negative_port else "fail",
        },
    }

    diagnostic_events = [
        _event(
            "nvme::ready",
            "nvme",
            "probe",
            "NVME: ready" if nvme_ready and source_green else "NVME: namespace missing",
            "pass" if source_green and nvme_ready else "fail",
            {"machine": "q35"},
        ),
        _event(
            "nvme::identify",
            "nvme",
            "queue",
            "NVME: identify ok" if nvme_identify and source_green else "NVME: namespace missing",
            "pass" if source_green and nvme_identify else "fail",
            {"nsid": 1},
        ),
        _event(
            "nvme::io",
            "nvme",
            "queue",
            "NVME: io queue ok" if nvme_io_queue and source_green else "BLK: flush timeout",
            "pass" if source_green and nvme_io_queue else "fail",
            {"queue_depth": 64},
        ),
        _event(
            "ahci::port",
            "ahci",
            "probe",
            "AHCI: port up" if ahci_port and source_green else "AHCI: port absent",
            "pass" if source_green and ahci_port else "fail",
            {"port": 0},
        ),
        _event(
            "ahci::rw",
            "ahci",
            "queue",
            "AHCI: rw ok" if ahci_rw and source_green else "BLK: flush timeout",
            "pass" if source_green and ahci_rw else "fail",
            {"queue_depth": 32},
        ),
        _event(
            "nvme::flush",
            "nvme",
            "flush",
            "BLK: fua ok" if blk_fua and source_green else "BLK: flush timeout",
            "pass" if source_green and blk_fua else "fail",
            {"command": "write+fua"},
        ),
        _event(
            "ahci::flush",
            "ahci",
            "flush",
            "BLK: flush ordered" if blk_flush and source_green else "BLK: flush timeout",
            "pass" if source_green and blk_flush else "fail",
            {"command": "fsync"},
        ),
    ]

    total_failures = sum(1 for row in checks if row["pass"] is False)
    gate_pass = total_failures <= max_failures

    stable_payload = {
        "schema": SCHEMA,
        "seed": seed,
        "matrix_digest": matrix_report["digest"],
        "checks": [
            {
                "check_id": row["check_id"],
                "pass": row["pass"],
                "observed": row["observed"],
            }
            for row in checks
        ],
        "queue_audits": [
            {
                "audit_id": row["audit_id"],
                "status": row["status"],
            }
            for row in queue_audits
        ],
        "flush_audits": [
            {
                "audit_id": row["audit_id"],
                "status": row["status"],
                "marker": row["marker"],
            }
            for row in flush_audits
        ],
        "injected_failures": sorted(failures),
        "max_failures": max_failures,
    }
    digest = hashlib.sha256(
        json.dumps(stable_payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
    ).hexdigest()

    return {
        "schema": SCHEMA,
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "contract_id": CONTRACT_ID,
        "parent_native_driver_contract_id": PARENT_NATIVE_DRIVER_CONTRACT_ID,
        "block_flush_contract_id": BLOCK_FLUSH_CONTRACT_ID,
        "matrix_contract_id": matrix_report["matrix_contract_id"],
        "seed": seed,
        "gate": "test-native-storage-v1",
        "matrix_gate": "test-hw-matrix-v7",
        "checks": checks,
        "summary": {
            "source": _domain_summary(checks, "source"),
            "nvme": _domain_summary(checks, "nvme"),
            "ahci": _domain_summary(checks, "ahci"),
            "flush": _domain_summary(checks, "flush"),
            "negative_path": _domain_summary(checks, "negative_path"),
        },
        "controllers": controllers,
        "queue_audits": queue_audits,
        "flush_audits": flush_audits,
        "negative_paths": negative_paths,
        "durability_bridge": {
            "block_flush_contract_id": BLOCK_FLUSH_CONTRACT_ID,
            "fsync_device_class": "nvme",
            "fdatasync_device_class": "ahci",
            "required_markers": ["BLK: fua ok", "BLK: flush ordered"],
            "status": "pass" if flush_status else "fail",
        },
        "diagnostic_events": diagnostic_events,
        "source_reports": {
            "matrix_v7": {
                "schema": matrix_report["schema"],
                "digest": matrix_report["digest"],
                "gate_pass": matrix_report["gate_pass"],
                "gate": matrix_report["gate"],
                "contract_id": matrix_report["matrix_contract_id"],
            }
        },
        "artifact_refs": {
            "junit": "out/pytest-native-storage-v1.xml",
            "diagnostics_report": "out/native-storage-v1.json",
            "matrix_report": "out/hw-matrix-v7.json",
            "ci_artifact": "native-storage-v1-artifacts",
            "matrix_ci_artifact": "hw-matrix-v7-artifacts",
        },
        "injected_failures": sorted(failures),
        "max_failures": max_failures,
        "total_failures": total_failures,
        "failures": sorted(row["check_id"] for row in checks if row["pass"] is False),
        "gate_pass": gate_pass,
        "digest": digest,
    }


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--seed", type=int, default=DEFAULT_SEED)
    parser.add_argument(
        "--inject-failure",
        action="append",
        default=[],
        help="force a diagnostics check to fail by check_id",
    )
    parser.add_argument("--max-failures", type=int, default=0)
    parser.add_argument("--out", default="out/native-storage-v1.json")
    return parser


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)
    if args.max_failures < 0:
        print("error: max-failures must be >= 0")
        return 2

    try:
        injected_failures = normalize_failures(args.inject_failure)
    except ValueError as exc:
        print(f"error: {exc}")
        return 2

    report = run_diagnostics(
        seed=args.seed,
        injected_failures=injected_failures,
        max_failures=args.max_failures,
    )

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    print(f"native-storage-report: {out_path}")
    print(f"total_failures: {report['total_failures']}")
    print(f"gate_pass: {report['gate_pass']}")
    return 0 if report["gate_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
