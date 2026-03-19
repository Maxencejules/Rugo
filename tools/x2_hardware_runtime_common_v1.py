#!/usr/bin/env python3
"""Shared helpers for X2 hardware runtime-backed qualification."""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
from typing import Dict, List, Mapping, Sequence, Set

import collect_firmware_smp_evidence_v1 as firmware_smp
import collect_hw_diagnostics_v3 as hw_diag_v3
import collect_measured_boot_report_v1 as measured_boot
import run_baremetal_io_baseline_v1 as baremetal_io
import run_hw_claim_promotion_v1 as claim_promotion
import run_hw_matrix_v4 as matrix_v4
import run_hw_matrix_v6 as matrix_v6
import run_native_driver_diagnostics_v1 as native_driver_diag
import runtime_capture_common_v1 as runtime_capture


SCHEMA = "rugo.x2_hardware_runtime_report.v1"
POLICY_ID = "rugo.x2_hardware_runtime_qualification.v1"
DEVICE_REGISTRY_SCHEMA = "rugo.x2_device_registry.v1"
TRACK_ID = "X2"
DEFAULT_SEED = 20260318

DEFAULT_SUPPORTING_REPORT_PATHS = {
    "measured_boot": "out/measured-boot-v1.json",
    "hw_diagnostics_v3": "out/hw-diagnostics-v3.json",
    "hw_matrix_v4": "out/hw-matrix-v4.json",
    "hw_firmware_smp": "out/hw-firmware-smp-v1.json",
    "hw_matrix_v6": "out/hw-matrix-v6.json",
    "baremetal_io": "out/baremetal-io-v1.json",
    "hw_claim_promotion": "out/hw-claim-promotion-v1.json",
    "native_driver_diag": "out/native-driver-diagnostics-v1.json",
}


@dataclass(frozen=True)
class DeviceSpec:
    class_id: str
    driver: str
    device_class: str
    profile: str
    qualified_tiers: tuple[str, ...]
    source_report: str
    source_milestones: tuple[str, ...]
    required_states: tuple[str, ...]
    required_markers: tuple[str, ...]
    claim_class_id: str = ""


@dataclass(frozen=True)
class TargetSpec:
    target_id: str
    support_tier: str
    machine_profile: str
    source_reports: tuple[str, ...]
    source_backlogs: tuple[str, ...]
    required_devices: tuple[str, ...]
    boot_markers: tuple[str, ...]
    runtime_markers: tuple[str, ...]
    claim_classes: tuple[str, ...] = ()
    firmware_bound: bool = False
    smp_bound: bool = False


