#!/usr/bin/env python3
"""Collect M29 observability trace bundle artifacts from booted runtime capture."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
from typing import Dict, List, Set

import runtime_capture_common_v1 as runtime_capture


SCHEMA = "rugo.trace_bundle.v2"
CONTRACT_ID = "rugo.observability_contract.v2"
DEFAULT_COMPONENTS = [
    "goinit",
    "gosvcm",
    "timesvc",
    "diagsvc",
    "shell",
    "storage",
    "network",
    "isolation",
]


def _known_components() -> Set[str]:
    return set(DEFAULT_COMPONENTS)


def _collect_injected(values: List[str], arg_name: str) -> Set[str]:
    injected = {value.strip() for value in values if value.strip()}
    unknown = sorted(injected - _known_components())
    if unknown:
        raise ValueError(f"unknown components in {arg_name}: {', '.join(unknown)}")
    return injected


def _structured_logs(capture: Dict[str, object]) -> List[Dict[str, object]]:
    logs: List[Dict[str, object]] = []
    for boot in runtime_capture.iter_boots(capture):
        boot_id = str(boot.get("boot_id", ""))
        boot_profile = str(boot.get("boot_profile", ""))
        for entry in boot.get("serial_lines", []):
            if not isinstance(entry, dict):
                continue
            parsed = runtime_capture.classify_runtime_line(str(entry.get("line", "")))
            logs.append(
                {
                    "boot_id": boot_id,
                    "boot_profile": boot_profile,
                    "ts_ms": round(float(entry.get("ts_ms", 0.0)), 3),
                    "line": parsed["line"],
                    "prefix": parsed["prefix"],
                    "component": parsed["component"],
                    "layer": parsed["layer"],
                    "event_kind": parsed["event_kind"],
                    "service": parsed.get("service", ""),
                    "message": parsed["message"],
                }
            )
    return logs


def _latency_samples_ms(capture: Dict[str, object], component: str) -> List[float]:
    samples: List[float] = []
    for boot in runtime_capture.iter_boots(capture):
        samples.extend(runtime_capture.event_deltas_ms(boot.get("serial_lines", []), component))
    return samples


def collect_trace_bundle(
    runtime_capture_payload: Dict[str, object],
    window_seconds: int,
    inject_errors: Set[str] | None = None,
    inject_drops: Set[str] | None = None,
) -> Dict[str, object]:
    error_components = set() if inject_errors is None else set(inject_errors)
    drop_components = set() if inject_drops is None else set(inject_drops)
    structured_logs = _structured_logs(runtime_capture_payload)
    sources: List[Dict[str, object]] = []

    total_spans = 0
    total_errors = 0
    total_dropped_spans = 0

    for component in DEFAULT_COMPONENTS:
        scoped = [entry for entry in structured_logs if entry["component"] == component]
        error_events = sum(1 for entry in scoped if entry["event_kind"] == "error")
        if component in error_components:
            error_events += 1
        dropped_spans = 1 if component in drop_components else 0
        latency_p95_ms = runtime_capture.p95_ms(_latency_samples_ms(runtime_capture_payload, component))
        source = {
            "component": component,
            "layer": scoped[0]["layer"] if scoped else "userspace",
            "boot_profiles": sorted({entry["boot_profile"] for entry in scoped}),
            "span_count": len(scoped),
            "event_count": len(scoped),
            "error_events": error_events,
            "dropped_spans": dropped_spans,
            "latency_p95_ms": latency_p95_ms,
            "status": "degraded" if (error_events or dropped_spans) else "ok",
        }
        sources.append(source)

        total_spans += source["span_count"]
        total_errors += error_events
        total_dropped_spans += dropped_spans

    stable_payload = {
        "schema": SCHEMA,
        "runtime_capture_digest": runtime_capture_payload.get("digest", ""),
        "sources": [
            {
                "component": source["component"],
                "span_count": source["span_count"],
                "error_events": source["error_events"],
                "dropped_spans": source["dropped_spans"],
                "latency_p95_ms": source["latency_p95_ms"],
            }
            for source in sources
        ],
    }
    digest = hashlib.sha256(
        json.dumps(stable_payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
    ).hexdigest()

    return {
        "schema": SCHEMA,
        "contract_id": CONTRACT_ID,
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "window_seconds": window_seconds,
        "runtime_capture_schema": runtime_capture_payload.get("schema", ""),
        "runtime_capture_digest": runtime_capture_payload.get("digest", ""),
        "release_image_path": runtime_capture_payload.get("image_path", ""),
        "release_image_digest": runtime_capture_payload.get("image_digest", ""),
        "trace_id": runtime_capture_payload.get("trace_id", ""),
        "trace_digest": runtime_capture_payload.get("trace_digest", ""),
        "sources": sources,
        "structured_logs": structured_logs,
        "totals": {
            "total_components": len(sources),
            "total_spans": total_spans,
            "total_errors": total_errors,
            "total_dropped_spans": total_dropped_spans,
            "total_logs": len(structured_logs),
        },
        "digest": digest,
    }


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--runtime-capture", required=True)
    parser.add_argument("--window-seconds", type=int, default=300)
    parser.add_argument("--max-errors", type=int, default=0)
    parser.add_argument("--max-dropped-spans", type=int, default=0)
    parser.add_argument(
        "--inject-error",
        action="append",
        default=[],
        help="force a component to report an error event",
    )
    parser.add_argument(
        "--inject-drop",
        action="append",
        default=[],
        help="force a component to report dropped spans",
    )
    parser.add_argument("--out", default="out/trace-bundle-v2.json")
    return parser


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)

    try:
        inject_errors = _collect_injected(args.inject_error, "--inject-error")
        inject_drops = _collect_injected(args.inject_drop, "--inject-drop")
    except ValueError as exc:
        print(f"error: {exc}")
        return 2

    capture = runtime_capture.read_json(Path(args.runtime_capture))
    report = collect_trace_bundle(
        runtime_capture_payload=capture,
        window_seconds=args.window_seconds,
        inject_errors=inject_errors,
        inject_drops=inject_drops,
    )
    totals = report["totals"]
    report["max_errors"] = args.max_errors
    report["max_dropped_spans"] = args.max_dropped_spans
    report["gate_pass"] = (
        totals["total_errors"] <= args.max_errors
        and totals["total_dropped_spans"] <= args.max_dropped_spans
    )

    out_path = Path(args.out)
    runtime_capture.write_json(out_path, report)

    print(f"trace-bundle-report: {out_path}")
    print(f"total_errors: {totals['total_errors']}")
    print(f"total_dropped_spans: {totals['total_dropped_spans']}")
    return 0 if report["gate_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
