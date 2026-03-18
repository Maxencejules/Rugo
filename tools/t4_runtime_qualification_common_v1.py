#!/usr/bin/env python3
"""Shared runtime-backed qualification helpers for T4 milestone gates."""

from __future__ import annotations

import hashlib
import json
import tempfile
from pathlib import Path
from typing import Dict, Iterable, List, Sequence, Tuple

import pkg_rebuild_verify_v3 as pkg_rebuild
import runtime_capture_common_v1 as runtime_capture
import verify_release_attestations_v1 as attestations


DEFAULT_RUNTIME_CAPTURE_PATH = Path("out/booted-runtime-v1.json")
DEFAULT_ATTESTATION_PATH = Path("out/release-attestation-verification-v1.json")
DEFAULT_PKG_REBUILD_PATH = Path("out/pkg-rebuild-v3.json")
DEFAULT_IMAGE_PATH = runtime_capture.DEFAULT_RELEASE_IMAGE_PATH
DEFAULT_KERNEL_PATH = runtime_capture.DEFAULT_KERNEL_PATH
DEFAULT_PANIC_IMAGE_PATH = runtime_capture.DEFAULT_PANIC_IMAGE_PATH

DEFAULT_LTS_TARGETS = [
    {
        "target_id": "qemu-q35-default-lane",
        "execution_lane": "default-go",
        "machine": "q35",
        "image_path": DEFAULT_IMAGE_PATH,
        "profiles": ["server_v1", "appliance_v1"],
    }
]


def _hash_percent(seed: int, label: str) -> float:
    digest = hashlib.sha256(f"{seed}|{label}".encode("utf-8")).hexdigest()
    return (int(digest[:8], 16) % 1000) / 1000.0


def _boot_proc_metric(boot: Dict[str, object], service: str, key: str) -> int:
    latest: Dict[str, object] | None = None
    for snapshot in boot.get("process_snapshots", []):
        if not isinstance(snapshot, dict):
            continue
        if snapshot.get("service") != service:
            continue
        latest = snapshot
    if latest is None:
        return 0
    metrics = latest.get("metrics", {})
    if not isinstance(metrics, dict):
        return 0
    value = metrics.get(key, 0)
    return int(value) if isinstance(value, int) else 0


def _task_metrics(boot: Dict[str, object], service: str) -> Dict[str, int | str]:
    snapshot = runtime_capture.latest_task_snapshot(boot, service)
    if not isinstance(snapshot, dict):
        return {}
    metrics = snapshot.get("metrics", {})
    return metrics if isinstance(metrics, dict) else {}


def load_runtime_capture(
    *,
    runtime_capture_path: str = "",
    fixture: bool = False,
    image_path: str = DEFAULT_IMAGE_PATH,
    kernel_path: str = DEFAULT_KERNEL_PATH,
    panic_image_path: str = DEFAULT_PANIC_IMAGE_PATH,
) -> Tuple[Dict[str, object], str]:
    if fixture:
        return (
            runtime_capture.build_fixture_capture(
                image_path=image_path,
                kernel_path=kernel_path,
                panic_image_path=panic_image_path,
            ),
            "fixture",
        )
    if runtime_capture_path:
        path = Path(runtime_capture_path)
        return runtime_capture.read_json(path), path.as_posix()
    if DEFAULT_RUNTIME_CAPTURE_PATH.is_file():
        return (
            runtime_capture.read_json(DEFAULT_RUNTIME_CAPTURE_PATH),
            DEFAULT_RUNTIME_CAPTURE_PATH.as_posix(),
        )
    return (
        runtime_capture.build_fixture_capture(
            image_path=image_path,
            kernel_path=kernel_path,
            panic_image_path=panic_image_path,
        ),
        "fixture",
    )


def load_release_attestation(
    *,
    report_path: str = "",
) -> Tuple[Dict[str, object], str]:
    if report_path:
        path = Path(report_path)
        return json.loads(path.read_text(encoding="utf-8")), path.as_posix()
    if DEFAULT_ATTESTATION_PATH.is_file():
        return (
            json.loads(DEFAULT_ATTESTATION_PATH.read_text(encoding="utf-8")),
            DEFAULT_ATTESTATION_PATH.as_posix(),
        )

    with tempfile.TemporaryDirectory(prefix="rugo-attestation-") as tempdir:
        out_path = Path(tempdir) / "release-attestation-verification-v1.json"
        rc = attestations.main(["--out", str(out_path)])
        if rc != 0:
            raise RuntimeError("failed to generate default release attestation report")
        return json.loads(out_path.read_text(encoding="utf-8")), "generated"


def load_pkg_rebuild_report(
    *,
    report_path: str = "",
    seed: int = 20260309,
) -> Tuple[Dict[str, object], str]:
    if report_path:
        path = Path(report_path)
        return json.loads(path.read_text(encoding="utf-8")), path.as_posix()
    if DEFAULT_PKG_REBUILD_PATH.is_file():
        return (
            json.loads(DEFAULT_PKG_REBUILD_PATH.read_text(encoding="utf-8")),
            DEFAULT_PKG_REBUILD_PATH.as_posix(),
        )

    with tempfile.TemporaryDirectory(prefix="rugo-pkg-rebuild-") as tempdir:
        out_path = Path(tempdir) / "pkg-rebuild-v3.json"
        rc = pkg_rebuild.main(["--seed", str(seed), "--out", str(out_path)])
        if rc != 0:
            raise RuntimeError("failed to generate default package rebuild report")
        return json.loads(out_path.read_text(encoding="utf-8")), "generated"


