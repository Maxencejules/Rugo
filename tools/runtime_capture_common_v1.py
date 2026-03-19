#!/usr/bin/env python3
"""Shared helpers for boot-backed runtime capture and provenance."""

from __future__ import annotations

import hashlib
import json
import os
import re
import shutil
import subprocess
import time
import uuid
from pathlib import Path
from typing import Dict, Iterable, List, Sequence, Tuple


BOOTED_RUNTIME_SCHEMA = "rugo.booted_runtime_capture.v1"
DEFAULT_RELEASE_IMAGE_PATH = "out/os-go.iso"
DEFAULT_KERNEL_PATH = "out/kernel-go.elf"
DEFAULT_PANIC_IMAGE_PATH = "out/os-panic.iso"
DEFAULT_MACHINE = "q35"
DEFAULT_TIMEOUT_SECONDS = 20.0
FIXTURE_SEED = 20260318
QEMU_SUCCESS_EXIT_CODES = {0, 1, 99}

COMPONENT_PREFIX_MAP = {
    "RUGO": ("kernel", "kernel"),
    "RECOV": ("kernel", "storage"),
    "BLK": ("kernel", "storage"),
    "STORC4": ("userspace", "storage"),
    "NETC4": ("userspace", "network"),
    "GOINIT": ("userspace", "goinit"),
    "GOSVCM": ("userspace", "gosvcm"),
    "SVC": ("userspace", "gosvcm"),
    "TIMESVC": ("userspace", "timesvc"),
    "DIAGSVC": ("userspace", "diagsvc"),
    "PKGSVC": ("userspace", "pkgsvc"),
    "GOSH": ("userspace", "shell"),
    "DESKBOOT": ("userspace", "desktop_boot"),
    "DESKDISP": ("userspace", "display_runtime"),
    "DESKSEAT": ("userspace", "seat_runtime"),
    "DESKCOMP": ("userspace", "window_compositor"),
    "DESKGUI": ("userspace", "gui_runtime"),
    "DSHELL": ("userspace", "desktop_shell"),
    "DINST": ("userspace", "graphical_installer"),
    "PROC": ("userspace", "diagnostics"),
    "TASK": ("userspace", "diagnostics"),
    "ISOC5": ("userspace", "isolation"),
    "SOAKC5": ("userspace", "isolation"),
    "UPD3": ("userspace", "update"),
    "CAT3": ("userspace", "catalog"),
    "STORX3": ("userspace", "platform"),
}

TASK_RE = re.compile(r"^TASK:\s+(\S+)\s+(.*)$")
PROC_RE = re.compile(r"^PROC:\s+(\S+)\s+(.*)$")
PANIC_CODE_RE = re.compile(r"panic code=0x([0-9A-Fa-f]+)")


def _sha256_text(value: str) -> str:
    return hashlib.sha256(value.encode("utf-8")).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        while True:
            chunk = handle.read(65536)
            if not chunk:
                break
            digest.update(chunk)
    return digest.hexdigest()


def maybe_sha256_file(path: Path, fallback_key: str) -> str:
    if path.is_file():
        return sha256_file(path)
    return _sha256_text(f"missing:{fallback_key}:{path.as_posix()}")


def stable_digest(payload: Dict[str, object]) -> str:
    return hashlib.sha256(
        json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
    ).hexdigest()


def read_json(path: Path) -> Dict[str, object]:
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, payload: Dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")


def posix_path(path: Path | str) -> str:
    return Path(path).as_posix()


def qemu_bin() -> str | None:
    candidates = [
        os.environ.get("QEMU_BIN"),
        shutil.which("qemu-system-x86_64"),
        shutil.which("qemu-system-x86_64.exe"),
    ]
    if os.name == "nt":
        program_files = os.environ.get("ProgramFiles", r"C:\Program Files")
        candidates.append(
            os.path.join(program_files, "qemu", "qemu-system-x86_64.exe")
        )
    else:
        candidates.extend(
            [
                "/mnt/c/Program Files/qemu/qemu-system-x86_64.exe",
                "/mnt/c/Program Files (x86)/qemu/qemu-system-x86_64.exe",
            ]
        )

    for candidate in candidates:
        if candidate and Path(candidate).is_file():
            return candidate
    return None


def qemu_available() -> bool:
    return qemu_bin() is not None


def parse_metric_tokens(raw: str) -> Dict[str, int | str]:
    metrics: Dict[str, int | str] = {}
    for token in raw.split():
        if "=" not in token:
            continue
        key, value = token.split("=", 1)
        if value.isdigit():
            metrics[key] = int(value)
        else:
            metrics[key] = value
    return metrics