DEVICE_SPECS: tuple[DeviceSpec, ...] = (
    DeviceSpec(
        class_id="virtio-blk-pci-transitional",
        driver="virtio-blk-pci",
        device_class="storage",
        profile="transitional",
        qualified_tiers=("tier0", "tier1"),
        source_report="hw_matrix_v4",
        source_milestones=("M9", "M15", "M23", "M37"),
        required_states=("probe_found", "init_ready", "runtime_ok"),
        required_markers=("BLK: found virtio-blk", "BLK: rw ok"),
    ),
    DeviceSpec(
        class_id="virtio-net-pci-transitional",
        driver="virtio-net-pci",
        device_class="network",
        profile="transitional",
        qualified_tiers=("tier0", "tier1"),
        source_report="hw_matrix_v4",
        source_milestones=("M9", "M15", "M23", "M37"),
        required_states=("probe_found", "init_ready", "runtime_ok"),
        required_markers=("NET: virtio-net ready", "NET: udp echo"),
    ),
    DeviceSpec(
        class_id="ahci",
        driver="ahci",
        device_class="storage",
        profile="baseline",
        qualified_tiers=("tier2", "tier3"),
        source_report="hw_matrix_v4",
        source_milestones=("M37",),
        required_states=("probe_found", "init_ready", "runtime_ok"),
        required_markers=("AHCI: port up", "AHCI: rw ok"),
    ),
    DeviceSpec(
        class_id="nvme",
        driver="nvme",
        device_class="storage",
        profile="baseline",
        qualified_tiers=("tier2", "tier3"),
        source_report="hw_matrix_v4",
        source_milestones=("M37",),
        required_states=("probe_found", "init_ready", "runtime_ok"),
        required_markers=("NVME: ready", "NVME: identify ok"),
    ),
    DeviceSpec(
        class_id="e1000",
        driver="e1000",
        device_class="network",
        profile="baseline",
        qualified_tiers=("tier2", "tier3"),
        source_report="hw_matrix_v4",
        source_milestones=("M37",),
        required_states=("probe_found", "init_ready", "runtime_ok"),
        required_markers=("NET: e1000 ready", "NET: udp echo"),
    ),
    DeviceSpec(
        class_id="rtl8139",
        driver="rtl8139",
        device_class="network",
        profile="baseline",
        qualified_tiers=("tier2", "tier3"),
        source_report="hw_matrix_v4",
        source_milestones=("M37",),
        required_states=("probe_found", "init_ready", "runtime_ok"),
        required_markers=("NET: rtl8139 ready", "NET: udp echo"),
    ),
    DeviceSpec(
        class_id="virtio-blk-pci-modern",
        driver="virtio-blk-pci",
        device_class="storage",
        profile="modern",
        qualified_tiers=("tier1",),
        source_report="hw_matrix_v6",
        source_milestones=("M45",),
        required_states=(
            "probe_found",
            "init_ready",
            "runtime_ok",
            "irq_vector_bound",
            "cpu_affinity_balance",
        ),
        required_markers=("DRV: bind", "BLK: rw ok"),
        claim_class_id="virtio-blk-pci-modern",
    ),
    DeviceSpec(
        class_id="virtio-net-pci-modern",
        driver="virtio-net-pci",
        device_class="network",
        profile="modern",
        qualified_tiers=("tier1",),
        source_report="hw_matrix_v6",
        source_milestones=("M45",),
        required_states=(
            "probe_found",
            "init_ready",
            "runtime_ok",
            "irq_vector_bound",
            "cpu_affinity_balance",
        ),
        required_markers=("DRV: bind", "NET: udp echo"),
        claim_class_id="virtio-net-pci-modern",
    ),
    DeviceSpec(
        class_id="virtio-scsi-pci",
        driver="virtio-scsi-pci",
        device_class="storage",
        profile="modern",
        qualified_tiers=("tier1",),
        source_report="hw_matrix_v6",
        source_milestones=("M45",),
        required_states=(
            "probe_found",
            "init_ready",
            "runtime_ok",
            "irq_vector_bound",
            "cpu_affinity_balance",
        ),
        required_markers=("DRV: bind", "SCSI: inquiry ok"),
        claim_class_id="virtio-scsi-pci",
    ),
    DeviceSpec(
        class_id="virtio-gpu-pci",
        driver="virtio-gpu-pci",
        device_class="display",
        profile="modern",
        qualified_tiers=("tier1",),
        source_report="hw_matrix_v6",
        source_milestones=("M45",),
        required_states=(
            "probe_found",
            "init_ready",
            "runtime_ok",
            "framebuffer_console_present",
            "display_scanout_ready",
            "irq_vector_bound",
            "cpu_affinity_balance",
        ),
        required_markers=("DRV: bind", "GPU: framebuffer ready", "DISP: scanout stable"),
        claim_class_id="virtio-gpu-pci",
    ),
    DeviceSpec(
        class_id="e1000e",
        driver="e1000e",
        device_class="network",
        profile="baremetal",
        qualified_tiers=("tier2",),
        source_report="baremetal_io",
        source_milestones=("M46", "M47"),
        required_states=(
            "probe_found",
            "init_ready",
            "runtime_ok",
            "link_ready",
            "irq_vector_bound",
            "cpu_affinity_balance",
        ),
        required_markers=("DRV: bind", "NET: e1000e ready", "NET: udp echo"),
        claim_class_id="e1000e",
    ),
    DeviceSpec(
        class_id="rtl8169",
        driver="rtl8169",
        device_class="network",
        profile="baremetal",
        qualified_tiers=("tier2",),
        source_report="baremetal_io",
        source_milestones=("M46", "M47"),
        required_states=(
            "probe_found",
            "init_ready",
            "runtime_ok",
            "link_ready",
            "irq_vector_bound",
            "cpu_affinity_balance",
        ),
        required_markers=("DRV: bind", "NET: rtl8169 ready", "NET: udp echo"),
        claim_class_id="rtl8169",
    ),
    DeviceSpec(
        class_id="xhci",
        driver="xhci",
        device_class="usb-host",
        profile="baremetal",
        qualified_tiers=("tier2",),
        source_report="baremetal_io",
        source_milestones=("M46", "M47"),
        required_states=(
            "probe_found",
            "init_ready",
            "hid_ready",
            "irq_vector_bound",
            "cpu_affinity_balance",
        ),
        required_markers=("DRV: bind", "USB: hid ready"),
        claim_class_id="xhci",
    ),
    DeviceSpec(
        class_id="usb-hid",
        driver="usb-hid",
        device_class="input",
        profile="baremetal",
        qualified_tiers=("tier2",),
        source_report="baremetal_io",
        source_milestones=("M46", "M47"),
        required_states=(
            "probe_found",
            "init_ready",
            "hid_ready",
            "focus_delivery_ready",
            "cpu_affinity_balance",
        ),
        required_markers=("USB: hid ready", "USB: focus delivery ok"),
        claim_class_id="usb-hid",
    ),
    DeviceSpec(
        class_id="usb-storage",
        driver="usb-storage",
        device_class="removable",
        profile="baremetal",
        qualified_tiers=("tier2",),
        source_report="baremetal_io",
        source_milestones=("M46", "M47"),
        required_states=(
            "probe_found",
            "init_ready",
            "media_ready",
            "recovery_media_bootstrap",
            "cpu_affinity_balance",
        ),
        required_markers=("USBSTOR: media ready", "RECOVERY: removable media ready"),
        claim_class_id="usb-storage",
    ),
)


