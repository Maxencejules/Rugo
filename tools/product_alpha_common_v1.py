#!/usr/bin/env python3
"""Shared helpers for the product-level alpha qualification gate."""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
from typing import Dict, List, Mapping, Sequence, Set

import build_installer_v2 as installer_tool
import collect_crash_dump_v1 as crash_tool
import collect_diagnostic_snapshot_v2 as diag_tool
import collect_trace_bundle_v2 as trace_tool
import release_bundle_v1 as release_bundle
import run_recovery_drill_v3 as recovery_tool
import run_upgrade_drill_v3 as upgrade_tool
import runtime_capture_common_v1 as runtime_capture
import symbolize_crash_dump_v1 as symbolizer
import update_repo_sign_v1 as update_sign
import x3_platform_runtime_common_v1 as x3_runtime
import x4_desktop_runtime_common_v1 as x4_runtime


SCHEMA = "rugo.product_alpha_qualification_report.v1"
POLICY_ID = "rugo.product_alpha_qualification.v1"
DEFAULT_SEED = 20260319
DEFAULT_MACHINE = "q35"
DEFAULT_CPU = "qemu64,+x2apic"
DEFAULT_DISK_DEVICE = "nvme,drive=disk0,serial=nvme0,logical_block_size=512"
DEFAULT_NET_DEVICE = "virtio-net-pci,netdev=n0,disable-modern=on"
DEFAULT_RELEASE_IMAGE_PATH = Path("out/os-go-desktop-native.iso")
DEFAULT_KERNEL_PATH = Path("out/kernel-go-desktop-native.elf")
DEFAULT_PANIC_IMAGE_PATH = Path("out/os-panic.iso")
DEFAULT_ARTIFACT_DIR = Path("out")
DEFAULT_RUNTIME_CAPTURE_PATH = Path("out/product-alpha-runtime-capture-v1.json")
DEFAULT_SUPPORTING_DIR = Path("out/product-alpha-supporting")
DEFAULT_CHANNEL = "alpha"
DEFAULT_VERSION = "1.0.0-alpha.1"
DEFAULT_BUILD_SEQUENCE = 1
WINDOW_SECONDS = 300

CHECK_IDS = {
    "bootable_default_image",
    "durable_nvme_storage",
    "wired_networking",
    "desktop_or_shell_boot",
    "install_path",
    "update_path",
    "recovery_path",
    "diagnostics_path",
}

BOOT_MARKERS = ("RUGO: boot ok", "GOINIT: ready", "RUGO: halt ok")
NVME_COLD_MARKERS = ("STORC4: block ready driver=nvme", "STORC4: journal staged")
NVME_REPLAY_MARKERS = (
    "STORC4: block ready driver=nvme",
    "RECOV: replay ok",
    "BLK: fua ok",
    "BLK: flush ordered",
    "STORC4: fsync ok",
)
NETWORK_MARKERS = (
    "NETC4: nic ready",
    "NETC4: ifcfg ok",
    "NETC4: route ok",
    "NETC4: listen ok",
    "NETC4: connect ok",
    "NETC4: accept ok",
    "NETC4: recv ok",
    "NETC4: reply ok",
)


@dataclass(frozen=True)
class AlphaPaths:
    artifact_dir: Path
    supporting_dir: Path
    runtime_capture: Path
    x4_report: Path
    x3_report: Path
    graphical_installer: Path
    trace_bundle: Path
    diagnostic_snapshot: Path
    crash_dump: Path
    crash_dump_symbolized: Path
    release_root: Path
    release_bundle: Path
    install_state: Path
    installer_contract: Path
    update_repo: Path
    update_metadata: Path
    upgrade_drill: Path
    recovery_drill: Path
    report: Path


def _created_utc() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def _stable_digest(payload: Dict[str, object]) -> str:
    return hashlib.sha256(
        json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
    ).hexdigest()