def _event_kind(prefix: str, message: str) -> str:
    lowered = message.lower()
    if prefix in {"TASK", "PROC"}:
        return "snapshot"
    if "panic code=" in lowered:
        return "panic"
    if "ready" in lowered:
        return "ready"
    if "start" in lowered or "starting" in lowered or "declared" in lowered:
        return "start"
    if "stop" in lowered or "stopping" in lowered or "stopped" in lowered or "reap" in lowered:
        return "stop"
    if "snapshot" in lowered:
        return "snapshot"
    if "reply" in lowered or "recv" in lowered or "send" in lowered or "connect" in lowered:
        return "io"
    if "err" in lowered or "fail" in lowered:
        return "error"
    return "marker"


def classify_runtime_line(line: str) -> Dict[str, object]:
    prefix, _, tail = line.partition(":")
    prefix = prefix.strip()
    message = tail.strip() if tail else line.strip()
    layer, component = COMPONENT_PREFIX_MAP.get(prefix, ("userspace", prefix.lower()))
    event: Dict[str, object] = {
        "line": line,
        "prefix": prefix,
        "message": message,
        "layer": layer,
        "component": component,
        "event_kind": _event_kind(prefix, message),
    }

    task_match = TASK_RE.match(line)
    if task_match:
        service, metrics_text = task_match.groups()
        event["service"] = service
        event["component"] = service
        event["metrics"] = parse_metric_tokens(metrics_text)
        return event

    proc_match = PROC_RE.match(line)
    if proc_match:
        service, metrics_text = proc_match.groups()
        event["service"] = service
        event["component"] = service
        event["metrics"] = parse_metric_tokens(metrics_text)
        return event

    if prefix == "SVC":
        parts = message.split()
        if parts:
            event["service"] = parts[0]
            event["component"] = parts[0]
    elif prefix in {"TIMESVC", "DIAGSVC", "PKGSVC", "GOINIT", "GOSVCM", "GOSH"}:
        event["service"] = component
    elif prefix in {"STORC4", "NETC4", "ISOC5", "SOAKC5", "UPD3", "CAT3", "STORX3"}:
        event["service"] = component
    elif prefix == "RUGO":
        event["service"] = "kernel"
    return event


def digest_lines(lines: Sequence[Dict[str, object]]) -> str:
    stable = [
        {"ts_ms": round(float(line["ts_ms"]), 3), "line": str(line["line"])}
        for line in lines
    ]
    return stable_digest({"lines": stable})


def event_deltas_ms(lines: Sequence[Dict[str, object]], component: str) -> List[float]:
    component_lines = [
        float(entry["ts_ms"])
        for entry in lines
        if classify_runtime_line(str(entry["line"])).get("component") == component
    ]
    if len(component_lines) < 2:
        return []
    return [
        round(component_lines[index] - component_lines[index - 1], 3)
        for index in range(1, len(component_lines))
    ]


def _quantile(values: Sequence[float], q: float) -> float:
    if not values:
        return 0.0
    ordered = sorted(values)
    index = int(round((len(ordered) - 1) * q))
    return round(float(ordered[index]), 3)


def p95_ms(values: Sequence[float]) -> float:
    return _quantile(values, 0.95)


def _extract_snapshots(
    lines: Sequence[Dict[str, object]],
) -> Tuple[List[Dict[str, object]], List[Dict[str, object]]]:
    process_snapshots: List[Dict[str, object]] = []
    task_snapshots: List[Dict[str, object]] = []
    for entry in lines:
        parsed = classify_runtime_line(str(entry["line"]))
        snapshot = {
            "ts_ms": round(float(entry["ts_ms"]), 3),
            "line": parsed["line"],
            "service": parsed.get("service", ""),
            "metrics": parsed.get("metrics", {}),
        }
        if parsed["prefix"] == "PROC":
            process_snapshots.append(snapshot)
        elif parsed["prefix"] == "TASK":
            task_snapshots.append(snapshot)
    return process_snapshots, task_snapshots


def latest_task_snapshot(
    boot: Dict[str, object],
    service: str,
) -> Dict[str, object] | None:
    latest: Dict[str, object] | None = None
    for snapshot in boot.get("task_snapshots", []):
        if not isinstance(snapshot, dict):
            continue
        if snapshot.get("service") != service:
            continue
        latest = snapshot
    return latest


def find_first_line_ts(boot: Dict[str, object], needle: str) -> float | None:
    for entry in boot.get("serial_lines", []):
        if not isinstance(entry, dict):
            continue
        if needle in str(entry.get("line", "")):
            return round(float(entry.get("ts_ms", 0.0)), 3)
    return None


def lines_containing(boot: Dict[str, object], needle: str) -> List[Dict[str, object]]:
    results: List[Dict[str, object]] = []
    for entry in boot.get("serial_lines", []):
        if not isinstance(entry, dict):
            continue
        if needle in str(entry.get("line", "")):
            results.append(entry)
    return results


def parse_panic_code(lines: Sequence[Dict[str, object]]) -> int:
    for entry in lines:
        match = PANIC_CODE_RE.search(str(entry.get("line", "")))
        if match:
            return int(match.group(1), 16)
    return 0xDEAD