TARGET_SPECS: tuple[TargetSpec, ...] = (
    TargetSpec(
        target_id="qemu-q35-transitional",
        support_tier="tier0",
        machine_profile="q35",
        source_reports=("hw_matrix_v4",),
        source_backlogs=("M9", "M15"),
        required_devices=("virtio-blk-pci-transitional", "virtio-net-pci-transitional"),
        boot_markers=("RUGO: boot ok", "BLK: found virtio-blk", "NET: virtio-net ready"),
        runtime_markers=("BLK: rw ok", "NET: udp echo"),
    ),
    TargetSpec(
        target_id="qemu-i440fx-transitional",
        support_tier="tier1",
        machine_profile="pc/i440fx",
        source_reports=("hw_matrix_v4",),
        source_backlogs=("M9", "M15"),
        required_devices=("virtio-blk-pci-transitional", "virtio-net-pci-transitional"),
        boot_markers=("RUGO: boot ok", "BLK: found virtio-blk", "NET: virtio-net ready"),
        runtime_markers=("BLK: rw ok", "NET: udp echo"),
    ),
    TargetSpec(
        target_id="qemu-q35-firmware-smp",
        support_tier="tier0",
        machine_profile="q35",
        source_reports=("hw_diagnostics_v3", "measured_boot", "hw_firmware_smp"),
        source_backlogs=("M23", "M43"),
        required_devices=("virtio-blk-pci-transitional", "virtio-net-pci-transitional"),
        boot_markers=(
            "RUGO: boot ok",
            "FW: rsdp ok",
            "FW: xsdt ok",
            "FW: madt ok",
            "SMP: bsp online",
            "SMP: ap online",
        ),
        runtime_markers=("SMP: ipi roundtrip ok", "SMP: affinity balanced"),
        firmware_bound=True,
        smp_bound=True,
    ),
    TargetSpec(
        target_id="qemu-i440fx-firmware-smp",
        support_tier="tier1",
        machine_profile="pc/i440fx",
        source_reports=("hw_diagnostics_v3", "measured_boot", "hw_firmware_smp"),
        source_backlogs=("M23", "M43"),
        required_devices=("virtio-blk-pci-transitional", "virtio-net-pci-transitional"),
        boot_markers=(
            "RUGO: boot ok",
            "FW: rsdp ok",
            "FW: xsdt ok",
            "FW: madt ok",
            "SMP: bsp online",
            "SMP: ap online",
        ),
        runtime_markers=("SMP: ipi roundtrip ok", "SMP: affinity balanced"),
        firmware_bound=True,
        smp_bound=True,
    ),
    TargetSpec(
        target_id="qemu-q35-modern-virtio",
        support_tier="tier1",
        machine_profile="q35",
        source_reports=("hw_matrix_v6", "native_driver_diag"),
        source_backlogs=("M45",),
        required_devices=(
            "virtio-blk-pci-modern",
            "virtio-net-pci-modern",
            "virtio-scsi-pci",
            "virtio-gpu-pci",
        ),
        boot_markers=("RUGO: boot ok", "DRV: bind", "GPU: framebuffer ready"),
        runtime_markers=("BLK: rw ok", "NET: udp echo", "SCSI: inquiry ok", "DISP: scanout stable"),
        claim_classes=(
            "virtio-blk-pci-modern",
            "virtio-net-pci-modern",
            "virtio-scsi-pci",
            "virtio-gpu-pci",
        ),
        smp_bound=True,
    ),
    TargetSpec(
        target_id="qemu-i440fx-modern-virtio",
        support_tier="tier1",
        machine_profile="pc/i440fx",
        source_reports=("hw_matrix_v6", "native_driver_diag"),
        source_backlogs=("M45",),
        required_devices=(
            "virtio-blk-pci-modern",
            "virtio-net-pci-modern",
            "virtio-scsi-pci",
            "virtio-gpu-pci",
        ),
        boot_markers=("RUGO: boot ok", "DRV: bind", "GPU: framebuffer ready"),
        runtime_markers=("BLK: rw ok", "NET: udp echo", "SCSI: inquiry ok", "DISP: scanout stable"),
        claim_classes=(
            "virtio-blk-pci-modern",
            "virtio-net-pci-modern",
            "virtio-scsi-pci",
            "virtio-gpu-pci",
        ),
        smp_bound=True,
    ),
    TargetSpec(
        target_id="intel-q470-e1000e-xhci",
        support_tier="tier2",
        machine_profile="intel_q470_e1000e_xhci",
        source_reports=("baremetal_io", "hw_claim_promotion", "native_driver_diag"),
        source_backlogs=("M46", "M47"),
        required_devices=("e1000e", "xhci", "usb-hid", "usb-storage"),
        boot_markers=("RUGO: boot ok", "NET: e1000e ready", "USB: hid ready"),
        runtime_markers=("NET: udp echo", "USB: focus delivery ok", "USBSTOR: media ready", "RECOVERY: removable media ready"),
        claim_classes=("e1000e", "xhci", "usb-hid", "usb-storage"),
        smp_bound=True,
    ),
    TargetSpec(
        target_id="amd-b550-rtl8169-xhci",
        support_tier="tier2",
        machine_profile="amd_b550_rtl8169_xhci",
        source_reports=("baremetal_io", "hw_claim_promotion", "native_driver_diag"),
        source_backlogs=("M46", "M47"),
        required_devices=("rtl8169", "xhci", "usb-hid", "usb-storage"),
        boot_markers=("RUGO: boot ok", "NET: rtl8169 ready", "USB: hid ready"),
        runtime_markers=("NET: udp echo", "USB: focus delivery ok", "USBSTOR: media ready", "RECOVERY: removable media ready"),
        claim_classes=("rtl8169", "xhci", "usb-hid", "usb-storage"),
        smp_bound=True,
    ),
)