def build_paths(
    *,
    artifact_dir: Path = DEFAULT_ARTIFACT_DIR,
    supporting_dir: Path = DEFAULT_SUPPORTING_DIR,
    report_path: Path | None = None,
    runtime_capture_path: Path | None = None,
) -> AlphaPaths:
    report = report_path if report_path is not None else artifact_dir / "product-alpha-v1.json"
    runtime_capture_out = (
        runtime_capture_path
        if runtime_capture_path is not None
        else artifact_dir / "product-alpha-runtime-capture-v1.json"
    )
    return AlphaPaths(
        artifact_dir=artifact_dir,
        supporting_dir=supporting_dir,
        runtime_capture=runtime_capture_out,
        x4_report=artifact_dir / "product-alpha-x4-runtime-v1.json",
        x3_report=artifact_dir / "product-alpha-x3-runtime-v1.json",
        graphical_installer=artifact_dir / "product-alpha-graphical-installer-v1.json",
        trace_bundle=artifact_dir / "product-alpha-trace-bundle-v2.json",
        diagnostic_snapshot=artifact_dir / "product-alpha-diagnostic-snapshot-v2.json",
        crash_dump=artifact_dir / "product-alpha-crash-dump-v1.json",
        crash_dump_symbolized=artifact_dir / "product-alpha-crash-dump-symbolized-v1.json",
        release_root=artifact_dir / "product-alpha-releases",
        release_bundle=artifact_dir / "product-alpha-release-bundle-v1.json",
        install_state=artifact_dir / "product-alpha-install-state-v1.json",
        installer_contract=artifact_dir / "product-alpha-installer-v2.json",
        update_repo=artifact_dir / "product-alpha-update-repo-v1",
        update_metadata=artifact_dir / "product-alpha-update-metadata-v1.json",
        upgrade_drill=artifact_dir / "product-alpha-upgrade-drill-v3.json",
        recovery_drill=artifact_dir / "product-alpha-recovery-drill-v3.json",
        report=report,
    )


def normalize_failures(values: Sequence[str]) -> Set[str]:
    failures = {value.strip() for value in values if value.strip()}
    unknown = sorted(failures - CHECK_IDS)
    if unknown:
        raise ValueError(f"unknown check ids in --inject-failure: {', '.join(unknown)}")
    return failures


def ensure_fixture_artifacts(
    *,
    image_path: str,
    kernel_path: str,
    panic_image_path: str,
) -> None:
    payloads = {
        Path(image_path): b"fixture alpha desktop image\n",
        Path(kernel_path): b"fixture alpha desktop kernel\n",
        Path(panic_image_path): b"fixture panic image\n",
    }
    for path, payload in payloads.items():
        path.parent.mkdir(parents=True, exist_ok=True)
        if not path.is_file():
            path.write_bytes(payload)


def _native_desktop_fixture_lines(
    lines: Sequence[Dict[str, object]],
) -> List[Dict[str, object]]:
    updated: List[Dict[str, object]] = []
    for entry in lines:
        ts_ms = round(float(entry["ts_ms"]), 3)
        line = str(entry["line"])
        if line == "STORC4: block ready":
            line = "STORC4: block ready driver=nvme"
        updated.append({"ts_ms": ts_ms, "line": line})
        if line == "STORC4: state ok":
            updated.append({"ts_ms": round(ts_ms + 0.5, 3), "line": "BLK: fua ok"})
    return updated


def build_fixture_capture(
    *,
    image_path: str = str(DEFAULT_RELEASE_IMAGE_PATH),
    kernel_path: str = str(DEFAULT_KERNEL_PATH),
    panic_image_path: str = str(DEFAULT_PANIC_IMAGE_PATH),
) -> Dict[str, object]:
    ensure_fixture_artifacts(
        image_path=image_path,
        kernel_path=kernel_path,
        panic_image_path=panic_image_path,
    )
    image_path_obj = Path(image_path)
    kernel_path_obj = Path(kernel_path)
    panic_image_path_obj = Path(panic_image_path)
    image_digest = runtime_capture.maybe_sha256_file(image_path_obj, "alpha-fixture-image")
    kernel_digest = runtime_capture.maybe_sha256_file(
        kernel_path_obj,
        "alpha-fixture-kernel",
    )
    panic_image_digest = runtime_capture.maybe_sha256_file(
        panic_image_path_obj,
        "alpha-fixture-panic-image",
    )
    fixture_lines = [
        _native_desktop_fixture_lines(x4_runtime._inject_desktop_markers(boot_lines))
        for boot_lines in runtime_capture.fixture_boot_lines()
    ]
    provisional = x4_runtime._build_capture(
        image_path=runtime_capture.posix_path(image_path_obj),
        kernel_path=runtime_capture.posix_path(kernel_path_obj),
        panic_image_path=runtime_capture.posix_path(panic_image_path_obj),
        image_digest=image_digest,
        kernel_digest=kernel_digest,
        panic_image_digest=panic_image_digest,
        capture_mode="fixture",
        boots=[],
    )
    capture_id = str(provisional["capture_id"])
    boots = [
        x4_runtime._build_boot_entry(
            capture_id=capture_id,
            boot_index=index + 1,
            boot_profile=profile,
            lines=lines,
            exit_code=0,
        )
        for index, (profile, lines) in enumerate(
            zip(["cold_boot", "replay_boot"], fixture_lines)
        )
    ]
    payload = x4_runtime._build_capture(
        image_path=runtime_capture.posix_path(image_path_obj),
        kernel_path=runtime_capture.posix_path(kernel_path_obj),
        panic_image_path=runtime_capture.posix_path(panic_image_path_obj),
        image_digest=image_digest,
        kernel_digest=kernel_digest,
        panic_image_digest=panic_image_digest,
        capture_mode="fixture",
        boots=boots,
    )
    payload["fixture_seed"] = runtime_capture.FIXTURE_SEED
    payload["created_utc"] = "2026-03-19T00:00:00Z"
    payload["boot_profiles"] = [boot["boot_profile"] for boot in boots]
    payload["panic_boot_id"] = f"panic-{payload['capture_id'][:12]}"
    payload["machine"] = DEFAULT_MACHINE
    payload["cpu"] = DEFAULT_CPU
    payload["timeout_seconds"] = runtime_capture.DEFAULT_TIMEOUT_SECONDS
    payload["disk_device"] = DEFAULT_DISK_DEVICE
    payload["net_device"] = DEFAULT_NET_DEVICE
    return payload