def qemu_capture_lines(
    *,
    image_path: Path,
    timeout_seconds: float = DEFAULT_TIMEOUT_SECONDS,
    machine: str = DEFAULT_MACHINE,
    cpu: str = "qemu64",
    disk_path: Path | None = None,
    disk_device: str = "virtio-blk-pci,drive=disk0,disable-modern=on",
    with_net: bool = False,
    net_device: str = "virtio-net-pci,netdev=n0,disable-modern=on",
) -> Tuple[int, List[Dict[str, object]]]:
    qemu = qemu_bin()
    if qemu is None:
        raise RuntimeError("qemu-system-x86_64 not found")
    if not image_path.is_file():
        raise FileNotFoundError(f"image not found: {image_path}")

    command = [
        qemu,
        "-machine",
        machine,
        "-cpu",
        cpu,
        "-m",
        "128",
        "-serial",
        "stdio",
        "-display",
        "none",
        "-no-reboot",
        "-device",
        "isa-debug-exit,iobase=0xf4,iosize=0x04",
        "-cdrom",
        str(image_path),
    ]
    if disk_path is not None:
        if not disk_path.is_file():
            disk_path.write_bytes(b"\x00" * (1024 * 1024))
        command.extend(
            [
                "-drive",
                f"file={disk_path},format=raw,if=none,id=disk0",
                "-device",
                disk_device,
            ]
        )
    if with_net:
        command.extend(
            [
                "-netdev",
                "user,id=n0",
                "-device",
                net_device,
            ]
        )

    process = subprocess.Popen(
        command,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
    )
    assert process.stdout is not None

    start = time.monotonic()
    lines: List[Dict[str, object]] = []
    while True:
        line = process.stdout.readline()
        if line:
            lines.append(
                {
                    "ts_ms": round((time.monotonic() - start) * 1000.0, 3),
                    "line": line.rstrip("\r\n"),
                }
            )
            continue
        if process.poll() is not None:
            break
        if (time.monotonic() - start) > timeout_seconds:
            process.kill()
            raise TimeoutError(
                f"QEMU timed out after {timeout_seconds:.1f}s while booting {image_path}"
            )
        time.sleep(0.01)

    return int(process.returncode or 0), lines


def _build_boot_entry(
    *,
    capture_id: str,
    boot_index: int,
    boot_profile: str,
    lines: Sequence[Dict[str, object]],
    exit_code: int,
) -> Dict[str, object]:
    process_snapshots, task_snapshots = _extract_snapshots(lines)
    serial_digest = digest_lines(lines)
    stable = {
        "capture_id": capture_id,
        "boot_index": boot_index,
        "boot_profile": boot_profile,
        "serial_digest": serial_digest,
    }
    boot_id = stable_digest(stable)[:16]
    duration_ms = round(
        float(lines[-1]["ts_ms"]) if lines else 0.0,
        3,
    )
    return {
        "boot_id": boot_id,
        "boot_index": boot_index,
        "boot_profile": boot_profile,
        "exit_code": exit_code,
        "duration_ms": duration_ms,
        "serial_line_count": len(lines),
        "serial_digest": serial_digest,
        "serial_lines": list(lines),
        "process_snapshots": process_snapshots,
        "task_snapshots": task_snapshots,
        "panic_code": parse_panic_code(lines),
    }


def _base_capture(
    *,
    image_path: str,
    kernel_path: str,
    panic_image_path: str,
    image_digest: str,
    kernel_digest: str,
    panic_image_digest: str,
    capture_mode: str,
    boots: Sequence[Dict[str, object]],
) -> Dict[str, object]:
    build_id = f"rugo-default-{image_digest[:12]}"
    capture_input = {
        "image_path": image_path,
        "kernel_path": kernel_path,
        "panic_image_path": panic_image_path,
        "image_digest": image_digest,
        "kernel_digest": kernel_digest,
        "panic_image_digest": panic_image_digest,
        "capture_mode": capture_mode,
        "boots": [
            {
                "boot_index": boot["boot_index"],
                "boot_profile": boot["boot_profile"],
                "serial_digest": boot["serial_digest"],
            }
            for boot in boots
        ],
    }
    capture_id = stable_digest(capture_input)
    trace_id = f"trace-qemu-{capture_id[:12]}"
    payload = {
        "schema": BOOTED_RUNTIME_SCHEMA,
        "capture_id": capture_id,
        "build_id": build_id,
        "execution_lane": "qemu",
        "capture_mode": capture_mode,
        "image_path": image_path,
        "image_digest": image_digest,
        "kernel_path": kernel_path,
        "kernel_digest": kernel_digest,
        "panic_image_path": panic_image_path,
        "panic_image_digest": panic_image_digest,
        "trace_id": trace_id,
        "boots": list(boots),
    }
    payload["digest"] = stable_digest(payload)
    payload["trace_digest"] = stable_digest(
        {
            "trace_id": trace_id,
            "capture_id": capture_id,
            "boots": [
                {
                    "boot_id": boot["boot_id"],
                    "serial_digest": boot["serial_digest"],
                }
                for boot in boots
            ],
        }
    )
    return payload


