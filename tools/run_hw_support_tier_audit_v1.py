#!/usr/bin/env python3
"""Run deterministic support-tier audits for M47 claimable hardware classes."""

from __future__ import annotations

import argparse
import copy
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
from typing import Dict, List, Sequence, Set, Tuple

import run_hw_claim_promotion_v1 as claim_promotion


SCHEMA = "rugo.hw_support_tier_audit_report.v1"
AUDIT_ID = "rugo.hw_support_tier_audit.v1"
DEFAULT_SEED = 20260310


def _normalize_strings(values: Sequence[str]) -> Set[str]:
    return {value.strip() for value in values if value.strip()}


def _parse_tier_drift(value: str) -> Tuple[str, str]:
    if "=" not in value:
        raise ValueError(
            "tier drift override must be '<class_id>=<support_tier>'"
        )
    class_id, tier = value.split("=", 1)
    class_id = class_id.strip()
    tier = tier.strip()
    if class_id not in claim_promotion.claim_specs_by_id():
        raise ValueError(f"unknown class id in --inject-tier-drift: {class_id}")
    if tier not in claim_promotion.SUPPORTED_TIERS:
        raise ValueError(f"unknown support tier in --inject-tier-drift: {tier}")
    return class_id, tier


def _find_claim(claims: Sequence[Dict[str, object]], class_id: str) -> Dict[str, object]:
    for claim in claims:
        if claim["class_id"] == class_id:
            return claim
    raise ValueError(f"claim record missing for class_id={class_id}")