def load_runtime_capture(
    *,
    runtime_capture_path: str = "",
    fixture: bool = False,
    image_path: str = str(DEFAULT_RELEASE_IMAGE_PATH),
    kernel_path: str = str(DEFAULT_KERNEL_PATH),
    panic_image_path: str = str(DEFAULT_PANIC_IMAGE_PATH),
    machine: str = DEFAULT_MACHINE,
    cpu: str = DEFAULT_CPU,
    timeout_seconds: float = runtime_capture.DEFAULT_TIMEOUT_SECONDS,
    disk_device: str = DEFAULT_DISK_DEVICE,
    net_device: str = DEFAULT_NET_DEVICE,
) -> Dict[str, object]:
    if fixture:
        return build_fixture_capture(
            image_path=image_path,
            kernel_path=kernel_path,
            panic_image_path=panic_image_path,
        )
    if runtime_capture_path:
        path = Path(runtime_capture_path)
        if not path.is_file():
            raise FileNotFoundError(f"runtime capture not found: {path}")
        return runtime_capture.read_json(path)
    return runtime_capture.collect_booted_runtime(
        image_path=image_path,
        kernel_path=kernel_path,
        panic_image_path=panic_image_path,
        machine=machine,
        cpu=cpu,
        timeout_seconds=timeout_seconds,
        disk_device=disk_device,
        net_device=net_device,
    )


def _boot_by_profile(capture: Mapping[str, object], profile: str) -> Dict[str, object]:
    for boot in runtime_capture.iter_boots(dict(capture)):
        if boot.get("boot_profile") == profile:
            return boot
    raise KeyError(f"missing boot profile: {profile}")


def _markers_present(boot: Mapping[str, object], markers: Sequence[str]) -> bool:
    return all(runtime_capture.find_first_line_ts(dict(boot), marker) is not None for marker in markers)


def _report_pass(report: Mapping[str, object]) -> bool:
    gate_pass = report.get("gate_pass")
    if isinstance(gate_pass, bool):
        return gate_pass
    status = report.get("status")
    if isinstance(status, str):
        return status == "pass"
    total_failures = report.get("total_failures")
    if isinstance(total_failures, int):
        return total_failures == 0
    return False


def _x4_check(report: Mapping[str, object], check_id: str) -> bool:
    for row in report.get("checks", []):
        if isinstance(row, dict) and row.get("check_id") == check_id:
            return row.get("pass") is True
    return False


def _write_json(path: Path, payload: Dict[str, object]) -> None:
    runtime_capture.write_json(path, payload)