def fixture_boot_lines() -> List[List[Dict[str, object]]]:
    boot1 = [
        (0, "RUGO: boot ok"),
        (24, "STORC4: block ready"),
        (43, "NETC4: nic ready"),
        (70, "GOINIT: start"),
        (102, "GOSVCM: start"),
        (135, "SVC: timesvc declared"),
        (162, "SVC: timesvc starting"),
        (188, "GOSVCM: class timesvc critical"),
        (219, "TIMESVC: start"),
        (251, "SVC: timesvc running"),
        (284, "TIMESVC: ready"),
        (301, "SVC: timesvc ready"),
        (319, "SVC: diagsvc declared"),
        (346, "SVC: diagsvc starting"),
        (373, "GOSVCM: class diagsvc best-effort"),
        (409, "DIAGSVC: start"),
        (438, "SVC: diagsvc running"),
        (472, "DIAGSVC: ready"),
        (489, "SVC: diagsvc ready"),
        (506, "SVC: pkgsvc declared"),
        (531, "SVC: pkgsvc starting"),
        (562, "GOSVCM: class pkgsvc best-effort"),
        (596, "PKGSVC: start"),
        (622, "SVC: pkgsvc running"),
        (647, "PKGSVC: ready"),
        (664, "SVC: pkgsvc ready"),
        (673, "SVC: shell declared"),
        (698, "SVC: shell starting"),
        (729, "GOSVCM: class shell best-effort"),
        (763, "GOSH: start"),
        (789, "SVC: shell running"),
        (826, "GOSH: lookup ok"),
        (845, "GOSH: recv deny"),
        (857, "GOSH: reg deny"),
        (869, "TIMESVC: req ok"),
        (885, "GOSH: spawn deny"),
        (891, "SVC: shell ready"),
        (897, "TIMESVC: time ok"),
        (928, "GOSH: reply ok"),
        (965, "DIAGSVC: snapshot"),
        (989, "PROC: timesvc s=1 r=0 f=0 x=0 tick=41 svc=ready res=online"),
        (1004, "PROC: diagsvc s=1 r=0 f=0 x=0 tick=42 svc=ready res=online"),
        (1020, "PROC: shell s=3 r=2 f=2 x=2 tick=43 svc=ready res=online"),
        (1037, "PROC: pkgsvc s=1 r=0 f=0 x=0 tick=44 svc=ready res=online"),
        (1066, "TASK: timesvc tid=1 parent=0 cls=critical st=blocked run=16 y=0 blk=2 tx=6 rx=3 ep=1 dom=1 cap=0 fd=0 sock=0"),
        (1088, "TASK: diagsvc tid=2 parent=0 cls=best-effort st=running run=11 y=1 blk=1 tx=3 rx=5 ep=1 dom=2 cap=0 fd=0 sock=0"),
        (1114, "TASK: shell tid=3 parent=0 cls=best-effort st=blocked run=18 y=4 blk=3 tx=4 rx=4 ep=1 dom=3 cap=3 fd=1 sock=3"),
        (1136, "TASK: pkgsvc tid=4 parent=0 cls=best-effort st=blocked run=9 y=1 blk=2 tx=0 rx=0 ep=1 dom=4 cap=1 fd=0 sock=0"),
        (1161, "GOSH: diag ok"),
        (1195, "STORC4: journal staged"),
        (1224, "NETC4: ifcfg ok"),
        (1252, "NETC4: route ok"),
        (1283, "NETC4: listen ok"),
        (1314, "NETC4: connect ok"),
        (1342, "NETC4: accept ok"),
        (1373, "NETC4: recv ok"),
        (1401, "NETC4: reply ok"),
        (1432, "ISOC5: domain ok"),
        (1460, "ISOC5: quota ok"),
        (1494, "DIAGSVC: snapshot"),
        (1518, "PROC: timesvc s=1 r=0 f=0 x=0 tick=45 svc=ready res=online"),
        (1533, "PROC: diagsvc s=1 r=0 f=0 x=0 tick=46 svc=ready res=online"),
        (1549, "PROC: shell s=3 r=2 f=2 x=2 tick=47 svc=ready res=online"),
        (1566, "PROC: pkgsvc s=1 r=0 f=0 x=0 tick=48 svc=ready res=online"),
        (1595, "TASK: timesvc tid=1 parent=0 cls=critical st=blocked run=20 y=0 blk=2 tx=8 rx=4 ep=1 dom=1 cap=0 fd=0 sock=0"),
        (1617, "TASK: diagsvc tid=2 parent=0 cls=best-effort st=running run=14 y=1 blk=1 tx=4 rx=7 ep=1 dom=2 cap=0 fd=0 sock=0"),
        (1643, "TASK: shell tid=3 parent=0 cls=best-effort st=blocked run=22 y=6 blk=4 tx=5 rx=5 ep=1 dom=3 cap=3 fd=1 sock=3"),
        (1665, "TASK: pkgsvc tid=4 parent=0 cls=best-effort st=blocked run=11 y=2 blk=3 tx=0 rx=0 ep=1 dom=4 cap=1 fd=0 sock=0"),
        (1693, "ISOC5: observe ok"),
        (1728, "SOAKC5: mixed ok"),
        (1756, "UPD3: metadata ok"),
        (1782, "CAT3: catalog ok"),
        (1808, "CAT3: install base ok"),
        (1835, "CAT3: install net ok"),
        (1860, "CAT3: telemetry ok"),
        (1889, "STORX3: snapshot ok"),
        (1916, "STORX3: xattr ok"),
        (1940, "STORX3: cap ok"),
        (1967, "GOSH: pkg ok"),
        (1979, "GOSVCM: phase shutdown"),
        (1994, "GOSVCM: stop pkgsvc"),
        (2021, "SVC: pkgsvc stopping"),
        (2047, "PKGSVC: stop"),
        (2076, "SVC: pkgsvc stopped"),
        (2107, "GOSVCM: reap pkgsvc stopped res=ordered-stop"),
        (2138, "GOSVCM: stop diagsvc"),
        (2165, "SVC: diagsvc stopping"),
        (2191, "DIAGSVC: stop"),
        (2220, "SVC: diagsvc stopped"),
        (2251, "GOSVCM: reap diagsvc stopped res=ordered-stop"),
        (2283, "ISOC5: cleanup ok"),
        (2310, "GOINIT: result shutdown-clean"),
        (2324, "GOINIT: ready"),
        (2338, "RUGO: halt ok"),
    ]
    boot2 = [
        (0, "RUGO: boot ok"),
        (18, "STORC4: block ready"),
        (35, "RECOV: replay ok"),
        (57, "NETC4: nic ready"),
        (83, "GOINIT: start"),
        (109, "GOSVCM: start"),
        (139, "SVC: timesvc declared"),
        (162, "SVC: timesvc starting"),
        (186, "GOSVCM: class timesvc critical"),
        (214, "TIMESVC: start"),
        (241, "SVC: timesvc running"),
        (267, "TIMESVC: ready"),
        (281, "SVC: timesvc ready"),
        (294, "SVC: diagsvc declared"),
        (317, "SVC: diagsvc starting"),
        (340, "GOSVCM: class diagsvc best-effort"),
        (367, "DIAGSVC: start"),
        (394, "SVC: diagsvc running"),
        (423, "DIAGSVC: ready"),
        (437, "SVC: diagsvc ready"),
        (447, "SVC: pkgsvc declared"),
        (470, "SVC: pkgsvc starting"),
        (494, "GOSVCM: class pkgsvc best-effort"),
        (521, "PKGSVC: start"),
        (548, "SVC: pkgsvc running"),
        (572, "PKGSVC: ready"),
        (586, "SVC: pkgsvc ready"),
        (596, "SVC: shell declared"),
        (622, "SVC: shell starting"),
        (648, "GOSVCM: class shell best-effort"),
        (674, "GOSH: start"),
        (702, "SVC: shell running"),
        (736, "GOSH: lookup ok"),
        (752, "GOSH: recv deny"),
        (765, "STORC4: state ok"),
        (781, "GOSH: reg deny"),
        (797, "BLK: flush ordered"),
        (813, "GOSH: spawn deny"),
        (820, "SVC: shell ready"),
        (827, "STORC4: fsync ok"),
        (855, "TIMESVC: req ok"),
        (883, "TIMESVC: time ok"),
        (910, "GOSH: reply ok"),
        (941, "DIAGSVC: snapshot"),
        (963, "PROC: timesvc s=1 r=0 f=0 x=0 tick=49 svc=ready res=online"),
        (977, "PROC: diagsvc s=1 r=0 f=0 x=0 tick=50 svc=ready res=online"),
        (992, "PROC: shell s=3 r=2 f=2 x=2 tick=51 svc=ready res=online"),
        (1008, "PROC: pkgsvc s=1 r=0 f=0 x=0 tick=52 svc=ready res=online"),
        (1035, "TASK: timesvc tid=1 parent=0 cls=critical st=blocked run=23 y=0 blk=2 tx=10 rx=5 ep=1 dom=1 cap=0 fd=0 sock=0"),
        (1056, "TASK: diagsvc tid=2 parent=0 cls=best-effort st=running run=16 y=1 blk=1 tx=5 rx=9 ep=1 dom=2 cap=0 fd=0 sock=0"),
        (1081, "TASK: shell tid=3 parent=0 cls=best-effort st=blocked run=26 y=8 blk=5 tx=6 rx=6 ep=1 dom=3 cap=3 fd=1 sock=3"),
        (1103, "TASK: pkgsvc tid=4 parent=0 cls=best-effort st=blocked run=12 y=2 blk=3 tx=0 rx=0 ep=1 dom=4 cap=1 fd=0 sock=0"),
        (1131, "GOSH: diag ok"),
        (1160, "NETC4: ifcfg ok"),
        (1187, "NETC4: route ok"),
        (1213, "NETC4: listen ok"),
        (1242, "NETC4: connect ok"),
        (1269, "NETC4: accept ok"),
        (1298, "NETC4: recv ok"),
        (1325, "NETC4: reply ok"),
        (1353, "ISOC5: domain ok"),
        (1378, "ISOC5: quota ok"),
        (1408, "DIAGSVC: snapshot"),
        (1431, "PROC: timesvc s=1 r=0 f=0 x=0 tick=53 svc=ready res=online"),
        (1445, "PROC: diagsvc s=1 r=0 f=0 x=0 tick=54 svc=ready res=online"),
        (1459, "PROC: shell s=3 r=2 f=2 x=2 tick=55 svc=ready res=online"),
        (1475, "PROC: pkgsvc s=1 r=0 f=0 x=0 tick=56 svc=ready res=online"),
        (1502, "TASK: timesvc tid=1 parent=0 cls=critical st=blocked run=27 y=0 blk=2 tx=12 rx=6 ep=1 dom=1 cap=0 fd=0 sock=0"),
        (1523, "TASK: diagsvc tid=2 parent=0 cls=best-effort st=running run=19 y=1 blk=1 tx=6 rx=11 ep=1 dom=2 cap=0 fd=0 sock=0"),
        (1549, "TASK: shell tid=3 parent=0 cls=best-effort st=blocked run=30 y=10 blk=6 tx=7 rx=7 ep=1 dom=3 cap=3 fd=1 sock=3"),
        (1571, "TASK: pkgsvc tid=4 parent=0 cls=best-effort st=blocked run=14 y=3 blk=4 tx=0 rx=0 ep=1 dom=4 cap=1 fd=0 sock=0"),
        (1601, "ISOC5: observe ok"),
        (1631, "SOAKC5: mixed ok"),
        (1657, "UPD3: rotate ok"),
        (1683, "UPD3: metadata ok"),
        (1710, "CAT3: catalog ok"),
        (1736, "CAT3: stage ok"),
        (1763, "CAT3: install media ok"),
        (1788, "CAT3: telemetry ok"),
        (1815, "STORX3: resize ok"),
        (1842, "STORX3: reflink ok"),
        (1868, "STORX3: xattr ok"),
        (1893, "STORX3: cap ok"),
        (1919, "UPD3: rollback ok"),
        (1944, "UPD3: apply ok"),
        (1971, "GOSH: pkg ok"),
        (1983, "GOSVCM: phase shutdown"),
        (1998, "GOSVCM: stop pkgsvc"),
        (2023, "SVC: pkgsvc stopping"),
        (2047, "PKGSVC: stop"),
        (2073, "SVC: pkgsvc stopped"),
        (2102, "GOSVCM: reap pkgsvc stopped res=ordered-stop"),
        (2128, "GOSVCM: stop diagsvc"),
        (2153, "SVC: diagsvc stopping"),
        (2177, "DIAGSVC: stop"),
        (2203, "SVC: diagsvc stopped"),
        (2232, "GOSVCM: reap diagsvc stopped res=ordered-stop"),
        (2258, "ISOC5: cleanup ok"),
        (2282, "GOINIT: result shutdown-clean"),
        (2296, "GOINIT: ready"),
        (2306, "RUGO: halt ok"),
    ]
    return [
        [{"ts_ms": round(ts, 3), "line": line} for ts, line in boot1],
        [{"ts_ms": round(ts, 3), "line": line} for ts, line in boot2],
    ]