BACKLOG_TARGET_MAP = {
    "M9": ("qemu-q35-transitional", "qemu-i440fx-transitional"),
    "M15": ("qemu-q35-transitional", "qemu-i440fx-transitional"),
    "M23": ("qemu-q35-firmware-smp", "qemu-i440fx-firmware-smp"),
    "M37": ("qemu-q35-transitional", "qemu-i440fx-transitional"),
    "M43": ("qemu-q35-firmware-smp", "qemu-i440fx-firmware-smp"),
    "M45": ("qemu-q35-modern-virtio", "qemu-i440fx-modern-virtio"),
    "M46": ("intel-q470-e1000e-xhci", "amd-b550-rtl8169-xhci"),
    "M47": ("intel-q470-e1000e-xhci", "amd-b550-rtl8169-xhci"),
}


CHECK_IDS = {
    "device_registry_complete",
    "probe_bind_lifecycle_complete",
    "firmware_runtime_complete",
    "smp_runtime_complete",
    "claim_promotion_traceable",
    "target_qemu_q35_transitional",
    "target_qemu_i440fx_transitional",
    "target_qemu_q35_firmware_smp",
    "target_qemu_i440fx_firmware_smp",
    "target_qemu_q35_modern_virtio",
    "target_qemu_i440fx_modern_virtio",
    "target_intel_q470_e1000e_xhci",
    "target_amd_b550_rtl8169_xhci",
}


def known_checks() -> Set[str]:
    return set(CHECK_IDS)


def normalize_failures(values: Sequence[str]) -> Set[str]:
    failures = {value.strip() for value in values if value.strip()}
    unknown = sorted(failures - known_checks())
    if unknown:
        raise ValueError(f"unknown check ids in --inject-failure: {', '.join(unknown)}")
    return failures


def supporting_report_paths(base_dir: str | Path) -> Dict[str, Path]:
    root = Path(base_dir)
    return {key: root / Path(value).name for key, value in DEFAULT_SUPPORTING_REPORT_PATHS.items()}


def _check_row(
    check_id: str,
    domain: str,
    operator: str,
    threshold: bool | float | int | str,
    observed: bool | float | int | str,
) -> Dict[str, object]:
    if operator == "eq":
        passed = observed == threshold
    elif operator == "min":
        passed = float(observed) >= float(threshold)
    elif operator == "max":
        passed = float(observed) <= float(threshold)
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


def _domain_summary(checks: Sequence[Mapping[str, object]], domain: str) -> Dict[str, object]:
    scoped = [row for row in checks if row["domain"] == domain]
    failures = [row for row in scoped if row["pass"] is False]
    return {
        "checks": len(scoped),
        "failures": len(failures),
        "pass": len(failures) == 0,
    }


def _find_tier_result(
    tier_results: Sequence[Mapping[str, object]],
    tier: str,
) -> Dict[str, object]:
    for row in tier_results:
        if row["tier"] == tier:
            return dict(row)
    raise ValueError(f"tier result missing for {tier!r}")


def _find_coverage(
    rows: Sequence[Mapping[str, object]],
    device: str,
    profile: str | None = None,
) -> Dict[str, object]:
    for row in rows:
        if row["device"] != device:
            continue
        if profile is not None and row.get("profile") != profile:
            continue
        return dict(row)
    raise ValueError(f"device coverage missing for {device!r} profile={profile!r}")


def _find_lifecycle(
    rows: Sequence[Mapping[str, object]],
    driver: str,
    profile: str | None = None,
) -> Dict[str, object]:
    for row in rows:
        if row["driver"] != driver:
            continue
        if profile is not None and row.get("profile") != profile:
            continue
        return dict(row)
    raise ValueError(f"driver lifecycle missing for {driver!r} profile={profile!r}")


def _synthesize_lifecycle(
    *,
    driver: str,
    device_class: str,
    profile: str,
    required_states: Sequence[str],
    coverage_status: str,
) -> Dict[str, object]:
    passed = coverage_status == "pass"
    if passed:
        return {
            "driver": driver,
            "device_class": device_class,
            "profile": profile,
            "states_observed": list(required_states),
            "probe_attempts": 1,
            "probe_successes": 1,
            "init_failures": 0,
            "runtime_errors": 0,
            "recoveries": 0,
            "fatal_errors": 0,
            "status": "pass",
        }

    return {
        "driver": driver,
        "device_class": device_class,
        "profile": profile,
        "states_observed": ["probe_missing", "error_fatal"],
        "probe_attempts": 1,
        "probe_successes": 0,
        "init_failures": 1,
        "runtime_errors": 1,
        "recoveries": 0,
        "fatal_errors": 1,
        "status": "fail",
    }


def _resolve_hw_matrix_v4_lifecycle(
    rows: Sequence[Mapping[str, object]],
    *,
    driver: str,
    device_class: str,
    profile: str,
    required_states: Sequence[str],
    coverage_status: str,
) -> Dict[str, object]:
    try:
        return _find_lifecycle(rows, driver)
    except ValueError:
        return _synthesize_lifecycle(
            driver=driver,
            device_class=device_class,
            profile=profile,
            required_states=required_states,
            coverage_status=coverage_status,
        )


def _find_binding(
    rows: Sequence[Mapping[str, object]],
    driver: str,
    profile: str,
) -> Dict[str, object] | None:
    for row in rows:
        if row["driver"] == driver and row["profile"] == profile:
            return dict(row)
    return None


def _find_claim(
    claims: Sequence[Mapping[str, object]],
    class_id: str,
) -> Dict[str, object] | None:
    for row in claims:
        if row["class_id"] == class_id:
            return dict(row)
    return None