def _build_release_flow(
    *,
    seed: int,
    image_path: str,
    kernel_path: str,
    panic_image_path: str,
    channel: str,
    version: str,
    build_sequence: int,
    paths: AlphaPaths,
) -> Dict[str, Dict[str, object]]:
    bundle = release_bundle.stage_release_bundle(
        channel=channel,
        version=version,
        build_sequence=build_sequence,
        system_image=Path(image_path),
        kernel=Path(kernel_path),
        panic_image=Path(panic_image_path),
        release_root=paths.release_root,
        capture_mode="fixture",
        release_notes_lines=[
            "Alpha candidate shipping lane: `image-go-desktop-native` on q35 + NVMe.",
            "Installer and recovery media reuse the shipped alpha candidate ISO.",
            "Product alpha qualification consumes a separate live runtime capture.",
        ],
    )
    release_bundle.write_json(paths.release_bundle, bundle)

    install_state = release_bundle.build_install_state(
        bundle=bundle,
        bundle_path=paths.release_bundle.as_posix(),
    )
    release_bundle.write_install_state(paths.install_state, install_state)

    installer_contract = installer_tool.build_installer_contract(
        channel=channel,
        version=version,
        build_sequence=build_sequence,
        bundle=bundle,
        install_state_path=paths.install_state.as_posix(),
    )
    _write_json(paths.installer_contract, installer_contract)

    update_metadata = update_sign.build_signed_metadata(
        repo=paths.update_repo,
        channel=channel,
        version=version,
        build_sequence=build_sequence,
        key="product-alpha-update-key-v1",
        key_id="alpha-key-2026-03",
        expires_hours=168,
        target_artifacts=[
            Path(path)
            for path in bundle.get("update_repo_targets", [])
            if isinstance(path, str)
        ],
    )
    _write_json(paths.update_metadata, update_metadata)
    repo_meta = paths.update_repo / "metadata" / "update-metadata-v1.json"
    _write_json(repo_meta, update_metadata)

    upgrade_report = upgrade_tool.run_upgrade_drill(
        seed=seed,
        candidate_sequence=int(update_metadata["build_sequence"]),
        rollback_floor_sequence=int(update_metadata["rollback_floor_sequence"]),
        bundle=bundle,
        install_state=install_state,
    )
    upgrade_report["max_failures"] = 0
    rollback = upgrade_report["rollback_safety"]
    upgrade_report["meets_target"] = (
        upgrade_report["total_failures"] == 0
        and rollback["rollback_floor_enforced"] is True
        and rollback["unsigned_artifact_rejected"] is True
        and rollback["rollback_path_verified"] is True
    )
    upgrade_report["gate_pass"] = upgrade_report["meets_target"]
    _write_json(paths.upgrade_drill, upgrade_report)

    recovery_report = recovery_tool.run_recovery_drill(
        seed=seed,
        bundle=bundle,
        install_state=install_state,
    )
    recovery_report["max_failures"] = 0
    readiness = recovery_report["recovery_readiness"]
    recovery_report["meets_target"] = (
        recovery_report["total_failures"] == 0
        and readiness["operator_checklist_completed"] is True
        and readiness["state_capture_complete"] is True
    )
    recovery_report["gate_pass"] = recovery_report["meets_target"]
    _write_json(paths.recovery_drill, recovery_report)

    return {
        "release_bundle": bundle,
        "installer_contract": installer_contract,
        "install_state": install_state,
        "update_metadata": update_metadata,
        "upgrade_drill": upgrade_report,
        "recovery_drill": recovery_report,
    }