def build_fixture_capture(
    *,
    image_path: str = DEFAULT_RELEASE_IMAGE_PATH,
    kernel_path: str = DEFAULT_KERNEL_PATH,
    panic_image_path: str = DEFAULT_PANIC_IMAGE_PATH,
) -> Dict[str, object]:
    image_path_obj = Path(image_path)
    kernel_path_obj = Path(kernel_path)
    panic_image_path_obj = Path(panic_image_path)
    image_digest = maybe_sha256_file(image_path_obj, "fixture-image")
    kernel_digest = maybe_sha256_file(kernel_path_obj, "fixture-kernel")
    panic_image_digest = maybe_sha256_file(panic_image_path_obj, "fixture-panic-image")
    provisional = _base_capture(
        image_path=posix_path(image_path_obj),
        kernel_path=posix_path(kernel_path_obj),
        panic_image_path=posix_path(panic_image_path_obj),
        image_digest=image_digest,
        kernel_digest=kernel_digest,
        panic_image_digest=panic_image_digest,
        capture_mode="fixture",
        boots=[],
    )
    capture_id = str(provisional["capture_id"])
    boots = [
        _build_boot_entry(
            capture_id=capture_id,
            boot_index=index + 1,
            boot_profile=profile,
            lines=lines,
            exit_code=0,
        )
        for index, (profile, lines) in enumerate(
            zip(["cold_boot", "replay_boot"], fixture_boot_lines())
        )
    ]
    payload = _base_capture(
        image_path=posix_path(image_path_obj),
        kernel_path=posix_path(kernel_path_obj),
        panic_image_path=posix_path(panic_image_path_obj),
        image_digest=image_digest,
        kernel_digest=kernel_digest,
        panic_image_digest=panic_image_digest,
        capture_mode="fixture",
        boots=boots,
    )
    payload["fixture_seed"] = FIXTURE_SEED
    payload["created_utc"] = "2026-03-18T00:00:00Z"
    payload["boot_profiles"] = [boot["boot_profile"] for boot in boots]
    payload["panic_boot_id"] = f"panic-{payload['capture_id'][:12]}"
    return payload


