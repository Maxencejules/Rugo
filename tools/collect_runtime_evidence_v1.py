#!/usr/bin/env python3
"""Collect runtime-backed evidence artifacts for M40."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
from typing import Dict, List, Sequence, Set

import runtime_capture_common_v1 as runtime_capture


SCHEMA = "rugo.runtime_evidence_report.v1"
EVIDENCE_INTEGRITY_POLICY_ID = "rugo.evidence_integrity_policy.v1"
RUNTIME_EVIDENCE_SCHEMA_ID = "rugo.runtime_evidence_schema.v1"
GATE_PROVENANCE_POLICY_ID = "rugo.gate_provenance_policy.v1"


@dataclass(frozen=True)
class CheckSpec:
    check_id: str
    domain: str
    metric_key: str
    operator: str
    threshold: float


CHECKS: Sequence[CheckSpec] = (
    CheckSpec("runtime_item_count", "execution", "runtime_item_count", "min", 7.0),
    CheckSpec("runtime_capture_ratio", "execution", "runtime_capture_ratio", "min", 1.0),
    CheckSpec("qemu_trace_presence_ratio", "execution", "qemu_trace_presence_ratio", "min", 1.0),
    CheckSpec("panic_trace_presence_ratio", "execution", "panic_trace_presence_ratio", "min", 1.0),
    CheckSpec("trace_linkage_ratio", "execution", "trace_linkage_ratio", "min", 1.0),
    CheckSpec("default_image_binding_ratio", "provenance", "default_image_binding_ratio", "min", 1.0),
    CheckSpec("boot_instance_binding_ratio", "provenance", "boot_instance_binding_ratio", "min", 1.0),
    CheckSpec("provenance_fields_ratio", "provenance", "provenance_fields_ratio", "min", 1.0),
    CheckSpec("unsigned_artifact_count", "provenance", "unsigned_artifact_count", "max", 0.0),
    CheckSpec("synthetic_evidence_ratio", "synthetic", "synthetic_evidence_ratio", "max", 0.0),
    CheckSpec("synthetic_only_artifacts", "synthetic", "synthetic_only_artifacts", "max", 0.0),
    CheckSpec("detached_trace_count", "synthetic", "detached_trace_count", "max", 0.0),
)


def _known_checks() -> Set[str]:
    return {spec.check_id for spec in CHECKS}


def _normalize_failures(values: Sequence[str]) -> Set[str]:
    failures = {value.strip() for value in values if value.strip()}
    unknown = sorted(failures - _known_checks())
    if unknown:
        raise ValueError(f"unknown check ids in --inject-failure: {', '.join(unknown)}")
    return failures


def _read_json(path: str) -> Dict[str, object]:
    return runtime_capture.read_json(Path(path))


def _artifact_digest(payload: Dict[str, object]) -> str:
    if "digest" in payload and isinstance(payload["digest"], str):
        return str(payload["digest"])
    stable = dict(payload)
    stable.pop("created_utc", None)
    return runtime_capture.stable_digest(stable)


def _trace_rows(
    runtime_capture_payload: Dict[str, object],
    crash_dump_payload: Dict[str, object],
    runtime_capture_path: str,
    crash_dump_path: str,
) -> List[Dict[str, object]]:
    crash_provenance = crash_dump_payload.get("runtime_provenance", {})
    if not isinstance(crash_provenance, dict):
        crash_provenance = {}
    return [
        {
            "trace_id": runtime_capture_payload["trace_id"],
            "execution_lane": "qemu",
            "capture_kind": "serial+structured-log",
            "trace_path": runtime_capture_path,
            "trace_digest": runtime_capture_payload["trace_digest"],
            "release_image_path": runtime_capture_payload["image_path"],
            "boot_id": runtime_capture_payload.get("capture_id", ""),
        },
        {
            "trace_id": crash_provenance.get("panic_trace_id", ""),
            "execution_lane": "panic",
            "capture_kind": "serial+panic-marker",
            "trace_path": crash_dump_path,
            "trace_digest": crash_provenance.get("panic_trace_digest", ""),
            "release_image_path": crash_provenance.get("release_image_path", ""),
            "boot_id": crash_provenance.get("panic_boot_id", ""),
        },
    ]


def _shared_signature(payload: Dict[str, object], valid: bool = True) -> Dict[str, object]:
    return {
        "algorithm": "sha256",
        "valid": valid,
        "digest": _artifact_digest(payload),
    }


def _make_item(
    *,
    artifact_id: str,
    execution_lane: str,
    trace_id: str,
    trace_digest: str,
    provenance: Dict[str, object],
    payload: Dict[str, object],
    runtime_kind: str,
) -> Dict[str, object]:
    return {
        "artifact_id": artifact_id,
        "execution_lane": execution_lane,
        "runtime_source": {
            "kind": runtime_kind,
            "collector": provenance.get("collector", ""),
            "command": provenance.get("command", ""),
        },
        "synthetic": False,
        "trace_id": trace_id,
        "trace_digest": trace_digest,
        "artifact_digest": _artifact_digest(payload),
        "provenance": provenance,
        "signature": _shared_signature(payload),
    }


def _required_provenance_fields() -> Set[str]:
    return {
        "collector",
        "command",
        "capture_mode",
        "release_image_path",
        "release_image_digest",
        "build_id",
        "boot_id",
        "trace_id",
    }


def _mark_synthetic(item: Dict[str, object]) -> None:
    item["synthetic"] = True
    runtime_source = item.get("runtime_source", {})
    if isinstance(runtime_source, dict):
        runtime_source["kind"] = "synthetic_model"
    provenance = item.get("provenance", {})
    if isinstance(provenance, dict):
        provenance["capture_mode"] = "synthetic"


def _apply_injected_failures(
    *,
    injected_failures: Set[str],
    traces: List[Dict[str, object]],
    evidence_items: List[Dict[str, object]],
    release_image_digest: str,
) -> None:
    if not evidence_items:
        return

    if "runtime_item_count" in injected_failures and len(evidence_items) > 1:
        evidence_items.pop()

    if "runtime_capture_ratio" in injected_failures:
        _mark_synthetic(evidence_items[0])

    if "synthetic_evidence_ratio" in injected_failures:
        _mark_synthetic(evidence_items[0])

    if "synthetic_only_artifacts" in injected_failures:
        for item in evidence_items:
            _mark_synthetic(item)

    if "provenance_fields_ratio" in injected_failures:
        provenance = evidence_items[0].get("provenance", {})
        if isinstance(provenance, dict):
            provenance.pop("collector", None)

    if "unsigned_artifact_count" in injected_failures:
        signature = evidence_items[0].get("signature", {})
        if isinstance(signature, dict):
            signature["valid"] = False

    if "trace_linkage_ratio" in injected_failures or "detached_trace_count" in injected_failures:
        evidence_items[0]["trace_id"] = "trace-detached"
        evidence_items[0]["trace_digest"] = runtime_capture.stable_digest({"trace_id": "trace-detached"})

    if "qemu_trace_presence_ratio" in injected_failures:
        traces[:] = [trace for trace in traces if trace.get("execution_lane") != "qemu"]

    if "panic_trace_presence_ratio" in injected_failures:
        traces[:] = [trace for trace in traces if trace.get("execution_lane") != "panic"]

    if "default_image_binding_ratio" in injected_failures:
        provenance = evidence_items[0].get("provenance", {})
        if isinstance(provenance, dict):
            provenance["release_image_digest"] = "drifted-image-digest"
        if evidence_items[0].get("execution_lane") == "qemu":
            evidence_items[0]["artifact_digest"] = runtime_capture.stable_digest({"drift": True})

    if "boot_instance_binding_ratio" in injected_failures:
        provenance = evidence_items[0].get("provenance", {})
        if isinstance(provenance, dict):
            provenance["boot_id"] = ""


def _derive_metrics(
    *,
    traces: Sequence[Dict[str, object]],
    evidence_items: Sequence[Dict[str, object]],
    release_image_digest: str,
) -> Dict[str, float]:
    total = len(evidence_items)
    trace_map = {
        str(trace.get("trace_id", "")): trace
        for trace in traces
        if isinstance(trace, dict) and trace.get("trace_id")
    }

    linked_count = 0
    runtime_count = 0
    synthetic_count = 0
    provenance_complete = 0
    unsigned_count = 0
    default_image_bound = 0
    boot_bound = 0

    required_provenance = _required_provenance_fields()

    for item in evidence_items:
        if not isinstance(item, dict):
            continue

        synthetic = bool(item.get("synthetic"))
        runtime_source = item.get("runtime_source", {})
        runtime_kind = (
            runtime_source.get("kind", "")
            if isinstance(runtime_source, dict)
            else ""
        )
        if synthetic or runtime_kind == "synthetic_model":
            synthetic_count += 1
        else:
            runtime_count += 1

        signature = item.get("signature", {})
        if not (isinstance(signature, dict) and signature.get("valid") is True):
            unsigned_count += 1

        provenance = item.get("provenance", {})
        if isinstance(provenance, dict):
            if all(provenance.get(field) not in (None, "") for field in required_provenance):
                provenance_complete += 1
            if provenance.get("release_image_digest") == release_image_digest:
                default_image_bound += 1
            if provenance.get("boot_id") not in (None, ""):
                boot_bound += 1

        trace_id = str(item.get("trace_id", ""))
        trace_digest = str(item.get("trace_digest", ""))
        linked_trace = trace_map.get(trace_id)
        if not isinstance(linked_trace, dict):
            continue
        if trace_digest != str(linked_trace.get("trace_digest", "")):
            continue
        if item.get("execution_lane") != linked_trace.get("execution_lane"):
            continue
        linked_count += 1

    qemu_trace_present = any(
        trace.get("execution_lane") == "qemu" for trace in traces if isinstance(trace, dict)
    )
    panic_trace_present = any(
        trace.get("execution_lane") == "panic" for trace in traces if isinstance(trace, dict)
    )

    if total == 0:
        synthetic_ratio = 1.0
        runtime_ratio = 0.0
        linkage_ratio = 0.0
        provenance_ratio = 0.0
        default_image_ratio = 0.0
        boot_binding_ratio = 0.0
    else:
        synthetic_ratio = synthetic_count / total
        runtime_ratio = runtime_count / total
        linkage_ratio = linked_count / total
        provenance_ratio = provenance_complete / total
        default_image_ratio = default_image_bound / total
        boot_binding_ratio = boot_bound / total

    synthetic_only_flag = 1.0 if total > 0 and synthetic_count == total else 0.0
    detached_trace_count = float(total - linked_count)

    return {
        "runtime_item_count": float(total),
        "runtime_capture_ratio": round(runtime_ratio, 3),
        "qemu_trace_presence_ratio": 1.0 if qemu_trace_present else 0.0,
        "panic_trace_presence_ratio": 1.0 if panic_trace_present else 0.0,
        "trace_linkage_ratio": round(linkage_ratio, 3),
        "default_image_binding_ratio": round(default_image_ratio, 3),
        "boot_instance_binding_ratio": round(boot_binding_ratio, 3),
        "provenance_fields_ratio": round(provenance_ratio, 3),
        "unsigned_artifact_count": float(unsigned_count),
        "synthetic_evidence_ratio": round(synthetic_ratio, 3),
        "synthetic_only_artifacts": synthetic_only_flag,
        "detached_trace_count": detached_trace_count,
    }


def _passes(operator: str, observed: float, threshold: float) -> bool:
    if operator == "max":
        return observed <= threshold
    if operator == "min":
        return observed >= threshold
    if operator == "eq":
        return observed == threshold
    raise ValueError(f"unsupported operator: {operator}")


def _domain_summary(checks: List[Dict[str, object]], domain: str) -> Dict[str, object]:
    scoped = [entry for entry in checks if entry["domain"] == domain]
    failures = [entry for entry in scoped if entry["pass"] is False]
    return {
        "checks": len(scoped),
        "failures": len(failures),
        "pass": len(failures) == 0,
    }


def collect_runtime_evidence(
    *,
    runtime_capture_payload: Dict[str, object],
    runtime_capture_path: str,
    trace_bundle_payload: Dict[str, object],
    trace_bundle_path: str,
    diagnostic_snapshot_payload: Dict[str, object],
    diagnostic_snapshot_path: str,
    crash_dump_payload: Dict[str, object],
    crash_dump_path: str,
    crash_symbolized_payload: Dict[str, object],
    crash_symbolized_path: str,
    perf_baseline_payload: Dict[str, object],
    perf_baseline_path: str,
    perf_regression_payload: Dict[str, object],
    perf_regression_path: str,
    injected_failures: Set[str] | None = None,
) -> Dict[str, object]:
    failures = set() if injected_failures is None else set(injected_failures)
    traces = _trace_rows(
        runtime_capture_payload,
        crash_dump_payload,
        runtime_capture_path,
        crash_dump_path,
    )

    qemu_trace_id = str(runtime_capture_payload["trace_id"])
    qemu_trace_digest = str(runtime_capture_payload["trace_digest"])
    qemu_boot_id = str(runtime_capture_payload.get("capture_id", ""))
    qemu_provenance = runtime_capture.shared_provenance(
        collector="tools/collect_runtime_evidence_v1.py",
        command="python tools/collect_runtime_evidence_v1.py --runtime-capture out/booted-runtime-v1.json",
        release_image_path=str(runtime_capture_payload["image_path"]),
        release_image_digest=str(runtime_capture_payload["image_digest"]),
        kernel_path=str(runtime_capture_payload["kernel_path"]),
        kernel_digest=str(runtime_capture_payload["kernel_digest"]),
        build_id=str(runtime_capture_payload["build_id"]),
        capture_mode=str(runtime_capture_payload["capture_mode"]),
        runtime_capture_path=runtime_capture_path,
        runtime_capture_digest=str(runtime_capture_payload["digest"]),
        boot_id=qemu_boot_id,
        trace_id=qemu_trace_id,
    )

    crash_provenance = crash_dump_payload.get("runtime_provenance", {})
    if not isinstance(crash_provenance, dict):
        crash_provenance = {}
    panic_trace_id = str(crash_provenance.get("panic_trace_id", ""))
    panic_trace_digest = str(crash_provenance.get("panic_trace_digest", ""))
    panic_base_provenance = runtime_capture.shared_provenance(
        collector="tools/collect_runtime_evidence_v1.py",
        command="python tools/collect_runtime_evidence_v1.py --crash-dump out/crash-dump-v1.json",
        release_image_path=str(crash_provenance.get("release_image_path", "")),
        release_image_digest=str(crash_provenance.get("release_image_digest", "")),
        kernel_path=str(runtime_capture_payload["kernel_path"]),
        kernel_digest=str(runtime_capture_payload["kernel_digest"]),
        build_id=str(runtime_capture_payload["build_id"]),
        capture_mode=str(crash_provenance.get("capture_mode", "")),
        runtime_capture_path=crash_dump_path,
        runtime_capture_digest=str(crash_dump_payload.get("digest", "")),
        boot_id=str(crash_provenance.get("panic_boot_id", "")),
        trace_id=panic_trace_id,
    )

    evidence_items = [
        _make_item(
            artifact_id="runtime.capture.v1",
            execution_lane="qemu",
            trace_id=qemu_trace_id,
            trace_digest=qemu_trace_digest,
            provenance={**qemu_provenance, "artifact_path": runtime_capture_path},
            payload=runtime_capture_payload,
            runtime_kind="booted_runtime_capture",
        ),
        _make_item(
            artifact_id="perf.baseline.v1",
            execution_lane="qemu",
            trace_id=qemu_trace_id,
            trace_digest=qemu_trace_digest,
            provenance={**qemu_provenance, "artifact_path": perf_baseline_path},
            payload=perf_baseline_payload,
            runtime_kind="booted_runtime_capture",
        ),
        _make_item(
            artifact_id="perf.regression.v1",
            execution_lane="qemu",
            trace_id=qemu_trace_id,
            trace_digest=qemu_trace_digest,
            provenance={**qemu_provenance, "artifact_path": perf_regression_path},
            payload=perf_regression_payload,
            runtime_kind="booted_runtime_capture",
        ),
        _make_item(
            artifact_id="trace.bundle.v2",
            execution_lane="qemu",
            trace_id=qemu_trace_id,
            trace_digest=qemu_trace_digest,
            provenance={**qemu_provenance, "artifact_path": trace_bundle_path},
            payload=trace_bundle_payload,
            runtime_kind="booted_runtime_capture",
        ),
        _make_item(
            artifact_id="diagnostic.snapshot.v2",
            execution_lane="qemu",
            trace_id=qemu_trace_id,
            trace_digest=qemu_trace_digest,
            provenance={**qemu_provenance, "artifact_path": diagnostic_snapshot_path},
            payload=diagnostic_snapshot_payload,
            runtime_kind="booted_runtime_capture",
        ),
        _make_item(
            artifact_id="crash.dump.v1",
            execution_lane="panic",
            trace_id=panic_trace_id,
            trace_digest=panic_trace_digest,
            provenance={**panic_base_provenance, "artifact_path": crash_dump_path},
            payload=crash_dump_payload,
            runtime_kind="booted_panic_capture",
        ),
        _make_item(
            artifact_id="crash.dump.symbolized.v1",
            execution_lane="panic",
            trace_id=panic_trace_id,
            trace_digest=panic_trace_digest,
            provenance={**panic_base_provenance, "artifact_path": crash_symbolized_path},
            payload=crash_symbolized_payload,
            runtime_kind="booted_panic_capture",
        ),
    ]

    _apply_injected_failures(
        injected_failures=failures,
        traces=traces,
        evidence_items=evidence_items,
        release_image_digest=str(runtime_capture_payload["image_digest"]),
    )

    metrics = _derive_metrics(
        traces=traces,
        evidence_items=evidence_items,
        release_image_digest=str(runtime_capture_payload["image_digest"]),
    )

    checks: List[Dict[str, object]] = []
    for spec in CHECKS:
        observed = round(metrics[spec.metric_key], 3)
        checks.append(
            {
                "check_id": spec.check_id,
                "domain": spec.domain,
                "metric_key": spec.metric_key,
                "operator": spec.operator,
                "threshold": spec.threshold,
                "observed": observed,
                "pass": _passes(spec.operator, observed, spec.threshold),
            }
        )

    summary = {
        "execution": _domain_summary(checks, "execution"),
        "provenance": _domain_summary(checks, "provenance"),
        "synthetic": _domain_summary(checks, "synthetic"),
    }
    total_failures = sum(1 for check in checks if check["pass"] is False)

    stable_payload = {
        "schema": SCHEMA,
        "checks": [
            {"check_id": check["check_id"], "pass": check["pass"], "observed": check["observed"]}
            for check in checks
        ],
        "traces": [
            {
                "trace_id": trace["trace_id"],
                "execution_lane": trace["execution_lane"],
                "trace_digest": trace["trace_digest"],
            }
            for trace in traces
        ],
        "evidence_items": [
            {
                "artifact_id": item["artifact_id"],
                "execution_lane": item["execution_lane"],
                "synthetic": item["synthetic"],
                "trace_id": item["trace_id"],
                "trace_digest": item["trace_digest"],
                "signature_valid": item.get("signature", {}).get("valid") is True,
            }
            for item in evidence_items
        ],
        "injected_failures": sorted(failures),
    }
    digest = hashlib.sha256(
        json.dumps(stable_payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
    ).hexdigest()

    return {
        "schema": SCHEMA,
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "evidence_integrity_policy_id": EVIDENCE_INTEGRITY_POLICY_ID,
        "runtime_evidence_schema_id": RUNTIME_EVIDENCE_SCHEMA_ID,
        "gate_provenance_policy_id": GATE_PROVENANCE_POLICY_ID,
        "gate": "test-evidence-integrity-v1",
        "release_image_path": runtime_capture_payload.get("image_path", ""),
        "release_image_digest": runtime_capture_payload.get("image_digest", ""),
        "traces": traces,
        "evidence_items": evidence_items,
        "checks": checks,
        "summary": summary,
        "totals": {
            "evidence_items": len(evidence_items),
            "runtime_items": sum(1 for item in evidence_items if item.get("synthetic") is False),
            "synthetic_items": sum(1 for item in evidence_items if item.get("synthetic") is True),
            "linked_trace_items": len(evidence_items) - int(metrics["detached_trace_count"]),
            "detached_trace_count": int(metrics["detached_trace_count"]),
        },
        "artifact_refs": {
            "runtime_capture": runtime_capture_path,
            "perf_baseline": perf_baseline_path,
            "perf_regression": perf_regression_path,
            "trace_bundle": trace_bundle_path,
            "diagnostic_snapshot": diagnostic_snapshot_path,
            "crash_dump": crash_dump_path,
            "crash_dump_symbolized": crash_symbolized_path,
            "runtime_evidence_report": "out/runtime-evidence-v1.json",
            "gate_evidence_audit_report": "out/gate-evidence-audit-v1.json",
            "junit": "out/pytest-evidence-integrity-v1.xml",
            "synthetic_ban_junit": "out/pytest-synthetic-evidence-ban-v1.xml",
            "ci_artifact": "evidence-integrity-v1-artifacts",
            "synthetic_ban_ci_artifact": "synthetic-evidence-ban-v1-artifacts",
        },
        "injected_failures": sorted(failures),
        "total_failures": total_failures,
        "failures": sorted(check["check_id"] for check in checks if check["pass"] is False),
        "digest": digest,
    }


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--runtime-capture", required=True)
    parser.add_argument("--trace-bundle", required=True)
    parser.add_argument("--diagnostic-snapshot", required=True)
    parser.add_argument("--crash-dump", required=True)
    parser.add_argument("--crash-symbolized", required=True)
    parser.add_argument("--perf-baseline", required=True)
    parser.add_argument("--perf-regression", required=True)
    parser.add_argument(
        "--inject-failure",
        action="append",
        default=[],
        help="force a check to fail by check_id",
    )
    parser.add_argument("--max-failures", type=int, default=0)
    parser.add_argument("--out", default="out/runtime-evidence-v1.json")
    return parser


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)
    if args.max_failures < 0:
        print("error: max-failures must be >= 0")
        return 2

    try:
        injected_failures = _normalize_failures(args.inject_failure)
    except ValueError as exc:
        print(f"error: {exc}")
        return 2

    report = collect_runtime_evidence(
        runtime_capture_payload=_read_json(args.runtime_capture),
        runtime_capture_path=args.runtime_capture,
        trace_bundle_payload=_read_json(args.trace_bundle),
        trace_bundle_path=args.trace_bundle,
        diagnostic_snapshot_payload=_read_json(args.diagnostic_snapshot),
        diagnostic_snapshot_path=args.diagnostic_snapshot,
        crash_dump_payload=_read_json(args.crash_dump),
        crash_dump_path=args.crash_dump,
        crash_symbolized_payload=_read_json(args.crash_symbolized),
        crash_symbolized_path=args.crash_symbolized,
        perf_baseline_payload=_read_json(args.perf_baseline),
        perf_baseline_path=args.perf_baseline,
        perf_regression_payload=_read_json(args.perf_regression),
        perf_regression_path=args.perf_regression,
        injected_failures=injected_failures,
    )
    report["max_failures"] = args.max_failures
    report["gate_pass"] = report["total_failures"] <= args.max_failures

    out_path = Path(args.out)
    runtime_capture.write_json(out_path, report)

    print(f"runtime-evidence-report: {out_path}")
    print(f"total_failures: {report['total_failures']}")
    print(f"gate_pass: {report['gate_pass']}")
    return 0 if report["gate_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