def _build_diagnostic_flow(
    *,
    capture: Dict[str, object],
    image_path: str,
    kernel_path: str,
    panic_image_path: str,
    fixture: bool,
    paths: AlphaPaths,
) -> Dict[str, Dict[str, object]]:
    trace_bundle = trace_tool.collect_trace_bundle(
        runtime_capture_payload=capture,
        window_seconds=WINDOW_SECONDS,
    )
    trace_bundle["max_errors"] = 0
    trace_bundle["max_dropped_spans"] = 0
    trace_bundle["gate_pass"] = (
        trace_bundle["totals"]["total_errors"] == 0
        and trace_bundle["totals"]["total_dropped_spans"] == 0
    )
    _write_json(paths.trace_bundle, trace_bundle)

    diagnostic_snapshot = diag_tool.collect_snapshot(
        runtime_capture_payload=capture,
        trace_bundle=trace_bundle,
    )
    diagnostic_snapshot["max_unhealthy_checks"] = 0
    diagnostic_snapshot["gate_pass"] = diagnostic_snapshot["unhealthy_checks"] == 0
    _write_json(paths.diagnostic_snapshot, diagnostic_snapshot)

    if fixture:
        panic_capture = runtime_capture.build_panic_fixture(
            release_image_path=image_path,
            kernel_path=kernel_path,
            panic_image_path=panic_image_path,
        )
    else:
        panic_capture = runtime_capture.collect_panic_capture(
            release_image_path=image_path,
            kernel_path=kernel_path,
            panic_image_path=panic_image_path,
            machine=DEFAULT_MACHINE,
            timeout_seconds=runtime_capture.DEFAULT_TIMEOUT_SECONDS,
        )

    crash_dump = crash_tool.build_dump(
        panic_code=int(panic_capture["panic_code"]),
        panic_reason="alpha_panic_validation",
        kernel_build_id=f"rugo-kernel-{str(panic_capture['kernel_digest'])[:12]}",
        release_channel=DEFAULT_CHANNEL,
        release_image_path=str(panic_capture["release_image_path"]),
        release_image_digest=str(panic_capture["release_image_digest"]),
        panic_image_path=str(panic_capture["panic_image_path"]),
        panic_image_digest=str(panic_capture["panic_image_digest"]),
        panic_boot_id=str(panic_capture["panic_boot_id"]),
        panic_trace_id=str(panic_capture["panic_trace_id"]),
        panic_trace_digest=str(panic_capture["panic_trace_digest"]),
        serial_digest=str(panic_capture["serial_digest"]),
        serial_lines=list(panic_capture.get("serial_lines", [])),
        capture_mode=str(panic_capture["capture_mode"]),
    )
    _write_json(paths.crash_dump, crash_dump)

    crash_symbolized = symbolizer.symbolize(crash_dump)
    crash_symbolized["max_unresolved"] = 0
    crash_symbolized["all_frames_symbolized"] = crash_symbolized["unresolved_frames"] == 0
    crash_symbolized["gate_pass"] = crash_symbolized["unresolved_frames"] == 0
    _write_json(paths.crash_dump_symbolized, crash_symbolized)

    return {
        "trace_bundle": trace_bundle,
        "diagnostic_snapshot": diagnostic_snapshot,
        "crash_dump": crash_dump,
        "crash_dump_symbolized": crash_symbolized,
    }


def collect_reports(
    *,
    seed: int,
    capture: Dict[str, object],
    image_path: str,
    kernel_path: str,
    panic_image_path: str,
    fixture: bool,
    channel: str,
    version: str,
    build_sequence: int,
    emit_supporting_reports: bool,
    paths: AlphaPaths,
) -> Dict[str, Dict[str, object]]:
    x4_source_reports = x4_runtime.collect_source_reports(seed=seed)
    if emit_supporting_reports:
        x4_runtime.write_supporting_reports(x4_source_reports, base_dir=paths.supporting_dir)
    x4_report = x4_runtime.build_report(
        seed=seed,
        capture=capture,
        reports=x4_source_reports,
    )
    _write_json(paths.x4_report, x4_report)

    x3_source_reports = x3_runtime.collect_source_reports(seed=seed)
    if emit_supporting_reports:
        x3_runtime.write_supporting_reports(x3_source_reports, base_dir=paths.supporting_dir)
    x3_report = x3_runtime.build_report(
        seed=seed,
        capture=capture,
        reports=x3_source_reports,
    )
    _write_json(paths.x3_report, x3_report)

    graphical_installer = dict(x4_source_reports["graphical_installer_v1"])
    _write_json(paths.graphical_installer, graphical_installer)

    release_reports = _build_release_flow(
        seed=seed,
        image_path=image_path,
        kernel_path=kernel_path,
        panic_image_path=panic_image_path,
        channel=channel,
        version=version,
        build_sequence=build_sequence,
        paths=paths,
    )
    diagnostic_reports = _build_diagnostic_flow(
        capture=capture,
        image_path=image_path,
        kernel_path=kernel_path,
        panic_image_path=panic_image_path,
        fixture=fixture,
        paths=paths,
    )

    return {
        "desktop_profile_runtime": x4_report,
        "platform_runtime": x3_report,
        "graphical_installer": graphical_installer,
        **release_reports,
        **diagnostic_reports,
    }


def _check_row(
    *,
    check_id: str,
    requirement: str,
    domain: str,
    passed: bool,
    evidence: Sequence[str],
    details: Mapping[str, object],
) -> Dict[str, object]:
    return {
        "check_id": check_id,
        "requirement": requirement,
        "domain": domain,
        "pass": bool(passed),
        "evidence": list(evidence),
        "details": dict(details),
    }