def run_audit(
    seed: int,
    tier_drifts: Dict[str, str] | None = None,
    unsupported_claims: Set[str] | None = None,
    dropped_history_claims: Set[str] | None = None,
) -> Dict[str, object]:
    tier_overrides = {} if tier_drifts is None else dict(tier_drifts)
    unsupported = set() if unsupported_claims is None else set(unsupported_claims)
    dropped_history = (
        set() if dropped_history_claims is None else set(dropped_history_claims)
    )

    claim_report = claim_promotion.run_claim_promotion(seed=seed)
    claims = copy.deepcopy(claim_report["claims"])

    for class_id, tier in sorted(tier_overrides.items()):
        _find_claim(claims, class_id)["support_tier"] = tier

    for class_id in sorted(dropped_history):
        _find_claim(claims, class_id)["promotion_history"] = []

    for class_id in sorted(unsupported):
        if class_id not in claim_promotion.UNSUPPORTED_CLASS_REGISTRY:
            raise ValueError(
                f"unsupported class not in registry for --inject-unsupported-claim: {class_id}"
            )
        claims.append(
            {
                "class_id": class_id,
                "device": class_id,
                "support_tier": "tier2",
                "claim_family": "unsupported",
                "claim_status": "promoted",
                "policy_id": claim_promotion.POLICY_ID,
                "promotion_policy_id": claim_promotion.POLICY_ID,
                "source_milestone": "M47",
                "source_contract_id": claim_promotion.POLICY_ID,
                "source_schema_id": claim_promotion.SCHEMA,
                "source_digest": claim_report["digest"],
                "qualifying_profiles": [],
                "desktop_bound": False,
                "recovery_bound": False,
                "desktop_bridge_green": True,
                "recovery_bridge_green": True,
                "required_artifacts": ["claim_promotion_report"],
                "evidence_ready": True,
                "promotion_history": [
                    {
                        "milestone": "M47",
                        "status": "promoted",
                        "policy_id": claim_promotion.POLICY_ID,
                        "effective_date": claim_promotion.POLICY_EFFECTIVE_DATE,
                        "source_schema_id": claim_promotion.SCHEMA,
                        "source_digest": claim_report["digest"],
                    }
                ],
            }
        )

    expected_tiers = claim_promotion.expected_claim_tiers()
    promoted_claims = [claim for claim in claims if claim["claim_status"] == "promoted"]
    drifted_claims = sorted(
        claim["class_id"]
        for claim in claims
        if claim["class_id"] in expected_tiers
        and claim["support_tier"] != expected_tiers[claim["class_id"]]
    )
    unsupported_promoted_claims = sorted(
        claim["class_id"]
        for claim in promoted_claims
        if claim["class_id"] in claim_promotion.UNSUPPORTED_CLASS_REGISTRY
    )
    missing_history_claims = sorted(
        claim["class_id"]
        for claim in promoted_claims
        if not any(
            entry.get("status") == "promoted" and entry.get("source_digest")
            for entry in claim["promotion_history"]
        )
    )
    missing_policy_id_claims = sorted(
        claim["class_id"]
        for claim in promoted_claims
        if not claim.get("policy_id") or not claim.get("promotion_policy_id")
    )
    desktop_bound_failures = sorted(
        claim["class_id"]
        for claim in promoted_claims
        if claim["desktop_bound"] and claim["desktop_bridge_green"] is not True
    )
    recovery_bound_failures = sorted(
        claim["class_id"]
        for claim in promoted_claims
        if claim["recovery_bound"] and claim["recovery_bridge_green"] is not True
    )

    observed_tier_summary = claim_promotion.build_support_tier_summary(claims)
    reported_tier_summary = claim_report["support_tier_summary"]

    checks = [
        {
            "check_id": "claim_report_green",
            "pass": bool(claim_report["gate_pass"]),
        },
        {
            "check_id": "promoted_claims_non_empty",
            "pass": len(promoted_claims) > 0,
        },
        {
            "check_id": "tier_assignments_match_policy",
            "pass": len(drifted_claims) == 0,
        },
        {
            "check_id": "promotion_history_traceable",
            "pass": len(missing_history_claims) == 0,
        },
        {
            "check_id": "policy_ids_present",
            "pass": len(missing_policy_id_claims) == 0,
        },
        {
            "check_id": "desktop_bound_claims_backed",
            "pass": len(desktop_bound_failures) == 0,
        },
        {
            "check_id": "recovery_bound_claims_backed",
            "pass": len(recovery_bound_failures) == 0,
        },
        {
            "check_id": "unsupported_registry_preserved",
            "pass": claim_report["unsupported_class_registry"]
            == list(claim_promotion.UNSUPPORTED_CLASS_REGISTRY),
        },
        {
            "check_id": "unsupported_classes_not_promoted",
            "pass": len(unsupported_promoted_claims) == 0,
        },
        {
            "check_id": "reported_tier_summary_matches_claims",
            "pass": reported_tier_summary == observed_tier_summary,
        },
    ]

    failures = sorted(check["check_id"] for check in checks if check["pass"] is False)
    total_failures = len(failures)
    gate_pass = total_failures == 0

    stable_payload = {
        "schema": SCHEMA,
        "seed": seed,
        "claim_report_digest": claim_report["digest"],
        "claims": [
            {
                "class_id": claim["class_id"],
                "support_tier": claim["support_tier"],
                "claim_status": claim["claim_status"],
            }
            for claim in claims
        ],
        "checks": [{"check_id": check["check_id"], "pass": check["pass"]} for check in checks],
        "unsupported_promoted_claims": unsupported_promoted_claims,
        "missing_history_claims": missing_history_claims,
    }
    digest = hashlib.sha256(
        json.dumps(stable_payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
    ).hexdigest()

    return {
        "schema": SCHEMA,
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "audit_id": AUDIT_ID,
        "claim_policy_id": claim_promotion.POLICY_ID,
        "baremetal_promotion_policy_id": claim_promotion.BAREMETAL_POLICY_ID,
        "claim_report_schema_id": claim_promotion.SCHEMA,
        "seed": seed,
        "gate": "test-hw-support-tier-audit-v1",
        "claim_report_digest": claim_report["digest"],
        "claim_report_gate_pass": claim_report["gate_pass"],
        "reported_tier_summary": reported_tier_summary,
        "observed_tier_summary": observed_tier_summary,
        "promoted_claims": [
            {
                "class_id": claim["class_id"],
                "support_tier": claim["support_tier"],
                "claim_family": claim["claim_family"],
                "policy_id": claim["policy_id"],
                "promotion_policy_id": claim["promotion_policy_id"],
            }
            for claim in promoted_claims
        ],
        "drifted_claims": drifted_claims,
        "unsupported_promoted_claims": unsupported_promoted_claims,
        "missing_history_claims": missing_history_claims,
        "missing_policy_id_claims": missing_policy_id_claims,
        "desktop_bound_failures": desktop_bound_failures,
        "recovery_bound_failures": recovery_bound_failures,
        "checks": checks,
        "injected_tier_drifts": {key: tier_overrides[key] for key in sorted(tier_overrides)},
        "injected_unsupported_claims": sorted(unsupported),
        "dropped_history_claims": sorted(dropped_history),
        "artifact_refs": {
            "claim_promotion_report": "out/hw-claim-promotion-v1.json",
            "support_tier_audit_report": "out/hw-support-tier-audit-v1.json",
            "ci_artifact": "hw-support-tier-audit-v1-artifacts",
        },
        "total_failures": total_failures,
        "failures": failures,
        "gate_pass": gate_pass,
        "digest": digest,
    }


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--seed", type=int, default=DEFAULT_SEED)
    parser.add_argument(
        "--inject-tier-drift",
        action="append",
        default=[],
        help="override a promoted class tier via '<class_id>=<support_tier>'",
    )
    parser.add_argument(
        "--inject-unsupported-claim",
        action="append",
        default=[],
        help="add an unsupported class as a promoted claim by class id",
    )
    parser.add_argument(
        "--drop-history",
        action="append",
        default=[],
        help="remove promotion history from a class id",
    )
    parser.add_argument("--out", default="out/hw-support-tier-audit-v1.json")
    return parser


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)
    try:
        tier_drifts = dict(_parse_tier_drift(value) for value in args.inject_tier_drift)
        unsupported_claims = _normalize_strings(args.inject_unsupported_claim)
        dropped_history = _normalize_strings(args.drop_history)
    except ValueError as exc:
        print(f"error: {exc}")
        return 2

    try:
        report = run_audit(
            seed=args.seed,
            tier_drifts=tier_drifts,
            unsupported_claims=unsupported_claims,
            dropped_history_claims=dropped_history,
        )
    except ValueError as exc:
        print(f"error: {exc}")
        return 2

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    print(f"hw-support-tier-audit-report: {out_path}")
    print(f"total_failures: {report['total_failures']}")
    print(f"gate_pass: {report['gate_pass']}")
    return 0 if report["gate_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