def collect_booted_runtime(
    *,
    image_path: str = DEFAULT_RELEASE_IMAGE_PATH,
    kernel_path: str = DEFAULT_KERNEL_PATH,
    panic_image_path: str = DEFAULT_PANIC_IMAGE_PATH,
    machine: str = DEFAULT_MACHINE,
    cpu: str = "qemu64",
    timeout_seconds: float = DEFAULT_TIMEOUT_SECONDS,
    disk_device: str = "virtio-blk-pci,drive=disk0,disable-modern=on",
    net_device: str = "virtio-net-pci,netdev=n0,disable-modern=on",
) -> Dict[str, object]:
    image_path_obj = Path(image_path)
    kernel_path_obj = Path(kernel_path)
    panic_image_path_obj = Path(panic_image_path)
    image_digest = maybe_sha256_file(image_path_obj, "release-image")
    kernel_digest = maybe_sha256_file(kernel_path_obj, "release-kernel")
    panic_image_digest = maybe_sha256_file(panic_image_path_obj, "panic-image")
    provisional = _base_capture(
        image_path=posix_path(image_path_obj),
        kernel_path=posix_path(kernel_path_obj),
        panic_image_path=posix_path(panic_image_path_obj),
        image_digest=image_digest,
        kernel_digest=kernel_digest,
        panic_image_digest=panic_image_digest,
        capture_mode="booted_qemu",
        boots=[],
    )
    capture_id = str(provisional["capture_id"])

    temp_root = image_path_obj.resolve().parent
    temp_root.mkdir(parents=True, exist_ok=True)
    disk_path = temp_root / f"rugo-runtime-capture-{uuid.uuid4().hex}.img"
    boots: List[Dict[str, object]] = []
    try:
        for index, profile in enumerate(["cold_boot", "replay_boot"], start=1):
            exit_code, lines = qemu_capture_lines(
                image_path=image_path_obj,
                timeout_seconds=timeout_seconds,
                machine=machine,
                cpu=cpu,
                disk_path=disk_path,
                disk_device=disk_device,
                with_net=True,
                net_device=net_device,
            )
            if exit_code not in QEMU_SUCCESS_EXIT_CODES:
                raise RuntimeError(
                    f"booted runtime capture failed for {image_path_obj} with exit code {exit_code}"
                )
            boots.append(
                _build_boot_entry(
                    capture_id=capture_id,
                    boot_index=index,
                    boot_profile=profile,
                    lines=lines,
                    exit_code=exit_code,
                )
            )
    finally:
        if disk_path.is_file():
            disk_path.unlink()

    payload = _base_capture(
        image_path=posix_path(image_path_obj),
        kernel_path=posix_path(kernel_path_obj),
        panic_image_path=posix_path(panic_image_path_obj),
        image_digest=image_digest,
        kernel_digest=kernel_digest,
        panic_image_digest=panic_image_digest,
        capture_mode="booted_qemu",
        boots=boots,
    )
    payload["created_utc"] = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
    payload["machine"] = machine
    payload["cpu"] = cpu
    payload["timeout_seconds"] = timeout_seconds
    payload["disk_device"] = disk_device
    payload["net_device"] = net_device
    payload["boot_profiles"] = [boot["boot_profile"] for boot in boots]
    payload["panic_boot_id"] = f"panic-{payload['capture_id'][:12]}"
    return payload