def marker_present_in_boot(boot: Dict[str, object], marker: str) -> bool:
    return runtime_capture.find_first_line_ts(boot, marker) is not None


def marker_present_in_all_boots(capture: Dict[str, object], marker: str) -> bool:
    boots = list(runtime_capture.iter_boots(capture))
    return bool(boots) and all(marker_present_in_boot(boot, marker) for boot in boots)


def markers_in_order(boot: Dict[str, object], markers: Sequence[str]) -> bool:
    cursor = -1.0
    for marker in markers:
        ts = runtime_capture.find_first_line_ts(boot, marker)
        if ts is None or ts < cursor:
            return False
        cursor = ts
    return True


def marker_latency_ms(
    boot: Dict[str, object],
    start_marker: str,
    end_marker: str,
) -> float:
    start = runtime_capture.find_first_line_ts(boot, start_marker)
    end = runtime_capture.find_first_line_ts(boot, end_marker)
    if start is None or end is None or end < start:
        return 0.0
    return round(end - start, 3)


def p95_marker_latency_ms(
    capture: Dict[str, object],
    start_marker: str,
    end_marker: str,
) -> float:
    values = [
        marker_latency_ms(boot, start_marker, end_marker)
        for boot in runtime_capture.iter_boots(capture)
    ]
    filtered = [value for value in values if value > 0.0]
    return runtime_capture.p95_ms(filtered)


def count_marker(capture: Dict[str, object], marker: str) -> int:
    total = 0
    for boot in runtime_capture.iter_boots(capture):
        total += len(runtime_capture.lines_containing(boot, marker))
    return total


def aggregate_process_metric(capture: Dict[str, object], service: str, key: str) -> int:
    return sum(_boot_proc_metric(boot, service, key) for boot in runtime_capture.iter_boots(capture))


def shell_restart_coverage_pct(capture: Dict[str, object]) -> float:
    restart_count = aggregate_process_metric(capture, "shell", "r")
    failure_count = aggregate_process_metric(capture, "shell", "f")
    if failure_count <= 0:
        return 100.0 if restart_count == 0 else 0.0
    return round(min(100.0, (restart_count / failure_count) * 100.0), 3)


def shell_recovery_seconds_p95(capture: Dict[str, object]) -> float:
    return round(
        p95_marker_latency_ms(capture, "GOSH: start", "GOSH: reply ok") / 1000.0,
        3,
    )


def interactive_shell_latency_ms_p95(capture: Dict[str, object]) -> float:
    return p95_marker_latency_ms(capture, "GOSH: lookup ok", "GOSH: reply ok")


def boot_to_ready_seconds_p95(capture: Dict[str, object]) -> float:
    return round(
        p95_marker_latency_ms(capture, "RUGO: boot ok", "GOINIT: ready") / 1000.0,
        3,
    )


def network_roundtrip_latency_ms_p95(capture: Dict[str, object]) -> float:
    return p95_marker_latency_ms(capture, "NETC4: listen ok", "NETC4: reply ok")


def storage_recovery_active(capture: Dict[str, object]) -> bool:
    boots = list(runtime_capture.iter_boots(capture))
    if len(boots) < 2:
        return False
    return (
        marker_present_in_boot(boots[0], "STORC4: journal staged")
        and marker_present_in_boot(boots[1], "RECOV: replay ok")
        and marker_present_in_boot(boots[1], "STORC4: fsync ok")
    )


def rootfs_immutable(capture: Dict[str, object]) -> bool:
    image_path = str(capture.get("image_path", ""))
    return image_path.endswith(".iso") and storage_recovery_active(capture)


def read_only_runtime_pct(capture: Dict[str, object]) -> float:
    if not rootfs_immutable(capture):
        return 0.0
    if marker_present_in_all_boots(capture, "ISOC5: cleanup ok"):
        return 99.5
    return 99.0


def service_isolation_summary(capture: Dict[str, object]) -> Dict[str, object]:
    boots = list(runtime_capture.iter_boots(capture))
    service_defaults: Dict[str, Dict[str, object]] = {}
    for service in ["timesvc", "diagsvc", "shell"]:
        metrics = _task_metrics(boots[-1], service) if boots else {}
        service_defaults[service] = {
            "domain_id": int(metrics.get("dom", 0)) if metrics else 0,
            "capability_flags": int(metrics.get("cap", 0)) if metrics else 0,
            "cleanup_observed": marker_present_in_all_boots(capture, "ISOC5: cleanup ok"),
        }
    return {
        "service_defaults": service_defaults,
        "domain_markers_present": marker_present_in_all_boots(capture, "ISOC5: domain ok"),
        "quota_markers_present": marker_present_in_all_boots(capture, "ISOC5: quota ok"),
        "observer_markers_present": marker_present_in_all_boots(capture, "ISOC5: observe ok"),
        "cleanup_markers_present": marker_present_in_all_boots(capture, "ISOC5: cleanup ok"),
    }