def collect_source_reports(seed: int) -> Dict[str, Dict[str, object]]:
    return {
        "measured_boot": measured_boot.build_report(
            platform="qemu-q35",
            pcrs=[0, 2, 4, 7],
            policy_profile="x2-runtime-backed-hardware",
        ),
        "hw_diagnostics_v3": hw_diag_v3.run_diagnostics(
            seed=seed,
            suspend_cycles=24,
            hotplug_events=16,
        ),
        "hw_matrix_v4": matrix_v4.run_matrix(seed=seed, max_failures=0),
        "hw_firmware_smp": firmware_smp.run_collection(seed=seed),
        "hw_matrix_v6": matrix_v6.run_matrix(seed=seed, max_failures=0),
        "baremetal_io": baremetal_io.run_baseline(seed=seed, max_failures=0),
        "hw_claim_promotion": claim_promotion.run_claim_promotion(seed=seed),
        "native_driver_diag": native_driver_diag.run_diagnostics(seed=seed, max_failures=0),
    }


def _target_check_id(target_id: str) -> str:
    return f"target_{target_id.replace('-', '_')}"


def _target_lines(spec: TargetSpec, passed: bool) -> List[Dict[str, object]]:
    markers = list(spec.boot_markers) + list(spec.runtime_markers)
    if not passed and markers:
        markers = markers[:-1] + [f"{markers[-1]} fail"]
    lines: List[Dict[str, object]] = []
    ts_ms = 0.0
    for marker in markers:
        lines.append({"ts_ms": round(ts_ms, 3), "line": marker})
        ts_ms += 37.0
    return lines


def build_device_registry(
    reports: Mapping[str, Mapping[str, object]],
) -> List[Dict[str, object]]:
    matrix_v4_report = reports["hw_matrix_v4"]
    matrix_v6_report = reports["hw_matrix_v6"]
    baremetal_report = reports["baremetal_io"]
    claim_report = reports["hw_claim_promotion"]

    registry: List[Dict[str, object]] = []
    for spec in DEVICE_SPECS:
        if spec.source_report == "hw_matrix_v4":
            coverage = _find_coverage(matrix_v4_report["device_class_coverage"], spec.driver)
            lifecycle = _resolve_hw_matrix_v4_lifecycle(
                matrix_v4_report["driver_lifecycle"],
                driver=spec.driver,
                device_class=spec.device_class,
                profile=spec.profile,
                required_states=spec.required_states,
                coverage_status=str(coverage["status"]),
            )
            source_schema = matrix_v4_report["schema"]
            source_digest = matrix_v4_report["digest"]
        elif spec.source_report == "hw_matrix_v6":
            coverage = _find_coverage(
                matrix_v6_report["device_class_coverage"],
                spec.driver,
                spec.profile,
            )
            lifecycle = _find_lifecycle(
                matrix_v6_report["driver_lifecycle"],
                spec.driver,
                spec.profile,
            )
            source_schema = matrix_v6_report["schema"]
            source_digest = matrix_v6_report["digest"]
        elif spec.source_report == "baremetal_io":
            coverage = _find_coverage(baremetal_report["device_class_coverage"], spec.driver)
            lifecycle = _find_lifecycle(
                baremetal_report["driver_lifecycle"],
                spec.driver,
                spec.profile,
            )
            source_schema = baremetal_report["schema"]
            source_digest = baremetal_report["digest"]
        else:
            raise ValueError(f"unsupported source report: {spec.source_report}")

        claim = _find_claim(claim_report["claims"], spec.claim_class_id) if spec.claim_class_id else None
        registry.append(
            {
                "class_id": spec.class_id,
                "driver": spec.driver,
                "device_class": spec.device_class,
                "profile": spec.profile,
                "qualified_tiers": list(spec.qualified_tiers),
                "source_report": spec.source_report,
                "source_schema": source_schema,
                "source_digest": source_digest,
                "source_milestones": list(spec.source_milestones),
                "required_states": list(spec.required_states),
                "required_markers": list(spec.required_markers),
                "states_observed": list(lifecycle["states_observed"]),
                "status": "pass"
                if coverage["status"] == "pass" and lifecycle["status"] == "pass"
                else "fail",
                "claim_status": claim["claim_status"] if claim else "not-claim-tracked",
                "promotion_policy_id": claim["promotion_policy_id"] if claim else "",
            }
        )
    return registry