def build_panic_fixture(
    *,
    release_image_path: str = DEFAULT_RELEASE_IMAGE_PATH,
    kernel_path: str = DEFAULT_KERNEL_PATH,
    panic_image_path: str = DEFAULT_PANIC_IMAGE_PATH,
) -> Dict[str, object]:
    release_image = Path(release_image_path)
    kernel = Path(kernel_path)
    panic_image = Path(panic_image_path)
    serial_lines = [
        {"ts_ms": 0.0, "line": "RUGO: boot ok"},
        {"ts_ms": 48.0, "line": "RUGO: panic code=0xDEAD"},
    ]
    serial_digest = digest_lines(serial_lines)
    panic_trace_id = f"trace-panic-{serial_digest[:12]}"
    return {
        "schema": "rugo.panic_capture_fixture.v1",
        "release_image_path": posix_path(release_image),
        "release_image_digest": maybe_sha256_file(release_image, "fixture-release-image"),
        "kernel_path": posix_path(kernel),
        "kernel_digest": maybe_sha256_file(kernel, "fixture-release-kernel"),
        "panic_image_path": posix_path(panic_image),
        "panic_image_digest": maybe_sha256_file(panic_image, "fixture-panic-image"),
        "panic_boot_id": f"panic-{serial_digest[:12]}",
        "panic_trace_id": panic_trace_id,
        "panic_trace_digest": stable_digest(
            {"panic_trace_id": panic_trace_id, "serial_digest": serial_digest}
        ),
        "panic_code": 0xDEAD,
        "serial_digest": serial_digest,
        "serial_lines": serial_lines,
        "capture_mode": "fixture",
        "build_id": f"rugo-default-{maybe_sha256_file(release_image, 'fixture-release-image')[:12]}",
    }


