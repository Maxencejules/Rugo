#!/usr/bin/env python3
"""Run deterministic hardware claim promotion checks for M47."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
from typing import Dict, Iterable, List, Sequence, Set

import collect_hw_promotion_evidence_v2 as baremetal_promotion
import run_baremetal_io_baseline_v1 as baremetal
import run_hw_matrix_v6 as matrix


SCHEMA = "rugo.hw_claim_promotion_report.v1"
POLICY_ID = "rugo.hw_support_claim_policy.v1"
BAREMETAL_POLICY_ID = "rugo.hw_baremetal_promotion_policy.v2"
SUPPORT_TIER_AUDIT_ID = "rugo.hw_support_tier_audit.v1"
DEFAULT_SEED = 20260310
POLICY_EFFECTIVE_DATE = "2026-03-10"
SUPPORTED_TIERS = ("tier0", "tier1", "tier2", "tier3", "tier4")
REQUIRED_ARTIFACT_KEYS = (
    "matrix_report",
    "desktop_report",
    "baremetal_report",
    "baremetal_promotion_report",
    "recovery_report",
    "claim_promotion_report",
)
UNSUPPORTED_CLASS_REGISTRY = (
    "wifi",
    "bluetooth",
    "audio",
    "webcam",
    "discrete-gpu",
    "laptop-power-management",
)


@dataclass(frozen=True)
class ClaimSpec:
    class_id: str
    device: str
    support_tier: str
    claim_family: str
    source_milestone: str
    source_contract_id: str
    source_schema_id: str
    promotion_policy_id: str
    qualifying_profiles: tuple[str, ...]
    profile: str | None = None
    desktop_bound: bool = False
    recovery_bound: bool = False
    required_artifacts: tuple[str, ...] = ()


CLAIM_SPECS: tuple[ClaimSpec, ...] = (
    ClaimSpec(
        class_id="virtio-blk-pci-modern",
        device="virtio-blk-pci",
        support_tier="tier1",
        claim_family="virtual-platform",
        source_milestone="M45",
        source_contract_id="rugo.hw.support_matrix.v6",
        source_schema_id=matrix.SCHEMA,
        promotion_policy_id=POLICY_ID,
        qualifying_profiles=("q35", "pc/i440fx"),
        profile="modern",
        required_artifacts=("matrix_report", "desktop_report", "claim_promotion_report"),
    ),
    ClaimSpec(
        class_id="virtio-net-pci-modern",
        device="virtio-net-pci",
        support_tier="tier1",
        claim_family="virtual-platform",
        source_milestone="M45",
        source_contract_id="rugo.hw.support_matrix.v6",
        source_schema_id=matrix.SCHEMA,
        promotion_policy_id=POLICY_ID,
        qualifying_profiles=("q35", "pc/i440fx"),
        profile="modern",
        required_artifacts=("matrix_report", "desktop_report", "claim_promotion_report"),
    ),
    ClaimSpec(
        class_id="virtio-scsi-pci",
        device="virtio-scsi-pci",
        support_tier="tier1",
        claim_family="virtual-platform",
        source_milestone="M45",
        source_contract_id="rugo.hw.support_matrix.v6",
        source_schema_id=matrix.SCHEMA,
        promotion_policy_id=POLICY_ID,
        qualifying_profiles=("q35", "pc/i440fx"),
        profile="modern",
        required_artifacts=("matrix_report", "desktop_report", "claim_promotion_report"),
    ),
    ClaimSpec(
        class_id="virtio-gpu-pci",
        device="virtio-gpu-pci",
        support_tier="tier1",
        claim_family="virtual-platform",
        source_milestone="M45",
        source_contract_id="rugo.hw.support_matrix.v6",
        source_schema_id=matrix.SCHEMA,
        promotion_policy_id=POLICY_ID,
        qualifying_profiles=("q35", "pc/i440fx"),
        profile="modern",
        desktop_bound=True,
        required_artifacts=("matrix_report", "desktop_report", "claim_promotion_report"),
    ),
    ClaimSpec(
        class_id="e1000e",
        device="e1000e",
        support_tier="tier2",
        claim_family="baremetal-io",
        source_milestone="M46",
        source_contract_id="rugo.baremetal_io_profile.v1",
        source_schema_id=baremetal.SCHEMA,
        promotion_policy_id=BAREMETAL_POLICY_ID,
        qualifying_profiles=("intel_q470_e1000e_xhci",),
        required_artifacts=(
            "baremetal_report",
            "baremetal_promotion_report",
            "claim_promotion_report",
        ),
    ),
    ClaimSpec(
        class_id="rtl8169",
        device="rtl8169",
        support_tier="tier2",
        claim_family="baremetal-io",
        source_milestone="M46",
        source_contract_id="rugo.baremetal_io_profile.v1",
        source_schema_id=baremetal.SCHEMA,
        promotion_policy_id=BAREMETAL_POLICY_ID,
        qualifying_profiles=("amd_b550_rtl8169_xhci",),
        required_artifacts=(
            "baremetal_report",
            "baremetal_promotion_report",
            "claim_promotion_report",
        ),
    ),
    ClaimSpec(
        class_id="xhci",
        device="xhci",
        support_tier="tier2",
        claim_family="baremetal-io",
        source_milestone="M46",
        source_contract_id="rugo.baremetal_io_profile.v1",
        source_schema_id=baremetal.SCHEMA,
        promotion_policy_id=BAREMETAL_POLICY_ID,
        qualifying_profiles=("intel_q470_e1000e_xhci", "amd_b550_rtl8169_xhci"),
        desktop_bound=True,
        required_artifacts=(
            "baremetal_report",
            "baremetal_promotion_report",
            "desktop_report",
            "claim_promotion_report",
        ),
    ),
    ClaimSpec(
        class_id="usb-hid",
        device="usb-hid",
        support_tier="tier2",
        claim_family="baremetal-io",
        source_milestone="M46",
        source_contract_id="rugo.baremetal_io_profile.v1",
        source_schema_id=baremetal.SCHEMA,
        promotion_policy_id=BAREMETAL_POLICY_ID,
        qualifying_profiles=("intel_q470_e1000e_xhci", "amd_b550_rtl8169_xhci"),
        desktop_bound=True,
        required_artifacts=(
            "baremetal_report",
            "baremetal_promotion_report",
            "desktop_report",
            "claim_promotion_report",
        ),
    ),
    ClaimSpec(
        class_id="usb-storage",
        device="usb-storage",
        support_tier="tier2",
        claim_family="baremetal-io",
        source_milestone="M46",
        source_contract_id="rugo.baremetal_io_profile.v1",
        source_schema_id=baremetal.SCHEMA,
        promotion_policy_id=BAREMETAL_POLICY_ID,
        qualifying_profiles=("intel_q470_e1000e_xhci", "amd_b550_rtl8169_xhci"),
        recovery_bound=True,
        required_artifacts=(
            "baremetal_report",
            "baremetal_promotion_report",
            "recovery_report",
            "claim_promotion_report",
        ),
    ),
)


def expected_claim_tiers() -> Dict[str, str]:
    return {spec.class_id: spec.support_tier for spec in CLAIM_SPECS}


def claim_specs_by_id() -> Dict[str, ClaimSpec]:
    return {spec.class_id: spec for spec in CLAIM_SPECS}


def build_support_tier_summary(claims: Sequence[Dict[str, object]]) -> Dict[str, Dict[str, object]]:
    summary = {
        tier: {"promoted_claims": 0, "evidence_only_claims": 0, "classes": []}
        for tier in SUPPORTED_TIERS
    }
    for claim in claims:
        bucket = summary[str(claim["support_tier"])]
        if claim["claim_status"] == "promoted":
            bucket["promoted_claims"] += 1
        else:
            bucket["evidence_only_claims"] += 1
        bucket["classes"].append(str(claim["class_id"]))
    for bucket in summary.values():
        bucket["classes"].sort()
    return summary


def _normalize_strings(values: Sequence[str]) -> Set[str]:
    return {value.strip() for value in values if value.strip()}


def _normalize_failures(
    values: Sequence[str],
    known: Iterable[str],
    label: str,
) -> Set[str]:
    normalized = _normalize_strings(values)
    unknown = sorted(normalized - set(known))
    if unknown:
        raise ValueError(f"unknown {label}: {', '.join(unknown)}")
    return normalized


def _validate_missing_artifacts(missing: Set[str]) -> None:
    unknown = sorted(missing - set(REQUIRED_ARTIFACT_KEYS))
    if unknown:
        raise ValueError(
            "unknown artifacts in --inject-missing-artifact: " + ", ".join(unknown)
        )


def _find_device_status(
    coverage: Sequence[Dict[str, object]],
    device: str,
    profile: str | None = None,
) -> bool:
    for row in coverage:
        if row["device"] != device:
            continue
        if profile is not None and row.get("profile") != profile:
            continue
        return row["status"] == "pass"
    raise ValueError(f"device coverage missing for {device!r}")


def _artifacts_complete(required: Sequence[str], missing: Set[str]) -> bool:
    return all(key not in missing for key in required)


def run_claim_promotion(
    seed: int,
    matrix_failures: Set[str] | None = None,
    baremetal_failures: Set[str] | None = None,
    missing_artifacts: Set[str] | None = None,
) -> Dict[str, object]:
    matrix_injected = set() if matrix_failures is None else set(matrix_failures)
    baremetal_injected = set() if baremetal_failures is None else set(baremetal_failures)
    missing = set() if missing_artifacts is None else set(missing_artifacts)

    matrix_report = matrix.run_matrix(seed=seed, injected_failures=matrix_injected, max_failures=0)
    baremetal_report = baremetal.run_baseline(
        seed=seed,
        injected_failures=baremetal_injected,
        max_failures=0,
    )
    baremetal_promotion_report = baremetal_promotion.run_promotion(
        seed=seed,
        campaign_runs=12,
        required_consecutive_green=12,
        min_pass_rate=0.98,
    )

    matrix_bundle_green = (
        matrix_report["gate_pass"]
        and matrix_report["virtio_profile_matrix"]["modern"]["checks_pass"]
        and matrix_report["desktop_display_checks"]["bridge_pass"]
    )
    baremetal_bundle_green = (
        baremetal_report["gate_pass"]
        and baremetal_promotion_report["gate_pass"]
        and baremetal_promotion_report["tier2_floor_met"]
        and baremetal_report["desktop_input_checks"]["input_checks_pass"]
        and baremetal_report["install_recovery_checks"]["recovery_gate_pass"]
    )

    claims: List[Dict[str, object]] = []
    for spec in CLAIM_SPECS:
        if spec.claim_family == "virtual-platform":
            device_pass = _find_device_status(
                matrix_report["device_class_coverage"],
                spec.device,
                spec.profile,
            )
            evidence_ready = (
                matrix_bundle_green
                and device_pass
                and _artifacts_complete(spec.required_artifacts, missing)
            )
            source_digest = matrix_report["digest"]
            desktop_green = bool(matrix_report["desktop_display_checks"]["bridge_pass"])
            recovery_green = True
            promoted_source_schema = matrix_report["schema"]
            promoted_source_digest = matrix_report["digest"]
            initial_policy_id = matrix_report["matrix_contract_id"]
        else:
            device_pass = _find_device_status(
                baremetal_report["device_class_coverage"],
                spec.device,
            )
            evidence_ready = (
                baremetal_bundle_green
                and device_pass
                and _artifacts_complete(spec.required_artifacts, missing)
            )
            source_digest = baremetal_report["digest"]
            desktop_green = bool(baremetal_report["desktop_input_checks"]["input_checks_pass"])
            recovery_green = bool(
                baremetal_report["install_recovery_checks"]["recovery_gate_pass"]
            )
            promoted_source_schema = baremetal_promotion_report["schema"]
            promoted_source_digest = baremetal_promotion_report["digest"]
            initial_policy_id = baremetal_report["profile_id"]

        claim_status = "promoted" if evidence_ready else "evidence_only"
        claims.append(
            {
                "class_id": spec.class_id,
                "device": spec.device,
                "support_tier": spec.support_tier,
                "claim_family": spec.claim_family,
                "claim_status": claim_status,
                "policy_id": POLICY_ID,
                "promotion_policy_id": spec.promotion_policy_id,
                "source_milestone": spec.source_milestone,
                "source_contract_id": spec.source_contract_id,
                "source_schema_id": spec.source_schema_id,
                "source_digest": source_digest,
                "qualifying_profiles": list(spec.qualifying_profiles),
                "desktop_bound": spec.desktop_bound,
                "recovery_bound": spec.recovery_bound,
                "desktop_bridge_green": desktop_green,
                "recovery_bridge_green": recovery_green,
                "required_artifacts": list(spec.required_artifacts),
                "evidence_ready": evidence_ready,
                "promotion_history": [
                    {
                        "milestone": spec.source_milestone,
                        "status": "evidence_only",
                        "policy_id": initial_policy_id,
                        "effective_date": POLICY_EFFECTIVE_DATE,
                        "source_schema_id": spec.source_schema_id,
                        "source_digest": source_digest,
                    },
                    {
                        "milestone": "M47",
                        "status": (
                            "promoted" if claim_status == "promoted" else "retained_evidence_only"
                        ),
                        "policy_id": spec.promotion_policy_id,
                        "effective_date": POLICY_EFFECTIVE_DATE,
                        "source_schema_id": promoted_source_schema,
                        "source_digest": promoted_source_digest,
                    },
                ],
            }
        )

    support_tier_summary = build_support_tier_summary(claims)
    promoted_virtual = [
        claim
        for claim in claims
        if claim["claim_family"] == "virtual-platform" and claim["claim_status"] == "promoted"
    ]
    promoted_baremetal = [
        claim
        for claim in claims
        if claim["claim_family"] == "baremetal-io" and claim["claim_status"] == "promoted"
    ]
    claims_have_policy_ids = all(
        claim["policy_id"] and claim["promotion_policy_id"] for claim in claims
    )
    tier_summary_consistent = (
        sum(bucket["promoted_claims"] for bucket in support_tier_summary.values())
        == len(promoted_virtual) + len(promoted_baremetal)
    )

    policy_checks = [
        {
            "check_id": "artifact_bundle_complete",
            "operator": "eq",
            "threshold": True,
            "observed": len(missing) == 0,
            "pass": len(missing) == 0,
        },
        {
            "check_id": "matrix_claim_bundle_green",
            "operator": "eq",
            "threshold": True,
            "observed": matrix_bundle_green,
            "pass": matrix_bundle_green,
        },
        {
            "check_id": "matrix_targets_promoted",
            "operator": "eq",
            "threshold": 4,
            "observed": len(promoted_virtual),
            "pass": len(promoted_virtual) == 4,
        },
        {
            "check_id": "baremetal_claim_bundle_green",
            "operator": "eq",
            "threshold": True,
            "observed": baremetal_bundle_green,
            "pass": baremetal_bundle_green,
        },
        {
            "check_id": "baremetal_targets_promoted",
            "operator": "eq",
            "threshold": 5,
            "observed": len(promoted_baremetal),
            "pass": len(promoted_baremetal) == 5,
        },
        {
            "check_id": "claims_have_policy_ids",
            "operator": "eq",
            "threshold": True,
            "observed": claims_have_policy_ids,
            "pass": claims_have_policy_ids,
        },
        {
            "check_id": "unsupported_registry_explicit",
            "operator": "min",
            "threshold": 1,
            "observed": len(UNSUPPORTED_CLASS_REGISTRY),
            "pass": len(UNSUPPORTED_CLASS_REGISTRY) >= 1,
        },
        {
            "check_id": "support_tier_summary_consistent",
            "operator": "eq",
            "threshold": True,
            "observed": tier_summary_consistent,
            "pass": tier_summary_consistent,
        },
    ]

    failures = sorted(
        check["check_id"] for check in policy_checks if check["pass"] is False
    )
    total_failures = len(failures)
    gate_pass = total_failures == 0

    stable_payload = {
        "schema": SCHEMA,
        "seed": seed,
        "matrix_digest": matrix_report["digest"],
        "baremetal_digest": baremetal_report["digest"],
        "baremetal_promotion_digest": baremetal_promotion_report["digest"],
        "claims": [
            {
                "class_id": claim["class_id"],
                "support_tier": claim["support_tier"],
                "claim_status": claim["claim_status"],
                "promotion_policy_id": claim["promotion_policy_id"],
            }
            for claim in claims
        ],
        "missing_artifacts": sorted(missing),
        "checks": [
            {"check_id": check["check_id"], "pass": check["pass"]} for check in policy_checks
        ],
    }
    digest = hashlib.sha256(
        json.dumps(stable_payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
    ).hexdigest()

    available_artifacts = sorted(
        key for key in REQUIRED_ARTIFACT_KEYS if key not in missing
    )

    return {
        "schema": SCHEMA,
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "policy_id": POLICY_ID,
        "baremetal_promotion_policy_id": BAREMETAL_POLICY_ID,
        "support_tier_audit_id": SUPPORT_TIER_AUDIT_ID,
        "matrix_schema_id": matrix.SCHEMA,
        "baremetal_schema_id": baremetal.SCHEMA,
        "baremetal_promotion_schema_id": baremetal_promotion.SCHEMA,
        "seed": seed,
        "gate": "test-hw-claim-promotion-v1",
        "claims": claims,
        "support_tier_summary": support_tier_summary,
        "unsupported_class_registry": list(UNSUPPORTED_CLASS_REGISTRY),
        "artifact_refs": {
            "matrix_report": "out/hw-matrix-v6.json",
            "desktop_report": "out/desktop-smoke-v1.json",
            "baremetal_report": "out/baremetal-io-v1.json",
            "baremetal_promotion_report": "out/hw-promotion-v2.json",
            "recovery_report": "out/recovery-drill-v3.json",
            "claim_promotion_report": "out/hw-claim-promotion-v1.json",
            "support_tier_audit_report": "out/hw-support-tier-audit-v1.json",
            "ci_artifact": "hw-claim-promotion-v1-artifacts",
            "audit_ci_artifact": "hw-support-tier-audit-v1-artifacts",
        },
        "available_artifacts": available_artifacts,
        "missing_artifacts": sorted(missing),
        "policy_checks": policy_checks,
        "source_reports": {
            "matrix": {
                "schema": matrix_report["schema"],
                "digest": matrix_report["digest"],
                "gate_pass": matrix_report["gate_pass"],
            },
            "baremetal": {
                "schema": baremetal_report["schema"],
                "digest": baremetal_report["digest"],
                "gate_pass": baremetal_report["gate_pass"],
            },
            "baremetal_promotion": {
                "schema": baremetal_promotion_report["schema"],
                "digest": baremetal_promotion_report["digest"],
                "gate_pass": baremetal_promotion_report["gate_pass"],
            },
            "desktop_display": {
                "schema": matrix_report["desktop_display_checks"]["source_schema"],
                "digest": matrix_report["desktop_display_checks"]["source_digest"],
                "bridge_pass": matrix_report["desktop_display_checks"]["bridge_pass"],
            },
            "desktop_input": {
                "schema": baremetal_report["desktop_input_checks"]["source_schema"],
                "digest": baremetal_report["desktop_input_checks"]["source_digest"],
                "bridge_pass": baremetal_report["desktop_input_checks"]["input_checks_pass"],
            },
            "recovery": {
                "schema": baremetal_report["install_recovery_checks"]["source_schema"],
                "workflow_id": baremetal_report["install_recovery_checks"]["workflow_id"],
                "gate_pass": baremetal_report["install_recovery_checks"]["recovery_gate_pass"],
            },
        },
        "injected_matrix_failures": sorted(matrix_injected),
        "injected_baremetal_failures": sorted(baremetal_injected),
        "total_failures": total_failures,
        "failures": failures,
        "gate_pass": gate_pass,
        "digest": digest,
    }


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--seed", type=int, default=DEFAULT_SEED)
    parser.add_argument(
        "--inject-matrix-failure",
        action="append",
        default=[],
        help="force an M45 matrix check to fail by check_id",
    )
    parser.add_argument(
        "--inject-baremetal-failure",
        action="append",
        default=[],
        help="force an M46 bare-metal check to fail by check_id",
    )
    parser.add_argument(
        "--inject-missing-artifact",
        action="append",
        default=[],
        help="remove required artifact key from the claim bundle",
    )
    parser.add_argument("--out", default="out/hw-claim-promotion-v1.json")
    return parser


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)
    try:
        matrix_failures = _normalize_failures(
            args.inject_matrix_failure,
            matrix.known_checks(),
            "matrix failure check ids in --inject-matrix-failure",
        )
        baremetal_failures = _normalize_failures(
            args.inject_baremetal_failure,
            baremetal.known_checks(),
            "bare-metal failure check ids in --inject-baremetal-failure",
        )
        missing = _normalize_strings(args.inject_missing_artifact)
        _validate_missing_artifacts(missing)
    except ValueError as exc:
        print(f"error: {exc}")
        return 2

    report = run_claim_promotion(
        seed=args.seed,
        matrix_failures=matrix_failures,
        baremetal_failures=baremetal_failures,
        missing_artifacts=missing,
    )

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    print(f"hw-claim-promotion-report: {out_path}")
    print(f"total_failures: {report['total_failures']}")
    print(f"gate_pass: {report['gate_pass']}")
    return 0 if report["gate_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
