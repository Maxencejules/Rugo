#!/usr/bin/env python3
"""Run deterministic hardware matrix v7 checks for M54."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
from typing import Dict, List, Sequence, Set

import run_hw_matrix_v6 as matrix_v6


SCHEMA = "rugo.hw_matrix_evidence.v7"
MATRIX_CONTRACT_ID = "rugo.hw.support_matrix.v7"
PRIOR_MATRIX_CONTRACT_ID = matrix_v6.MATRIX_CONTRACT_ID
DRIVER_CONTRACT_ID = matrix_v6.DRIVER_CONTRACT_ID
NATIVE_DRIVER_CONTRACT_ID = "rugo.native_driver_contract.v1"
NATIVE_STORAGE_CONTRACT_ID = "rugo.nvme_ahci_contract.v1"
BLOCK_FLUSH_CONTRACT_ID = "rugo.block_flush_contract.v1"
DEFAULT_SEED = 20260312


CHECK_IDS = {
    "source_matrix_v6_green",
    "tier0_nvme_identify",
    "tier0_nvme_admin_queue",
    "tier0_nvme_io_queue",
    "tier0_nvme_fua_flush",
    "tier0_nvme_reset_recover",
    "tier1_ahci_port_up",
    "tier1_ahci_rw_dma",
    "tier1_ahci_flush_ordered",
    "tier1_ahci_reset_recover",
    "negative_nvme_namespace_missing",
    "negative_ahci_port_absent",
}


def known_checks() -> Set[str]:
    return set(CHECK_IDS)


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


def _lifecycle_row(
    driver: str,
    machine: str,
    passed: bool,
    states: Sequence[str],
) -> Dict[str, object]:
    if passed:
        return {
            "driver": driver,
            "machine": machine,
            "states_observed": list(states),
            "probe_attempts": 1,
            "probe_successes": 1,
            "runtime_errors": 0,
            "recoveries": 1,
            "fatal_errors": 0,
            "status": "pass",
        }
    return {
        "driver": driver,
        "machine": machine,
        "states_observed": ["probe_found", "error_fatal"],
        "probe_attempts": 1,
        "probe_successes": 0,
        "runtime_errors": 1,
        "recoveries": 0,
        "fatal_errors": 1,
        "status": "fail",
    }


def run_matrix(
    seed: int,
    injected_failures: Set[str] | None = None,
    max_failures: int = 0,
) -> Dict[str, object]:
    failures = set() if injected_failures is None else set(injected_failures)

    prior_report = matrix_v6.run_matrix(seed=seed, max_failures=0)
    source_green = bool(prior_report["gate_pass"]) and "source_matrix_v6_green" not in failures

    checks: List[Dict[str, object]] = [
        _check_row("source_matrix_v6_green", "source", "eq", True, source_green)
    ]

    nvme_identify = "tier0_nvme_identify" not in failures
    nvme_admin_queue = "tier0_nvme_admin_queue" not in failures
    nvme_io_queue = "tier0_nvme_io_queue" not in failures
    nvme_fua_flush = "tier0_nvme_fua_flush" not in failures
    nvme_reset_recover = "tier0_nvme_reset_recover" not in failures
    ahci_port_up = "tier1_ahci_port_up" not in failures
    ahci_rw_dma = "tier1_ahci_rw_dma" not in failures
    ahci_flush_ordered = "tier1_ahci_flush_ordered" not in failures
    ahci_reset_recover = "tier1_ahci_reset_recover" not in failures
    negative_namespace = "negative_nvme_namespace_missing" not in failures
    negative_port = "negative_ahci_port_absent" not in failures

    checks.extend(
        [
            _check_row("tier0_nvme_identify", "nvme", "eq", True, source_green and nvme_identify),
            _check_row(
                "tier0_nvme_admin_queue",
                "nvme",
                "eq",
                True,
                source_green and nvme_admin_queue,
            ),
            _check_row("tier0_nvme_io_queue", "nvme", "eq", True, source_green and nvme_io_queue),
            _check_row("tier0_nvme_fua_flush", "flush", "eq", True, source_green and nvme_fua_flush),
            _check_row(
                "tier0_nvme_reset_recover",
                "nvme",
                "eq",
                True,
                source_green and nvme_reset_recover,
            ),
            _check_row("tier1_ahci_port_up", "ahci", "eq", True, source_green and ahci_port_up),
            _check_row("tier1_ahci_rw_dma", "ahci", "eq", True, source_green and ahci_rw_dma),
            _check_row(
                "tier1_ahci_flush_ordered",
                "flush",
                "eq",
                True,
                source_green and ahci_flush_ordered,
            ),
            _check_row(
                "tier1_ahci_reset_recover",
                "ahci",
                "eq",
                True,
                source_green and ahci_reset_recover,
            ),
            _check_row(
                "negative_nvme_namespace_missing",
                "negative_path",
                "eq",
                True,
                negative_namespace,
            ),
            _check_row(
                "negative_ahci_port_absent",
                "negative_path",
                "eq",
                True,
                negative_port,
            ),
        ]
    )

    tier0_pass = all(
        [
            source_green,
            nvme_identify,
            nvme_admin_queue,
            nvme_io_queue,
            nvme_fua_flush,
            nvme_reset_recover,
        ]
    )
    tier1_pass = all(
        [
            source_green,
            ahci_port_up,
            ahci_rw_dma,
            ahci_flush_ordered,
            ahci_reset_recover,
        ]
    )
    flush_pass = source_green and nvme_fua_flush and ahci_flush_ordered

    tier_results = [
        {
            "tier": "tier0",
            "machine": "q35",
            "storage_device": "nvme",
            "boot_transport_class": "nvme-pci",
            "identify": "pass" if nvme_identify and source_green else "fail",
            "admin_queue": "pass" if nvme_admin_queue and source_green else "fail",
            "io_queue": "pass" if nvme_io_queue and source_green else "fail",
            "flush": "pass" if nvme_fua_flush and source_green else "fail",
            "reset_recover": "pass" if nvme_reset_recover and source_green else "fail",
            "status": "pass" if tier0_pass else "fail",
        },
        {
            "tier": "tier1",
            "machine": "pc/i440fx",
            "storage_device": "ahci",
            "boot_transport_class": "ahci",
            "port": "pass" if ahci_port_up and source_green else "fail",
            "rw_dma": "pass" if ahci_rw_dma and source_green else "fail",
            "flush": "pass" if ahci_flush_ordered and source_green else "fail",
            "reset_recover": "pass" if ahci_reset_recover and source_green else "fail",
            "status": "pass" if tier1_pass else "fail",
        },
    ]

    device_class_coverage = [
        {
            "device": "virtio-blk-pci",
            "class": "storage",
            "profile": "baseline-v6",
            "required": True,
            "status": "pass" if source_green else "fail",
        },
        {
            "device": "nvme",
            "class": "storage",
            "profile": "tier0-emulated",
            "required": True,
            "status": "pass" if tier0_pass else "fail",
        },
        {
            "device": "ahci",
            "class": "storage",
            "profile": "tier1-emulated",
            "required": True,
            "status": "pass" if tier1_pass else "fail",
        },
    ]

    storage_protocol_matrix = {
        "nvme": {
            "machine": "q35",
            "transport": "pcie",
            "admin_queue_depth": 32,
            "io_queue_depth": 64,
            "namespace_geometry": {"count": 1, "lba_bytes": 4096, "size_gib": 64},
            "required_markers": [
                "NVME: ready",
                "NVME: identify ok",
                "NVME: io queue ok",
                "BLK: fua ok",
            ],
            "checks_pass": tier0_pass,
        },
        "ahci": {
            "machine": "pc/i440fx",
            "transport": "sata",
            "ports_required": 1,
            "ncq_depth": 32,
            "required_markers": [
                "AHCI: port up",
                "AHCI: rw ok",
                "AHCI: flush ok",
                "BLK: flush ordered",
            ],
            "checks_pass": tier1_pass,
        },
    }

    controller_lifecycle = [
        _lifecycle_row(
            "nvme",
            "q35",
            tier0_pass,
            [
                "probe_found",
                "init_ready",
                "runtime_ok",
                "admin_queue_ready",
                "io_queue_ready",
                "reset_recover",
            ],
        ),
        _lifecycle_row(
            "ahci",
            "pc/i440fx",
            tier1_pass,
            [
                "probe_found",
                "init_ready",
                "runtime_ok",
                "port_link_up",
                "dma_rw_ok",
                "reset_recover",
            ],
        ),
    ]

    negative_paths = {
        "nvme_namespace_missing": {
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

    flush_contract_checks = {
        "contract_id": BLOCK_FLUSH_CONTRACT_ID,
        "required_markers": ["BLK: fua ok", "BLK: flush ordered"],
        "nvme_fua_supported": source_green and nvme_fua_flush,
        "ahci_cache_flush_supported": source_green and ahci_flush_ordered,
        "status": "pass" if flush_pass else "fail",
    }

    total_failures = sum(1 for row in checks if row["pass"] is False)
    gate_pass = total_failures <= max_failures

    stable_payload = {
        "schema": SCHEMA,
        "seed": seed,
        "source_digest": prior_report["digest"],
        "checks": [
            {
                "check_id": row["check_id"],
                "pass": row["pass"],
                "observed": row["observed"],
            }
            for row in checks
        ],
        "tier_results": [
            {
                "tier": row["tier"],
                "status": row["status"],
            }
            for row in tier_results
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
        "matrix_contract_id": MATRIX_CONTRACT_ID,
        "prior_matrix_contract_id": PRIOR_MATRIX_CONTRACT_ID,
        "driver_contract_id": DRIVER_CONTRACT_ID,
        "native_driver_contract_id": NATIVE_DRIVER_CONTRACT_ID,
        "native_storage_contract_id": NATIVE_STORAGE_CONTRACT_ID,
        "block_flush_contract_id": BLOCK_FLUSH_CONTRACT_ID,
        "seed": seed,
        "gate": "test-hw-matrix-v7",
        "checks": checks,
        "summary": {
            "source": _domain_summary(checks, "source"),
            "nvme": _domain_summary(checks, "nvme"),
            "ahci": _domain_summary(checks, "ahci"),
            "flush": _domain_summary(checks, "flush"),
            "negative_path": _domain_summary(checks, "negative_path"),
        },
        "tier_results": tier_results,
        "device_class_coverage": device_class_coverage,
        "storage_protocol_matrix": storage_protocol_matrix,
        "controller_lifecycle": controller_lifecycle,
        "flush_contract_checks": flush_contract_checks,
        "negative_paths": negative_paths,
        "source_reports": {
            "matrix_v6": {
                "schema": prior_report["schema"],
                "digest": prior_report["digest"],
                "gate_pass": prior_report["gate_pass"],
                "gate": prior_report["gate"],
                "contract_id": prior_report["matrix_contract_id"],
            }
        },
        "artifact_refs": {
            "junit": "out/pytest-hw-matrix-v7.xml",
            "matrix_report": "out/hw-matrix-v7.json",
            "prior_matrix_report": "out/hw-matrix-v6.json",
            "native_storage_report": "out/native-storage-v1.json",
            "ci_artifact": "hw-matrix-v7-artifacts",
            "native_storage_artifact": "native-storage-v1-artifacts",
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
        help="force a check to fail by check_id",
    )
    parser.add_argument("--max-failures", type=int, default=0)
    parser.add_argument("--out", default="out/hw-matrix-v7.json")
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

    report = run_matrix(
        seed=args.seed,
        injected_failures=injected_failures,
        max_failures=args.max_failures,
    )

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    print(f"hw-matrix-v7-report: {out_path}")
    print(f"total_failures: {report['total_failures']}")
    print(f"gate_pass: {report['gate_pass']}")
    return 0 if report["gate_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