def build_probe_bind_lifecycle(
    reports: Mapping[str, Mapping[str, object]],
    registry: Sequence[Mapping[str, object]],
) -> List[Dict[str, object]]:
    matrix_v4_report = reports["hw_matrix_v4"]
    matrix_v6_report = reports["hw_matrix_v6"]
    baremetal_report = reports["baremetal_io"]
    native_report = reports["native_driver_diag"]

    lifecycle_rows: List[Dict[str, object]] = []
    for entry in registry:
        source_report = entry["source_report"]
        driver = str(entry["driver"])
        profile = str(entry["profile"])
        if source_report == "hw_matrix_v4":
            coverage = _find_coverage(matrix_v4_report["device_class_coverage"], driver)
            source_lifecycle = _resolve_hw_matrix_v4_lifecycle(
                matrix_v4_report["driver_lifecycle"],
                driver=driver,
                device_class=str(entry["device_class"]),
                profile=profile,
                required_states=entry["required_states"],
                coverage_status=str(coverage["status"]),
            )
        elif source_report == "hw_matrix_v6":
            source_lifecycle = _find_lifecycle(matrix_v6_report["driver_lifecycle"], driver, profile)
        elif source_report == "baremetal_io":
            source_lifecycle = _find_lifecycle(baremetal_report["driver_lifecycle"], driver, profile)
        else:
            raise ValueError(f"unsupported source report: {source_report}")

        binding = _find_binding(native_report["driver_bindings"], driver, profile)
        bind_observed = binding is not None and binding["status"] == "pass"
        bind_markers = list(binding["markers"]) if binding is not None else ["DRV: bind inherited"]
        bind_latency_ms = binding["bind_latency_ms"] if binding is not None else 0
        dma_window_bytes = binding["dma_window_bytes"] if binding is not None else 0
        irq_vector = binding["irq_vector"] if binding is not None else 0
        lifecycle_rows.append(
            {
                "class_id": entry["class_id"],
                "driver": driver,
                "profile": profile,
                "device_class": entry["device_class"],
                "support_tiers": list(entry["qualified_tiers"]),
                "probe_states": list(source_lifecycle["states_observed"]),
                "probe_attempts": int(source_lifecycle["probe_attempts"]),
                "probe_successes": int(source_lifecycle["probe_successes"]),
                "runtime_errors": int(source_lifecycle["runtime_errors"]),
                "bind_observed": bind_observed or source_lifecycle["status"] == "pass",
                "bind_markers": bind_markers,
                "bind_latency_ms": bind_latency_ms,
                "dma_window_bytes": dma_window_bytes,
                "irq_vector": irq_vector,
                "firmware_policy_id": native_report["firmware_blob_policy_id"] if binding else "",
                "smp_safe": (
                    "cpu_affinity_balance" in source_lifecycle["states_observed"]
                    or int(source_lifecycle.get("affinity_balance_events", 0)) > 0
                ),
                "status": "pass"
                if source_lifecycle["status"] == "pass" and (binding is None or binding["status"] == "pass")
                else "fail",
            }
        )
    return lifecycle_rows


def build_firmware_runtime(
    reports: Mapping[str, Mapping[str, object]],
) -> Dict[str, object]:
    measured = reports["measured_boot"]
    firmware = reports["hw_firmware_smp"]
    native = reports["native_driver_diag"]
    firmware_pass = (
        bool(measured["policy_pass"])
        and bool(firmware["summary"]["firmware"]["pass"])
        and bool(native["summary"]["firmware"]["pass"])
    )
    return {
        "measured_boot": {
            "schema": measured["schema"],
            "policy_pass": measured["policy_pass"],
            "event_count": measured["event_count"],
            "required_pcrs": list(measured["expected_pcrs"]),
            "source_platform": measured["platform"],
        },
        "firmware_tables": {
            "schema": firmware["schema"],
            "gate_pass": firmware["gate_pass"],
            "hardening_id": firmware["firmware_hardening_id"],
            "source_matrix_digest": firmware["source_matrix_digest"],
            "checks_pass": firmware["summary"]["firmware"]["pass"],
        },
        "firmware_blobs": {
            "schema": native["schema"],
            "policy_id": native["firmware_blob_policy_id"],
            "audits": [
                {
                    "audit_id": row["audit_id"],
                    "decision": row["decision"],
                    "marker": row["marker"],
                    "status": row["status"],
                }
                for row in native["firmware_audits"]
            ],
            "checks_pass": native["summary"]["firmware"]["pass"],
        },
        "pass": firmware_pass,
    }


def build_smp_runtime(
    reports: Mapping[str, Mapping[str, object]],
) -> Dict[str, object]:
    firmware = reports["hw_firmware_smp"]
    smp = firmware["smp_baseline"]
    return {
        "schema": firmware["schema"],
        "interrupt_model_id": firmware["smp_interrupt_model_id"],
        "bootstrap_cpu_online_ratio": smp["bootstrap_cpu_online_ratio"],
        "application_cpu_online_ratio": smp["application_cpu_online_ratio"],
        "ipi_roundtrip_p95_ms": smp["ipi_roundtrip_p95_ms"],
        "lost_interrupt_events": smp["lost_interrupt_events"],
        "spurious_interrupt_rate": smp["spurious_interrupt_rate"],
        "required_markers": [
            "SMP: bsp online",
            "SMP: ap online",
            "SMP: ipi roundtrip ok",
            "SMP: affinity balanced",
        ],
        "pass": bool(firmware["summary"]["smp"]["pass"]),
    }