def _build_checks(
    *,
    capture: Dict[str, object],
    reports: Mapping[str, Dict[str, object]],
    image_path: str,
    injected_failures: Set[str],
) -> List[Dict[str, object]]:
    cold = _boot_by_profile(capture, "cold_boot")
    replay = _boot_by_profile(capture, "replay_boot")
    x4_report = reports["desktop_profile_runtime"]
    x3_report = reports["platform_runtime"]
    installer_report = reports["graphical_installer"]
    installer_contract = reports["installer_contract"]
    update_metadata = reports["update_metadata"]
    upgrade_report = reports["upgrade_drill"]
    recovery_report = reports["recovery_drill"]
    trace_bundle = reports["trace_bundle"]
    diagnostic_snapshot = reports["diagnostic_snapshot"]
    crash_symbolized = reports["crash_dump_symbolized"]

    checks = [
        _check_row(
            check_id="bootable_default_image",
            requirement="Bootable default image on the declared q35 desktop profile.",
            domain="boot",
            passed=(
                capture.get("image_path") == runtime_capture.posix_path(Path(image_path))
                and capture.get("machine") == DEFAULT_MACHINE
                and len(list(runtime_capture.iter_boots(capture))) == 2
                and _markers_present(cold, BOOT_MARKERS)
                and _markers_present(replay, BOOT_MARKERS)
            ),
            evidence=["product-alpha-runtime-capture-v1.json", "product-alpha-x4-runtime-v1.json"],
            details={
                "image_path": capture.get("image_path", ""),
                "machine": capture.get("machine", ""),
                "boot_profiles": capture.get("boot_profiles", []),
            },
        ),
        _check_row(
            check_id="durable_nvme_storage",
            requirement="Durable NVMe-backed storage path on the shipped alpha candidate image.",
            domain="storage",
            passed=(
                capture.get("cpu") == DEFAULT_CPU
                and "nvme" in str(capture.get("disk_device", ""))
                and _markers_present(cold, NVME_COLD_MARKERS)
                and _markers_present(replay, NVME_REPLAY_MARKERS)
            ),
            evidence=["product-alpha-runtime-capture-v1.json"],
            details={
                "cpu": capture.get("cpu", ""),
                "disk_device": capture.get("disk_device", ""),
                "cold_markers": list(NVME_COLD_MARKERS),
                "replay_markers": list(NVME_REPLAY_MARKERS),
            },
        ),
        _check_row(
            check_id="wired_networking",
            requirement="Wired networking is visible and consumed on the declared profile.",
            domain="network",
            passed=_markers_present(cold, NETWORK_MARKERS) and _markers_present(replay, NETWORK_MARKERS),
            evidence=["product-alpha-runtime-capture-v1.json", "product-alpha-x3-runtime-v1.json"],
            details={"required_markers": list(NETWORK_MARKERS)},
        ),
        _check_row(
            check_id="desktop_or_shell_boot",
            requirement="Boot to desktop or shell without source-level intervention.",
            domain="desktop",
            passed=(
                _report_pass(x4_report)
                and all(
                    _x4_check(x4_report, check_id)
                    for check_id in [
                        "desktop_bootstrap",
                        "display_scanout",
                        "input_seat",
                        "window_compositor",
                        "gui_runtime",
                        "shell_workflows",
                    ]
                )
            ),
            evidence=["product-alpha-x4-runtime-v1.json"],
            details={
                "desktop_profile_id": x4_report.get("desktop_profile_id", ""),
                "gate_pass": x4_report.get("gate_pass"),
            },
        ),
        _check_row(
            check_id="install_path",
            requirement="Installer path is demonstrated on the shipped alpha candidate image.",
            domain="operations",
            passed=(
                _report_pass(installer_report)
                and _x4_check(x4_report, "graphical_installer")
                and installer_report.get("selected_target", {}).get("device_id") == "disk0"
                and "bootable_media" in installer_contract.get("installer_profile", {})
            ),
            evidence=[
                "product-alpha-graphical-installer-v1.json",
                "product-alpha-installer-v2.json",
            ],
            details={
                "selected_target": installer_report.get("selected_target", {}).get("device_id", ""),
                "installer_media": installer_contract.get("installer_profile", {}).get(
                    "bootable_media",
                    {},
                ),
            },
        ),
        _check_row(
            check_id="update_path",
            requirement="Update path is demonstrated on the shipped alpha candidate image.",
            domain="operations",
            passed=(
                _report_pass(x3_report)
                and x3_report.get("summary", {}).get("package_update", {}).get("pass") is True
                and _report_pass(upgrade_report)
                and bool(update_metadata.get("signature", {}).get("value", ""))
            ),
            evidence=[
                "product-alpha-x3-runtime-v1.json",
                "product-alpha-update-metadata-v1.json",
                "product-alpha-upgrade-drill-v3.json",
            ],
            details={
                "package_update_pass": x3_report.get("summary", {}).get("package_update", {}).get("pass"),
                "upgrade_gate_pass": upgrade_report.get("gate_pass"),
                "signed_targets": len(update_metadata.get("targets", [])),
            },
        ),
        _check_row(
            check_id="recovery_path",
            requirement="Recovery path is demonstrated on the shipped alpha candidate image.",
            domain="operations",
            passed=(
                _report_pass(recovery_report)
                and installer_report.get("source_reports", {})
                .get("recovery_drill", {})
                .get("gate_pass")
                is True
                and installer_contract.get("recovery_profile", {}).get("rollback_supported") is True
            ),
            evidence=[
                "product-alpha-graphical-installer-v1.json",
                "product-alpha-recovery-drill-v3.json",
            ],
            details={
                "recovery_gate_pass": recovery_report.get("gate_pass"),
                "rollback_supported": installer_contract.get("recovery_profile", {}).get(
                    "rollback_supported",
                    False,
                ),
            },
        ),
        _check_row(
            check_id="diagnostics_path",
            requirement="Readable crash and diagnostics flow is available on the alpha candidate image.",
            domain="diagnostics",
            passed=(
                _report_pass(trace_bundle)
                and _report_pass(diagnostic_snapshot)
                and _report_pass(crash_symbolized)
            ),
            evidence=[
                "product-alpha-trace-bundle-v2.json",
                "product-alpha-diagnostic-snapshot-v2.json",
                "product-alpha-crash-dump-symbolized-v1.json",
            ],
            details={
                "trace_gate_pass": trace_bundle.get("gate_pass"),
                "diagnostic_gate_pass": diagnostic_snapshot.get("gate_pass"),
                "crash_gate_pass": crash_symbolized.get("gate_pass"),
            },
        ),
    ]

    for row in checks:
        if row["check_id"] in injected_failures:
            row["pass"] = False
            row["details"]["injected_failure"] = True
    return checks