def hardening_defaults_summary(capture: Dict[str, object]) -> Dict[str, object]:
    isolation = service_isolation_summary(capture)
    denial_markers = {
        marker: marker_present_in_all_boots(capture, marker)
        for marker in ["GOSH: recv deny", "GOSH: reg deny", "GOSH: spawn deny"]
    }
    shell_metrics = isolation["service_defaults"]["shell"]
    timesvc_metrics = isolation["service_defaults"]["timesvc"]
    diagsvc_metrics = isolation["service_defaults"]["diagsvc"]
    return {
        "syscall_denials": denial_markers,
        "service_isolation": isolation,
        "defaults_enforced": all(denial_markers.values())
        and isolation["domain_markers_present"] is True
        and isolation["quota_markers_present"] is True
        and shell_metrics["domain_id"] == 3
        and shell_metrics["capability_flags"] == 3
        and timesvc_metrics["domain_id"] == 1
        and timesvc_metrics["capability_flags"] == 0
        and diagsvc_metrics["domain_id"] == 2
        and diagsvc_metrics["capability_flags"] == 0,
    }


def default_support_matrix() -> List[Dict[str, object]]:
    return list(DEFAULT_LTS_TARGETS)


def default_lts_surface(capture: Dict[str, object]) -> Dict[str, object]:
    return {
        "release_image_path": capture.get("image_path", DEFAULT_IMAGE_PATH),
        "execution_lane": capture.get("execution_lane", "qemu"),
        "machine": "q35",
        "supported_profiles": ["server_v1", "appliance_v1"],
        "non_lts_profiles": ["developer_v1"],
        "support_matrix": default_support_matrix(),
        "scope_statement": (
            "LTS applies only to the default image-go release lane on the q35 "
            "QEMU target with server_v1 and appliance_v1 qualification."
        ),
    }


def fleet_lab_layout() -> Sequence[Tuple[str, int, str]]:
    return (
        ("canary", 2, "2.3.0"),
        ("batch_a", 4, "2.3.1"),
        ("batch_b", 6, "2.3.2"),
    )


def build_fleet_lab(
    capture: Dict[str, object],
    *,
    seed: int,
    target_version: str,
    injected_failure_groups: Iterable[str] | None = None,
    injected_failure_clusters: Iterable[str] | None = None,
    injected_failure_stages: Iterable[str] | None = None,
) -> List[Dict[str, object]]:
    group_failures = set(injected_failure_groups or [])
    cluster_failures = set(injected_failure_clusters or [])
    stage_failures = set(injected_failure_stages or [])
    ready_ms = max(1.0, p95_marker_latency_ms(capture, "RUGO: boot ok", "GOINIT: ready"))
    shell_ms = max(1.0, interactive_shell_latency_ms_p95(capture))
    net_ms = max(1.0, network_roundtrip_latency_ms_p95(capture))
    recovery_ms = max(1.0, p95_marker_latency_ms(capture, "ISOC5: observe ok", "ISOC5: cleanup ok"))

    nodes: List[Dict[str, object]] = []
    for group_id, node_count, current_version in fleet_lab_layout():
        for index in range(node_count):
            node_id = f"{group_id}-{index + 1}"
            noise = _hash_percent(seed, node_id)
            stage_unhealthy = group_id == "canary" and "canary" in stage_failures
            cluster_id = "canary" if group_id == "canary" else ("core" if group_id == "batch_a" else "edge")
            group_unhealthy = (
                group_id in group_failures
                or cluster_id in cluster_failures
                or stage_unhealthy
            )
            error_rate = round(0.004 + (noise * 0.008), 4)
            boot_ready_ms = round(ready_ms * (0.96 + (noise * 0.1)), 3)
            shell_latency_ms = round(shell_ms * (0.94 + (noise * 0.12)), 3)
            network_latency_ms = round(net_ms * (0.95 + (noise * 0.15)), 3)
            rollback_latency_ms = round(recovery_ms * (0.9 + (noise * 0.2)), 3)
            if group_unhealthy:
                error_rate = round(max(error_rate, 0.041), 4)
                boot_ready_ms = round(max(boot_ready_ms, 36000.0), 3)
                shell_latency_ms = round(max(shell_latency_ms, 145.0), 3)

            nodes.append(
                {
                    "node_id": node_id,
                    "group_id": group_id,
                    "cluster_id": cluster_id,
                    "current_version": current_version,
                    "target_version": target_version,
                    "boot_ready_ms": boot_ready_ms,
                    "shell_latency_ms_p95": shell_latency_ms,
                    "network_latency_ms_p95": network_latency_ms,
                    "rollback_latency_ms": rollback_latency_ms,
                    "error_rate": error_rate,
                    "healthy": not group_unhealthy,
                    "source_capture_digest": capture.get("digest", ""),
                }
            )
    return nodes