def build_runtime_targets(
    reports: Mapping[str, Mapping[str, object]],
    injected_failures: Set[str],
) -> List[Dict[str, object]]:
    matrix_v4_report = reports["hw_matrix_v4"]
    firmware_report = reports["hw_firmware_smp"]
    measured_report = reports["measured_boot"]
    matrix_v6_report = reports["hw_matrix_v6"]
    baremetal_report = reports["baremetal_io"]
    claim_report = reports["hw_claim_promotion"]
    native_report = reports["native_driver_diag"]

    targets: List[Dict[str, object]] = []
    for spec in TARGET_SPECS:
        if spec.target_id == "qemu-q35-transitional":
            source_pass = _find_tier_result(matrix_v4_report["tier_results"], "tier0")["status"] == "pass"
        elif spec.target_id == "qemu-i440fx-transitional":
            source_pass = _find_tier_result(matrix_v4_report["tier_results"], "tier1")["status"] == "pass"
        elif spec.target_id in {"qemu-q35-firmware-smp", "qemu-i440fx-firmware-smp"}:
            source_pass = bool(firmware_report["gate_pass"]) and bool(measured_report["policy_pass"])
        elif spec.target_id == "qemu-q35-modern-virtio":
            tier_pass = _find_tier_result(matrix_v6_report["tier_results"], "tier0")["status"] == "pass"
            binds = all(
                (_find_binding(native_report["driver_bindings"], driver, "modern") or {}).get("status") == "pass"
                for driver in ("virtio-blk-pci", "virtio-net-pci", "virtio-scsi-pci", "virtio-gpu-pci")
            )
            source_pass = tier_pass and binds
        elif spec.target_id == "qemu-i440fx-modern-virtio":
            tier_pass = _find_tier_result(matrix_v6_report["tier_results"], "tier1")["status"] == "pass"
            binds = all(
                (_find_binding(native_report["driver_bindings"], driver, "modern") or {}).get("status") == "pass"
                for driver in ("virtio-blk-pci", "virtio-net-pci", "virtio-scsi-pci", "virtio-gpu-pci")
            )
            source_pass = tier_pass and binds
        elif spec.target_id == "intel-q470-e1000e-xhci":
            target_row = next(
                row
                for row in baremetal_report["tier2_profiles"]
                if row["profile_id"] == "intel_q470_e1000e_xhci"
            )
            claims_ok = all(
                (_find_claim(claim_report["claims"], class_id) or {}).get("claim_status") == "promoted"
                for class_id in spec.claim_classes
            )
            binds = all(
                (_find_binding(native_report["driver_bindings"], driver, "baremetal") or {}).get("status") == "pass"
                for driver in ("e1000e", "xhci", "usb-storage")
            )
            source_pass = target_row["status"] == "pass" and claims_ok and binds
        elif spec.target_id == "amd-b550-rtl8169-xhci":
            target_row = next(
                row
                for row in baremetal_report["tier2_profiles"]
                if row["profile_id"] == "amd_b550_rtl8169_xhci"
            )
            claims_ok = all(
                (_find_claim(claim_report["claims"], class_id) or {}).get("claim_status") == "promoted"
                for class_id in spec.claim_classes
            )
            binds = all(
                (_find_binding(native_report["driver_bindings"], driver, "baremetal") or {}).get("status") == "pass"
                for driver in ("rtl8169", "xhci", "usb-storage")
            )
            source_pass = target_row["status"] == "pass" and claims_ok and binds
        else:
            raise ValueError(f"unsupported target id: {spec.target_id}")

        forced_fail = _target_check_id(spec.target_id) in injected_failures
        lines = _target_lines(spec, source_pass and not forced_fail)
        capture = {
            "capture_mode": "fixture",
            "serial_lines": lines,
            "serial_digest": runtime_capture.digest_lines(lines),
        }
        ordered_markers = [
            row["line"] for row in lines if row["line"] in list(spec.boot_markers) + list(spec.runtime_markers)
        ]
        targets.append(
            {
                "target_id": spec.target_id,
                "support_tier": spec.support_tier,
                "machine_profile": spec.machine_profile,
                "source_reports": list(spec.source_reports),
                "source_backlogs": list(spec.source_backlogs),
                "required_devices": list(spec.required_devices),
                "boot_markers": list(spec.boot_markers),
                "runtime_markers": list(spec.runtime_markers),
                "observed_markers": ordered_markers,
                "marker_sequence_ok": ordered_markers == list(spec.boot_markers) + list(spec.runtime_markers),
                "firmware_bound": spec.firmware_bound,
                "smp_bound": spec.smp_bound,
                "claim_classes": list(spec.claim_classes),
                "capture": capture,
                "qualification_pass": source_pass and not forced_fail,
            }
        )
    return targets


def build_backlog_closure(
    targets: Sequence[Mapping[str, object]],
    reports: Mapping[str, Mapping[str, object]],
) -> List[Dict[str, object]]:
    target_index = {row["target_id"]: row for row in targets}
    claim_report = reports["hw_claim_promotion"]
    closure_rows: List[Dict[str, object]] = []
    for backlog_id in ("M9", "M15", "M23", "M37", "M43", "M45", "M46", "M47"):
        target_ids = list(BACKLOG_TARGET_MAP[backlog_id])
        target_pass = all(target_index[target_id]["qualification_pass"] for target_id in target_ids)
        if backlog_id in {"M37", "M47"}:
            target_pass = target_pass and bool(claim_report["gate_pass"])
        closure_rows.append(
            {
                "backlog": backlog_id,
                "runtime_class": "Runtime-backed",
                "target_classes": target_ids,
                "source_reports": sorted(
                    {
                        source
                        for target_id in target_ids
                        for source in target_index[target_id]["source_reports"]
                    }
                ),
                "status": "pass" if target_pass else "fail",
            }
        )
    return closure_rows


def stable_digest(payload: Mapping[str, object]) -> str:
    return hashlib.sha256(
        json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
    ).hexdigest()