def build_report(
    *,
    seed: int,
    capture: Dict[str, object],
    reports: Mapping[str, Dict[str, object]],
    image_path: str,
    kernel_path: str,
    panic_image_path: str,
    paths: AlphaPaths,
    injected_failures: Set[str] | None = None,
) -> Dict[str, object]:
    failures = set() if injected_failures is None else set(injected_failures)
    checks = _build_checks(
        capture=capture,
        reports=reports,
        image_path=image_path,
        injected_failures=failures,
    )
    total_failures = sum(1 for row in checks if row["pass"] is False)

    summary = {
        "boot": {"pass": next(row["pass"] for row in checks if row["check_id"] == "bootable_default_image")},
        "storage": {"pass": next(row["pass"] for row in checks if row["check_id"] == "durable_nvme_storage")},
        "network": {"pass": next(row["pass"] for row in checks if row["check_id"] == "wired_networking")},
        "desktop": {"pass": next(row["pass"] for row in checks if row["check_id"] == "desktop_or_shell_boot")},
        "operations": {
            "checks": 3,
            "failures": sum(
                1
                for row in checks
                if row["domain"] == "operations" and row["pass"] is False
            ),
            "pass": all(
                row["pass"] for row in checks if row["domain"] == "operations"
            ),
        },
        "diagnostics": {"pass": next(row["pass"] for row in checks if row["check_id"] == "diagnostics_path")},
        "requirements": {
            "passed": sum(1 for row in checks if row["pass"] is True),
            "total": len(checks),
        },
    }

    stable_payload = {
        "schema": SCHEMA,
        "policy_id": POLICY_ID,
        "seed": seed,
        "capture_digest": capture.get("digest", ""),
        "checks": [
            {"check_id": row["check_id"], "pass": row["pass"]} for row in checks
        ],
    }
    digest = _stable_digest(stable_payload)

    source_reports = {
        "desktop_profile_runtime": {
            "path": paths.x4_report.as_posix(),
            "schema": reports["desktop_profile_runtime"].get("schema", ""),
            "digest": reports["desktop_profile_runtime"].get("digest", ""),
            "gate_pass": reports["desktop_profile_runtime"].get("gate_pass"),
        },
        "platform_runtime": {
            "path": paths.x3_report.as_posix(),
            "schema": reports["platform_runtime"].get("schema", ""),
            "digest": reports["platform_runtime"].get("digest", ""),
            "gate_pass": reports["platform_runtime"].get("gate_pass"),
        },
        "graphical_installer": {
            "path": paths.graphical_installer.as_posix(),
            "schema": reports["graphical_installer"].get("schema", ""),
            "digest": reports["graphical_installer"].get("digest", ""),
            "gate_pass": reports["graphical_installer"].get("gate_pass"),
        },
        "release_bundle": {
            "path": paths.release_bundle.as_posix(),
            "schema": reports["release_bundle"].get("schema", ""),
            "digest": reports["release_bundle"].get("digest", ""),
            "selected_channel": reports["release_bundle"].get("selected_channel", ""),
        },
        "installer_contract": {
            "path": paths.installer_contract.as_posix(),
            "schema": reports["installer_contract"].get("schema", ""),
            "selected_channel": reports["installer_contract"].get("selected_channel", ""),
        },
        "update_metadata": {
            "path": paths.update_metadata.as_posix(),
            "schema": reports["update_metadata"].get("schema", ""),
            "signed_targets": len(reports["update_metadata"].get("targets", [])),
        },
        "upgrade_drill": {
            "path": paths.upgrade_drill.as_posix(),
            "schema": reports["upgrade_drill"].get("schema", ""),
            "gate_pass": reports["upgrade_drill"].get("gate_pass"),
        },
        "recovery_drill": {
            "path": paths.recovery_drill.as_posix(),
            "schema": reports["recovery_drill"].get("schema", ""),
            "gate_pass": reports["recovery_drill"].get("gate_pass"),
        },
        "trace_bundle": {
            "path": paths.trace_bundle.as_posix(),
            "schema": reports["trace_bundle"].get("schema", ""),
            "digest": reports["trace_bundle"].get("digest", ""),
            "gate_pass": reports["trace_bundle"].get("gate_pass"),
        },
        "diagnostic_snapshot": {
            "path": paths.diagnostic_snapshot.as_posix(),
            "schema": reports["diagnostic_snapshot"].get("schema", ""),
            "digest": reports["diagnostic_snapshot"].get("digest", ""),
            "gate_pass": reports["diagnostic_snapshot"].get("gate_pass"),
        },
        "crash_dump": {
            "path": paths.crash_dump.as_posix(),
            "schema": reports["crash_dump"].get("schema", ""),
            "digest": reports["crash_dump"].get("digest", ""),
        },
        "crash_dump_symbolized": {
            "path": paths.crash_dump_symbolized.as_posix(),
            "schema": reports["crash_dump_symbolized"].get("schema", ""),
            "digest": reports["crash_dump_symbolized"].get("digest", ""),
            "gate_pass": reports["crash_dump_symbolized"].get("gate_pass"),
        },
    }

    return {
        "schema": SCHEMA,
        "policy_id": POLICY_ID,
        "created_utc": _created_utc(),
        "seed": seed,
        "gate": "test-product-alpha-v1",
        "declared_profile": {
            "profile_id": "qemu-q35-default-desktop",
            "machine": DEFAULT_MACHINE,
            "cpu": DEFAULT_CPU,
            "storage": "nvme",
            "network": "wired-virtio-net",
            "display": "desktop-profile",
            "input": "keyboard-pointer",
        },
        "release_image": {
            "image_path": runtime_capture.posix_path(Path(image_path)),
            "kernel_path": runtime_capture.posix_path(Path(kernel_path)),
            "panic_image_path": runtime_capture.posix_path(Path(panic_image_path)),
            "capture_id": capture.get("capture_id", ""),
            "capture_mode": capture.get("capture_mode", ""),
            "digest": capture.get("digest", ""),
        },
        "checks": checks,
        "summary": summary,
        "source_reports": source_reports,
        "artifact_refs": {
            "runtime_capture": paths.runtime_capture.as_posix(),
            "report": paths.report.as_posix(),
            "junit": "out/pytest-product-alpha-v1.xml",
            "boot_image": runtime_capture.posix_path(Path(image_path)),
            "kernel_image": runtime_capture.posix_path(Path(kernel_path)),
            "panic_image": runtime_capture.posix_path(Path(panic_image_path)),
            "supporting_dir": paths.supporting_dir.as_posix(),
            "ci_artifact": "product-alpha-v1-artifacts",
        },
        "injected_failures": sorted(failures),
        "failures": [row["check_id"] for row in checks if row["pass"] is False],
        "total_failures": total_failures,
        "gate_pass": total_failures == 0,
        "digest": digest,
    }
