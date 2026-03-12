#!/usr/bin/env python3
"""Run deterministic native-driver diagnostics checks for M53."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
from typing import Dict, List, Sequence, Set

import run_baremetal_io_baseline_v1 as baremetal_io
import run_hw_matrix_v6 as hw_matrix


SCHEMA = "rugo.native_driver_diagnostics_report.v1"
CONTRACT_ID = "rugo.native_driver_contract.v1"
PCIE_DMA_CONTRACT_ID = "rugo.pcie_dma_contract.v1"
FIRMWARE_BLOB_POLICY_ID = "rugo.firmware_blob_policy.v1"
DIAG_SCHEMA_ID = "rugo.native_driver_diag_schema.v1"
FIRMWARE_MANIFEST_SCHEMA = "rugo.firmware_manifest.v1"
DEFAULT_SEED = 20260311


@dataclass(frozen=True)
class BindingSpec:
    check_id: str
    source: str
    driver: str
    profile: str
    device_class: str
    dma_window_bytes: int


BINDING_SPECS: Sequence[BindingSpec] = (
    BindingSpec(
        check_id="bind_virtio_blk_modern",
        source="matrix_v6",
        driver="virtio-blk-pci",
        profile="modern",
        device_class="storage",
        dma_window_bytes=1_048_576,
    ),
    BindingSpec(
        check_id="bind_virtio_net_modern",
        source="matrix_v6",
        driver="virtio-net-pci",
        profile="modern",
        device_class="network",
        dma_window_bytes=1_048_576,
    ),
    BindingSpec(
        check_id="bind_virtio_scsi",
        source="matrix_v6",
        driver="virtio-scsi-pci",
        profile="modern",
        device_class="storage",
        dma_window_bytes=2_097_152,
    ),
    BindingSpec(
        check_id="bind_virtio_gpu",
        source="matrix_v6",
        driver="virtio-gpu-pci",
        profile="modern",
        device_class="display",
        dma_window_bytes=4_194_304,
    ),
    BindingSpec(
        check_id="bind_e1000e",
        source="baremetal_io",
        driver="e1000e",
        profile="baremetal",
        device_class="network",
        dma_window_bytes=524_288,
    ),
    BindingSpec(
        check_id="bind_rtl8169",
        source="baremetal_io",
        driver="rtl8169",
        profile="baremetal",
        device_class="network",
        dma_window_bytes=524_288,
    ),
    BindingSpec(
        check_id="bind_xhci",
        source="baremetal_io",
        driver="xhci",
        profile="baremetal",
        device_class="usb-host",
        dma_window_bytes=1_048_576,
    ),
    BindingSpec(
        check_id="bind_usb_storage",
        source="baremetal_io",
        driver="usb-storage",
        profile="baremetal",
        device_class="removable",
        dma_window_bytes=1_048_576,
    ),
)


POLICY_CHECKS = {
    "source_matrix_green",
    "source_baremetal_green",
    "irq_vector_policy",
    "dma_map_ok",
    "dma_bounce_window",
    "dma_unsafe_denied",
    "iommu_strict_mode",
    "firmware_signed_allowed",
    "firmware_unsigned_denied",
    "firmware_missing_manifest_denied",
    "firmware_hash_mismatch_denied",
}


def known_checks() -> Set[str]:
    return {spec.check_id for spec in BINDING_SPECS} | POLICY_CHECKS


def _noise(seed: int, key: str) -> int:
    digest = hashlib.sha256(f"{seed}|{key}".encode("utf-8")).hexdigest()
    return int(digest[:8], 16)


def _metric(seed: int, key: str, base: int, spread: int) -> int:
    return base + (_noise(seed, key) % spread)


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


def _driver_row(report: dict, driver: str, profile: str) -> dict:
    rows = [
        entry
        for entry in report["driver_lifecycle"]
        if entry["driver"] == driver and entry["profile"] == profile
    ]
    assert len(rows) == 1
    return rows[0]


def _event(
    event_id: str,
    driver: str,
    device_class: str,
    profile: str,
    phase: str,
    severity: str,
    marker: str,
    status: str,
    details: Dict[str, object],
) -> Dict[str, object]:
    return {
        "event_id": event_id,
        "driver": driver,
        "device_class": device_class,
        "profile": profile,
        "phase": phase,
        "severity": severity,
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

    matrix_report = hw_matrix.run_matrix(seed=seed, max_failures=0)
    baremetal_report = baremetal_io.run_baseline(seed=seed, max_failures=0)

    source_reports = {
        "matrix_v6": {
            "schema": matrix_report["schema"],
            "digest": matrix_report["digest"],
            "gate_pass": matrix_report["gate_pass"],
            "gate": matrix_report["gate"],
            "contract_id": matrix_report["matrix_contract_id"],
        },
        "baremetal_io": {
            "schema": baremetal_report["schema"],
            "digest": baremetal_report["digest"],
            "gate_pass": baremetal_report["gate_pass"],
            "gate": baremetal_report["gate"],
            "contract_id": baremetal_report["profile_id"],
        },
    }

    checks: List[Dict[str, object]] = []
    source_matrix_green = bool(matrix_report["gate_pass"]) and "source_matrix_green" not in failures
    source_baremetal_green = (
        bool(baremetal_report["gate_pass"]) and "source_baremetal_green" not in failures
    )
    checks.append(
        _check_row("source_matrix_green", "source", "eq", True, source_matrix_green)
    )
    checks.append(
        _check_row("source_baremetal_green", "source", "eq", True, source_baremetal_green)
    )

    bindings: List[Dict[str, object]] = []
    diagnostic_events: List[Dict[str, object]] = []

    for spec in BINDING_SPECS:
        report = matrix_report if spec.source == "matrix_v6" else baremetal_report
        source_green = source_matrix_green if spec.source == "matrix_v6" else source_baremetal_green
        source_row = _driver_row(report, spec.driver, spec.profile)
        binding_pass = (
            source_green
            and source_row["status"] == "pass"
            and spec.check_id not in failures
        )
        bind_latency_ms = _metric(seed, f"{spec.check_id}|bind", 12, 16)
        irq_vector = _metric(seed, f"{spec.check_id}|irq", 48, 32)
        markers = [
            "DRV: bind",
            "IRQ: vector bound" if binding_pass else "IRQ: vector denied",
            "DMA: map ok" if binding_pass else "DMA: deny unsafe",
        ]
        bindings.append(
            {
                "driver": spec.driver,
                "profile": spec.profile,
                "device_class": spec.device_class,
                "source_schema": report["schema"],
                "source_digest": report["digest"],
                "states_observed": (
                    list(source_row["states_observed"])
                    if binding_pass
                    else ["probe_found", "init_ready", "error_fatal"]
                ),
                "bind_latency_ms": bind_latency_ms,
                "irq_vector": irq_vector if binding_pass else 0,
                "dma_window_bytes": spec.dma_window_bytes,
                "markers": markers,
                "status": "pass" if binding_pass else "fail",
            }
        )
        checks.append(_check_row(spec.check_id, "bind", "eq", True, binding_pass))
        diagnostic_events.append(
            _event(
                event_id=f"bind::{spec.driver}::{spec.profile}",
                driver=spec.driver,
                device_class=spec.device_class,
                profile=spec.profile,
                phase="bind",
                severity="info" if binding_pass else "error",
                marker="DRV: bind",
                status="pass" if binding_pass else "fail",
                details={
                    "bind_latency_ms": bind_latency_ms,
                    "source_schema": report["schema"],
                    "source_digest": report["digest"],
                },
            )
        )

    binding_index = {
        (entry["driver"], entry["profile"]): entry for entry in bindings
    }

    irq_policy_pass = "irq_vector_policy" not in failures
    irq_audits = [
        {
            "driver": "virtio-gpu-pci",
            "profile": "modern",
            "device_class": "display",
            "vector": binding_index[("virtio-gpu-pci", "modern")]["irq_vector"],
            "delivery_mode": "msix",
            "retarget_events": 1 if irq_policy_pass else 0,
            "marker": "IRQ: vector bound" if irq_policy_pass else "IRQ: vector denied",
            "status": "pass" if irq_policy_pass else "fail",
        },
        {
            "driver": "e1000e",
            "profile": "baremetal",
            "device_class": "network",
            "vector": binding_index[("e1000e", "baremetal")]["irq_vector"],
            "delivery_mode": "msi",
            "retarget_events": 1 if irq_policy_pass else 0,
            "marker": "IRQ: vector bound" if irq_policy_pass else "IRQ: vector denied",
            "status": "pass" if irq_policy_pass else "fail",
        },
    ]
    checks.append(_check_row("irq_vector_policy", "irq_dma", "eq", True, irq_policy_pass))
    for row in irq_audits:
        diagnostic_events.append(
            _event(
                event_id=f"irq::{row['driver']}::{row['profile']}",
                driver=row["driver"],
                device_class=row["device_class"],
                profile=row["profile"],
                phase="irq",
                severity="info" if row["status"] == "pass" else "error",
                marker=row["marker"],
                status=row["status"],
                details={
                    "vector": row["vector"],
                    "delivery_mode": row["delivery_mode"],
                    "retarget_events": row["retarget_events"],
                },
            )
        )

    iommu_mode = "strict" if "iommu_strict_mode" not in failures else "passthrough-shadow"
    dma_map_ok_pass = "dma_map_ok" not in failures
    dma_bounce_pass = "dma_bounce_window" not in failures
    dma_unsafe_denied_pass = "dma_unsafe_denied" not in failures
    dma_policy = {
        "contract_id": PCIE_DMA_CONTRACT_ID,
        "iommu_mode": iommu_mode,
        "iommu_domain_id": "iommu/native-driver-floor",
        "max_map_window_bytes": 4_194_304,
        "bounce_buffer_allowed": True,
        "peer_to_peer_dma_allowed": False,
        "map_success_marker": "DMA: map ok",
        "map_bounce_marker": "DMA: map bounce",
        "unsafe_path_marker": "DMA: deny unsafe",
    }
    dma_audits = [
        {
            "audit_id": "nvme_admin_submission_queue",
            "driver": "nvme-shadow",
            "profile": "contract-only",
            "device_class": "storage",
            "map_kind": "streaming",
            "iommu_mode": iommu_mode,
            "dma_window_bytes": 1_048_576,
            "bounce_buffer_used": False,
            "unsafe_reason": "",
            "marker": "DMA: map ok" if dma_map_ok_pass else "DMA: deny unsafe",
            "status": "pass" if dma_map_ok_pass else "fail",
        },
        {
            "audit_id": "gpu_ring_bounce_window",
            "driver": "native-gpu-shadow",
            "profile": "contract-only",
            "device_class": "display",
            "map_kind": "bounce",
            "iommu_mode": iommu_mode,
            "dma_window_bytes": 2_097_152,
            "bounce_buffer_used": True,
            "unsafe_reason": "",
            "marker": "DMA: map bounce" if dma_bounce_pass else "DMA: deny unsafe",
            "status": "pass" if dma_bounce_pass else "fail",
        },
        {
            "audit_id": "wifi_peer_to_peer_denial",
            "driver": "wifi-pcie-shadow",
            "profile": "contract-only",
            "device_class": "network",
            "map_kind": "peer-to-peer",
            "iommu_mode": iommu_mode,
            "dma_window_bytes": 0,
            "bounce_buffer_used": False,
            "unsafe_reason": "peer-to-peer DMA denied",
            "marker": "DMA: deny unsafe" if dma_unsafe_denied_pass else "DMA: map ok",
            "status": "pass" if dma_unsafe_denied_pass else "fail",
        },
    ]
    checks.append(_check_row("dma_map_ok", "irq_dma", "eq", True, dma_map_ok_pass))
    checks.append(_check_row("dma_bounce_window", "irq_dma", "eq", True, dma_bounce_pass))
    checks.append(
        _check_row("dma_unsafe_denied", "irq_dma", "eq", True, dma_unsafe_denied_pass)
    )
    checks.append(
        _check_row("iommu_strict_mode", "irq_dma", "eq", "strict", iommu_mode)
    )
    for row in dma_audits:
        diagnostic_events.append(
            _event(
                event_id=f"dma::{row['audit_id']}",
                driver=row["driver"],
                device_class=row["device_class"],
                profile=row["profile"],
                phase="dma",
                severity="info" if row["status"] == "pass" else "error",
                marker=row["marker"],
                status=row["status"],
                details={
                    "map_kind": row["map_kind"],
                    "iommu_mode": row["iommu_mode"],
                    "dma_window_bytes": row["dma_window_bytes"],
                    "bounce_buffer_used": row["bounce_buffer_used"],
                    "unsafe_reason": row["unsafe_reason"],
                },
            )
        )

    firmware_policy = {
        "policy_id": FIRMWARE_BLOB_POLICY_ID,
        "manifest_schema": FIRMWARE_MANIFEST_SCHEMA,
        "manifest_required": True,
        "signature_required": True,
        "measured_boot_reference_required": True,
        "allowlist_required": True,
        "storage_outside_kernel_image": True,
    }
    firmware_signed_allowed_pass = "firmware_signed_allowed" not in failures
    firmware_unsigned_denied_pass = "firmware_unsigned_denied" not in failures
    firmware_missing_manifest_denied_pass = (
        "firmware_missing_manifest_denied" not in failures
    )
    firmware_hash_mismatch_denied_pass = (
        "firmware_hash_mismatch_denied" not in failures
    )
    firmware_audits = [
        {
            "audit_id": "gpu_guc_signed",
            "driver": "native-gpu-shadow",
            "profile": "contract-only",
            "device_class": "display",
            "blob_id": "intel-guc-shadow",
            "manifest_present": True,
            "signature_valid": True,
            "hash_match": True,
            "measured_boot_ref": "pcr7:intel-guc-shadow",
            "decision": "allow" if firmware_signed_allowed_pass else "deny_unexpected",
            "marker": "FW: allow signed" if firmware_signed_allowed_pass else "FW: denied signed",
            "status": "pass" if firmware_signed_allowed_pass else "fail",
        },
        {
            "audit_id": "wifi_ucode_unsigned",
            "driver": "wifi-pcie-shadow",
            "profile": "contract-only",
            "device_class": "network",
            "blob_id": "wifi-ucode-shadow",
            "manifest_present": True,
            "signature_valid": False,
            "hash_match": True,
            "measured_boot_ref": "pcr7:wifi-ucode-shadow",
            "decision": (
                "denied_unsigned" if firmware_unsigned_denied_pass else "allow_unexpected"
            ),
            "marker": (
                "FW: denied unsigned"
                if firmware_unsigned_denied_pass
                else "FW: allow unsigned"
            ),
            "status": "pass" if firmware_unsigned_denied_pass else "fail",
        },
        {
            "audit_id": "gpu_vbios_missing_manifest",
            "driver": "native-gpu-shadow",
            "profile": "contract-only",
            "device_class": "display",
            "blob_id": "gpu-vbios-shadow",
            "manifest_present": False,
            "signature_valid": False,
            "hash_match": False,
            "measured_boot_ref": "missing",
            "decision": (
                "denied_missing_manifest"
                if firmware_missing_manifest_denied_pass
                else "allow_untracked"
            ),
            "marker": (
                "FW: denied missing manifest"
                if firmware_missing_manifest_denied_pass
                else "FW: allow missing manifest"
            ),
            "status": "pass" if firmware_missing_manifest_denied_pass else "fail",
        },
        {
            "audit_id": "nvme_admin_hash_mismatch",
            "driver": "nvme-shadow",
            "profile": "contract-only",
            "device_class": "storage",
            "blob_id": "nvme-admin-shadow",
            "manifest_present": True,
            "signature_valid": True,
            "hash_match": False,
            "measured_boot_ref": "pcr7:nvme-admin-shadow",
            "decision": (
                "denied_hash_mismatch"
                if firmware_hash_mismatch_denied_pass
                else "allow_hash_mismatch"
            ),
            "marker": (
                "FW: denied hash mismatch"
                if firmware_hash_mismatch_denied_pass
                else "FW: allow hash mismatch"
            ),
            "status": "pass" if firmware_hash_mismatch_denied_pass else "fail",
        },
    ]
    checks.append(
        _check_row(
            "firmware_signed_allowed", "firmware", "eq", True, firmware_signed_allowed_pass
        )
    )
    checks.append(
        _check_row(
            "firmware_unsigned_denied",
            "firmware",
            "eq",
            True,
            firmware_unsigned_denied_pass,
        )
    )
    checks.append(
        _check_row(
            "firmware_missing_manifest_denied",
            "firmware",
            "eq",
            True,
            firmware_missing_manifest_denied_pass,
        )
    )
    checks.append(
        _check_row(
            "firmware_hash_mismatch_denied",
            "firmware",
            "eq",
            True,
            firmware_hash_mismatch_denied_pass,
        )
    )
    for row in firmware_audits:
        diagnostic_events.append(
            _event(
                event_id=f"firmware::{row['audit_id']}",
                driver=row["driver"],
                device_class=row["device_class"],
                profile=row["profile"],
                phase="firmware",
                severity="info" if row["status"] == "pass" else "error",
                marker=row["marker"],
                status=row["status"],
                details={
                    "blob_id": row["blob_id"],
                    "decision": row["decision"],
                    "manifest_present": row["manifest_present"],
                    "signature_valid": row["signature_valid"],
                    "hash_match": row["hash_match"],
                    "measured_boot_ref": row["measured_boot_ref"],
                },
            )
        )

    total_failures = sum(1 for row in checks if row["pass"] is False)
    gate_pass = total_failures <= max_failures

    stable_payload = {
        "schema": SCHEMA,
        "seed": seed,
        "source_reports": {
            "matrix_v6": source_reports["matrix_v6"]["digest"],
            "baremetal_io": source_reports["baremetal_io"]["digest"],
        },
        "checks": [
            {
                "check_id": row["check_id"],
                "pass": row["pass"],
                "observed": row["observed"],
            }
            for row in checks
        ],
        "driver_bindings": [
            {
                "driver": row["driver"],
                "profile": row["profile"],
                "status": row["status"],
            }
            for row in bindings
        ],
        "dma_audits": [
            {
                "audit_id": row["audit_id"],
                "status": row["status"],
                "marker": row["marker"],
            }
            for row in dma_audits
        ],
        "firmware_audits": [
            {
                "audit_id": row["audit_id"],
                "status": row["status"],
                "marker": row["marker"],
            }
            for row in firmware_audits
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
        "pcie_dma_contract_id": PCIE_DMA_CONTRACT_ID,
        "firmware_blob_policy_id": FIRMWARE_BLOB_POLICY_ID,
        "diag_schema_id": DIAG_SCHEMA_ID,
        "driver_lifecycle_contract_id": hw_matrix.DRIVER_CONTRACT_ID,
        "support_matrix_id": hw_matrix.MATRIX_CONTRACT_ID,
        "seed": seed,
        "gate": "test-native-driver-diagnostics-v1",
        "contract_gate": "test-native-driver-contract-v1",
        "checks": checks,
        "summary": {
            "source": _domain_summary(checks, "source"),
            "bind": _domain_summary(checks, "bind"),
            "irq_dma": _domain_summary(checks, "irq_dma"),
            "firmware": _domain_summary(checks, "firmware"),
            "events": {
                "count": len(diagnostic_events),
                "marker_set": sorted({row["marker"] for row in diagnostic_events}),
                "pass": True,
            },
        },
        "driver_bindings": bindings,
        "irq_audits": irq_audits,
        "dma_policy": dma_policy,
        "dma_audits": dma_audits,
        "firmware_policy": firmware_policy,
        "firmware_audits": firmware_audits,
        "diagnostic_events": diagnostic_events,
        "source_reports": source_reports,
        "artifact_refs": {
            "junit": "out/pytest-native-driver-diagnostics-v1.xml",
            "diagnostics_report": "out/native-driver-diagnostics-v1.json",
            "matrix_report": "out/hw-matrix-v6.json",
            "baremetal_io_report": "out/baremetal-io-v1.json",
            "ci_artifact": "native-driver-diagnostics-v1-artifacts",
            "contract_ci_artifact": "native-driver-contract-v1-artifacts",
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
    parser.add_argument("--out", default="out/native-driver-diagnostics-v1.json")
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

    print(f"native-driver-diagnostics-report: {out_path}")
    print(f"total_failures: {report['total_failures']}")
    print(f"gate_pass: {report['gate_pass']}")
    return 0 if report["gate_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