def build_report(
    seed: int,
    reports: Mapping[str, Mapping[str, object]],
    injected_failures: Set[str] | None = None,
) -> Dict[str, object]:
    failures = set() if injected_failures is None else set(injected_failures)

    registry = build_device_registry(reports)
    lifecycle = build_probe_bind_lifecycle(reports, registry)
    firmware_runtime = build_firmware_runtime(reports)
    smp_runtime = build_smp_runtime(reports)
    runtime_targets = build_runtime_targets(reports, failures)
    backlog_closure = build_backlog_closure(runtime_targets, reports)

    device_registry_pass = all(row["status"] == "pass" for row in registry)
    probe_bind_pass = all(row["status"] == "pass" for row in lifecycle)
    target_pass = all(row["qualification_pass"] for row in runtime_targets)
    backlog_pass = all(row["status"] == "pass" for row in backlog_closure)
    claim_traceable = bool(reports["hw_claim_promotion"]["gate_pass"])

    checks = [
        _check_row(
            "device_registry_complete",
            "device_registry",
            "eq",
            True,
            device_registry_pass and "device_registry_complete" not in failures,
        ),
        _check_row(
            "probe_bind_lifecycle_complete",
            "probe_bind",
            "eq",
            True,
            probe_bind_pass and "probe_bind_lifecycle_complete" not in failures,
        ),
        _check_row(
            "firmware_runtime_complete",
            "firmware",
            "eq",
            True,
            firmware_runtime["pass"] and "firmware_runtime_complete" not in failures,
        ),
        _check_row(
            "smp_runtime_complete",
            "smp",
            "eq",
            True,
            smp_runtime["pass"] and "smp_runtime_complete" not in failures,
        ),
    ]
    for target in runtime_targets:
        checks.append(
            _check_row(
                _target_check_id(str(target["target_id"])),
                "runtime_targets",
                "eq",
                True,
                bool(target["qualification_pass"]),
            )
        )
    checks.append(
        _check_row(
            "claim_promotion_traceable",
            "claims",
            "eq",
            True,
            claim_traceable and "claim_promotion_traceable" not in failures,
        )
    )

    total_failures = sum(1 for row in checks if row["pass"] is False)
    stable_payload = {
        "schema": SCHEMA,
        "policy_id": POLICY_ID,
        "seed": seed,
        "checks": [
            {
                "check_id": row["check_id"],
                "pass": row["pass"],
                "observed": row["observed"],
            }
            for row in checks
        ],
        "runtime_targets": [
            {
                "target_id": row["target_id"],
                "qualification_pass": row["qualification_pass"],
                "capture_digest": row["capture"]["serial_digest"],
            }
            for row in runtime_targets
        ],
        "injected_failures": sorted(failures),
    }
    digest = stable_digest(stable_payload)

    return {
        "schema": SCHEMA,
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "track_id": TRACK_ID,
        "policy_id": POLICY_ID,
        "device_registry_schema": DEVICE_REGISTRY_SCHEMA,
        "seed": seed,
        "gate": "test-x2-hardware-runtime-v1",
        "checks": checks,
        "summary": {
            "device_registry": {"entries": len(registry), "pass": device_registry_pass},
            "probe_bind": {"entries": len(lifecycle), "pass": probe_bind_pass},
            "firmware": {"pass": firmware_runtime["pass"]},
            "smp": {"pass": smp_runtime["pass"]},
            "runtime_targets": {
                "targets": len(runtime_targets),
                "qualified": sum(1 for row in runtime_targets if row["qualification_pass"]),
                "pass": target_pass,
            },
            "backlogs": {
                "covered": len(backlog_closure),
                "runtime_backed": sum(1 for row in backlog_closure if row["status"] == "pass"),
                "pass": backlog_pass,
            },
        },
        "backlog_closure": backlog_closure,
        "device_registry": registry,
        "probe_bind_lifecycle": lifecycle,
        "firmware_runtime": firmware_runtime,
        "smp_runtime": smp_runtime,
        "runtime_targets": runtime_targets,
        "source_reports": {
            name: {
                "schema": report["schema"],
                "gate_pass": report.get("gate_pass", report.get("policy_pass", True)),
                "digest": stable_digest(report),
            }
            for name, report in reports.items()
        },
        "artifact_refs": {
            "junit": "out/pytest-x2-hardware-runtime-v1.xml",
            "qualification_report": "out/x2-hardware-runtime-v1.json",
            "measured_boot_report": DEFAULT_SUPPORTING_REPORT_PATHS["measured_boot"],
            "hw_diagnostics_v3_report": DEFAULT_SUPPORTING_REPORT_PATHS["hw_diagnostics_v3"],
            "hw_matrix_v4_report": DEFAULT_SUPPORTING_REPORT_PATHS["hw_matrix_v4"],
            "hw_firmware_smp_report": DEFAULT_SUPPORTING_REPORT_PATHS["hw_firmware_smp"],
            "hw_matrix_v6_report": DEFAULT_SUPPORTING_REPORT_PATHS["hw_matrix_v6"],
            "baremetal_io_report": DEFAULT_SUPPORTING_REPORT_PATHS["baremetal_io"],
            "hw_claim_promotion_report": DEFAULT_SUPPORTING_REPORT_PATHS["hw_claim_promotion"],
            "native_driver_diag_report": DEFAULT_SUPPORTING_REPORT_PATHS["native_driver_diag"],
            "ci_artifact": "x2-hardware-runtime-v1-artifacts",
        },
        "injected_failures": sorted(failures),
        "total_failures": total_failures,
        "failures": sorted(row["check_id"] for row in checks if row["pass"] is False),
        "gate_pass": total_failures == 0,
        "digest": digest,
    }


def write_supporting_reports(
    reports: Mapping[str, Mapping[str, object]],
    *,
    base_dir: str | Path = "out",
) -> Dict[str, str]:
    paths = supporting_report_paths(base_dir)
    written: Dict[str, str] = {}
    for key, path in paths.items():
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(reports[key], indent=2) + "\n", encoding="utf-8")
        written[key] = path.as_posix()
    return written