def collect_panic_capture(
    *,
    release_image_path: str = DEFAULT_RELEASE_IMAGE_PATH,
    kernel_path: str = DEFAULT_KERNEL_PATH,
    panic_image_path: str = DEFAULT_PANIC_IMAGE_PATH,
    machine: str = DEFAULT_MACHINE,
    timeout_seconds: float = DEFAULT_TIMEOUT_SECONDS,
) -> Dict[str, object]:
    release_image = Path(release_image_path)
    kernel = Path(kernel_path)
    panic_image = Path(panic_image_path)
    exit_code, serial_lines = qemu_capture_lines(
        image_path=panic_image,
        timeout_seconds=timeout_seconds,
        machine=machine,
    )
    if exit_code not in QEMU_SUCCESS_EXIT_CODES:
        raise RuntimeError(
            f"panic capture failed for {panic_image} with exit code {exit_code}"
        )
    serial_digest = digest_lines(serial_lines)
    panic_trace_id = f"trace-panic-{serial_digest[:12]}"
    return {
        "schema": "rugo.panic_capture_fixture.v1",
        "release_image_path": posix_path(release_image),
        "release_image_digest": maybe_sha256_file(release_image, "release-image"),
        "kernel_path": posix_path(kernel),
        "kernel_digest": maybe_sha256_file(kernel, "release-kernel"),
        "panic_image_path": posix_path(panic_image),
        "panic_image_digest": maybe_sha256_file(panic_image, "panic-image"),
        "panic_boot_id": f"panic-{serial_digest[:12]}",
        "panic_trace_id": panic_trace_id,
        "panic_trace_digest": stable_digest(
            {"panic_trace_id": panic_trace_id, "serial_digest": serial_digest}
        ),
        "panic_code": parse_panic_code(serial_lines),
        "serial_digest": serial_digest,
        "serial_lines": serial_lines,
        "capture_mode": "booted_qemu",
        "build_id": f"rugo-default-{maybe_sha256_file(release_image, 'release-image')[:12]}",
    }


def shared_provenance(
    *,
    collector: str,
    command: str,
    release_image_path: str,
    release_image_digest: str,
    kernel_path: str,
    kernel_digest: str,
    build_id: str,
    capture_mode: str,
    runtime_capture_path: str = "",
    runtime_capture_digest: str = "",
    boot_id: str = "",
    trace_id: str = "",
) -> Dict[str, object]:
    return {
        "collector": collector,
        "command": command,
        "capture_mode": capture_mode,
        "release_image_path": release_image_path,
        "release_image_digest": release_image_digest,
        "kernel_path": kernel_path,
        "kernel_digest": kernel_digest,
        "build_id": build_id,
        "runtime_capture_path": runtime_capture_path,
        "runtime_capture_digest": runtime_capture_digest,
        "boot_id": boot_id,
        "trace_id": trace_id,
    }


def component_event_count(boot: Dict[str, object], component: str) -> int:
    total = 0
    for entry in boot.get("serial_lines", []):
        if not isinstance(entry, dict):
            continue
        parsed = classify_runtime_line(str(entry.get("line", "")))
        if parsed.get("component") == component:
            total += 1
    return total


def iter_boots(capture: Dict[str, object]) -> Iterable[Dict[str, object]]:
    for boot in capture.get("boots", []):
        if isinstance(boot, dict):
            yield boot
