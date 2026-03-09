#!/usr/bin/env python3
"""Generate deterministic M34 maturity qualification bundle and LTS decision."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
from typing import Any, Callable, Dict, List

import collect_crash_dump_v1 as crash_dump
import collect_measured_boot_report_v1 as measured_boot
import generate_provenance_v1 as provenance
import generate_sbom_v1 as sbom
import release_branch_audit_v2 as branch_audit
import run_canary_rollout_sim_v1 as canary_rollout
import run_conformance_suite_v1 as conformance
import run_fleet_health_sim_v1 as fleet_health
import run_fleet_update_sim_v1 as fleet_update
import run_rollout_abort_drill_v1 as rollout_abort
import security_advisory_lint_v1 as advisory_lint
import security_embargo_drill_v1 as embargo_drill
import support_window_audit_v1 as support_audit
import symbolize_crash_dump_v1 as crash_symbolize
import verify_release_attestations_v1 as attest_verify
import verify_sbom_provenance_v2 as supply_chain_verify


SCHEMA = "rugo.maturity_qualification_bundle.v1"
POLICY_ID = "rugo.maturity_qualification_policy.v1"
LTS_POLICY_ID = "rugo.lts_declaration_policy.v1"
LTS_SCHEMA = "rugo.lts_declaration_report.v1"
DEFAULT_SEED = 20260309


def _read_json(path: Path) -> Dict[str, Any]:
    if not path.is_file():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def _created_utc() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def _invoke(
    name: str,
    runner: Callable[[List[str] | None], int],
    argv: List[str],
    runs: List[Dict[str, Any]],
) -> int:
    rc = runner(argv)
    runs.append({"tool": name, "argv": argv, "rc": rc})
    return rc


def _check(name: str, passed: bool) -> Dict[str, Any]:
    return {"name": name, "pass": bool(passed)}


def _lts_summary(
    *,
    qualified_release_count: int,
    min_qualified_releases: int,
    lts_min_support_days: int,
    advisory: Dict[str, Any],
    embargo: Dict[str, Any],
    attestation: Dict[str, Any],
    support: Dict[str, Any],
) -> Dict[str, Any]:
    windows = support.get("windows", [])
    lts_window = None
    if isinstance(windows, list):
        for entry in windows:
            if isinstance(entry, dict) and entry.get("channel") == "lts":
                lts_window = entry
                break

    lts_support_days = 0
    if isinstance(lts_window, dict):
        lts_support_days = int(lts_window.get("support_days", 0))

    advisory_breaches = 0 if advisory.get("valid") is True else 1
    embargo_breaches = 0 if embargo.get("meets_sla") is True else 1
    drift_count = int(attestation.get("drift_count", 0))

    criteria = [
        _check(
            "minimum_qualified_releases",
            qualified_release_count >= min_qualified_releases,
        ),
        _check("lts_channel_present", isinstance(lts_window, dict)),
        _check(
            "minimum_support_window_days",
            lts_support_days >= lts_min_support_days,
        ),
        _check("advisory_lint_valid", advisory.get("valid") is True),
        _check("embargo_drill_meets_sla", embargo.get("meets_sla") is True),
        _check("attestation_drift_within_tolerance", drift_count == 0),
    ]

    eligible = all(criterion["pass"] for criterion in criteria)
    return {
        "schema": LTS_SCHEMA,
        "policy_id": LTS_POLICY_ID,
        "created_utc": _created_utc(),
        "qualified_release_count": qualified_release_count,
        "min_qualified_releases": min_qualified_releases,
        "lts_support_days": lts_support_days,
        "min_support_days": lts_min_support_days,
        "advisory_sla_breach_count": advisory_breaches + embargo_breaches,
        "supply_chain_drift_count": drift_count,
        "criteria": criteria,
        "eligible": eligible,
    }


def run_qualification(
    *,
    seed: int,
    artifact_dir: Path,
    qualified_release_count: int,
    min_qualified_releases: int,
    lts_min_support_days: int,
) -> Dict[str, Any]:
    artifact_dir.mkdir(parents=True, exist_ok=True)
    runs: List[Dict[str, Any]] = []

    paths = {
        "security_advisory": artifact_dir / "security-advisory-lint-v1.json",
        "security_embargo": artifact_dir / "security-embargo-drill-v1.json",
        "sbom": artifact_dir / "sbom-v1.spdx.json",
        "provenance": artifact_dir / "provenance-v1.json",
        "supply_chain": artifact_dir / "supply-chain-revalidation-v1.json",
        "attestation": artifact_dir / "release-attestation-verification-v1.json",
        "canary": artifact_dir / "canary-rollout-sim-v1.json",
        "rollout_abort": artifact_dir / "rollout-abort-drill-v1.json",
        "fleet_update": artifact_dir / "fleet-update-sim-v1.json",
        "fleet_health": artifact_dir / "fleet-health-sim-v1.json",
        "conformance": artifact_dir / "conformance-v1.json",
        "release_branch": artifact_dir / "release-branch-audit-v2.json",
        "support_window": artifact_dir / "support-window-audit-v1.json",
        "measured_boot": artifact_dir / "measured-boot-v1.json",
        "crash_dump": artifact_dir / "crash-dump-v1.json",
        "crash_symbolized": artifact_dir / "crash-dump-symbolized-v1.json",
    }

    _invoke(
        "security_advisory_lint_v1",
        advisory_lint.main,
        ["--out", str(paths["security_advisory"])],
        runs,
    )
    _invoke(
        "security_embargo_drill_v1",
        embargo_drill.main,
        ["--out", str(paths["security_embargo"])],
        runs,
    )
    _invoke("generate_sbom_v1", sbom.main, ["--out", str(paths["sbom"])], runs)
    _invoke(
        "generate_provenance_v1",
        provenance.main,
        ["--out", str(paths["provenance"])],
        runs,
    )
    _invoke(
        "verify_sbom_provenance_v2",
        supply_chain_verify.main,
        [
            "--sbom",
            str(paths["sbom"]),
            "--provenance",
            str(paths["provenance"]),
            "--out",
            str(paths["supply_chain"]),
        ],
        runs,
    )
    _invoke(
        "verify_release_attestations_v1",
        attest_verify.main,
        ["--out", str(paths["attestation"])],
        runs,
    )
    _invoke(
        "run_canary_rollout_sim_v1",
        canary_rollout.main,
        ["--seed", str(seed), "--out", str(paths["canary"])],
        runs,
    )
    _invoke(
        "run_rollout_abort_drill_v1",
        rollout_abort.main,
        ["--out", str(paths["rollout_abort"])],
        runs,
    )
    _invoke(
        "run_fleet_update_sim_v1",
        fleet_update.main,
        ["--seed", str(seed), "--out", str(paths["fleet_update"])],
        runs,
    )
    _invoke(
        "run_fleet_health_sim_v1",
        fleet_health.main,
        ["--seed", str(seed), "--out", str(paths["fleet_health"])],
        runs,
    )
    _invoke(
        "run_conformance_suite_v1",
        conformance.main,
        ["--seed", str(seed), "--out", str(paths["conformance"])],
        runs,
    )
    _invoke(
        "release_branch_audit_v2",
        branch_audit.main,
        ["--max-failures", "0", "--out", str(paths["release_branch"])],
        runs,
    )
    _invoke(
        "support_window_audit_v1",
        support_audit.main,
        ["--max-failures", "0", "--out", str(paths["support_window"])],
        runs,
    )
    _invoke(
        "collect_measured_boot_report_v1",
        measured_boot.main,
        ["--out", str(paths["measured_boot"])],
        runs,
    )
    _invoke(
        "collect_crash_dump_v1",
        crash_dump.main,
        ["--out", str(paths["crash_dump"])],
        runs,
    )
    _invoke(
        "symbolize_crash_dump_v1",
        crash_symbolize.main,
        [
            "--dump",
            str(paths["crash_dump"]),
            "--out",
            str(paths["crash_symbolized"]),
        ],
        runs,
    )

    advisory = _read_json(paths["security_advisory"])
    embargo = _read_json(paths["security_embargo"])
    supply_chain = _read_json(paths["supply_chain"])
    attestation = _read_json(paths["attestation"])
    canary = _read_json(paths["canary"])
    rollout_abort_report = _read_json(paths["rollout_abort"])
    fleet_update_report = _read_json(paths["fleet_update"])
    fleet_health_report = _read_json(paths["fleet_health"])
    conformance_report = _read_json(paths["conformance"])
    branch_report = _read_json(paths["release_branch"])
    support_report = _read_json(paths["support_window"])
    measured_report = _read_json(paths["measured_boot"])
    crash_symbolized_report = _read_json(paths["crash_symbolized"])

    checks = [
        _check("all_tools_exit_zero", all(run["rc"] == 0 for run in runs)),
        _check("security_advisory_valid", advisory.get("valid") is True),
        _check("security_embargo_meets_sla", embargo.get("meets_sla") is True),
        _check("supply_chain_revalidation_pass", supply_chain.get("total_failures", 1) == 0),
        _check("release_attestation_pass", attestation.get("meets_target") is True),
        _check("rollout_simulation_pass", canary.get("gate_pass") is True),
        _check(
            "rollout_abort_policy_enforced",
            rollout_abort_report.get("policy_enforced") is True
            and rollout_abort_report.get("meets_target") is True,
        ),
        _check("fleet_update_pass", fleet_update_report.get("gate_pass") is True),
        _check("fleet_health_pass", fleet_health_report.get("gate_pass") is True),
        _check("conformance_pass", conformance_report.get("gate_pass") is True),
        _check("release_branch_audit_pass", branch_report.get("meets_target") is True),
        _check("support_window_audit_pass", support_report.get("meets_target") is True),
        _check("measured_boot_policy_pass", measured_report.get("policy_pass") is True),
        _check("crash_dump_symbolization_pass", crash_symbolized_report.get("gate_pass") is True),
        _check(
            "qualified_release_window",
            qualified_release_count >= min_qualified_releases,
        ),
    ]

    lts_declaration = _lts_summary(
        qualified_release_count=qualified_release_count,
        min_qualified_releases=min_qualified_releases,
        lts_min_support_days=lts_min_support_days,
        advisory=advisory,
        embargo=embargo,
        attestation=attestation,
        support=support_report,
    )
    checks.append(_check("lts_declaration_eligible", lts_declaration["eligible"] is True))

    total_failures = sum(1 for check in checks if check["pass"] is False)

    evidence_summary = {
        "security": {
            "advisory_valid": advisory.get("valid"),
            "embargo_meets_sla": embargo.get("meets_sla"),
        },
        "supply_chain": {
            "total_failures": supply_chain.get("total_failures"),
            "attestation_meets_target": attestation.get("meets_target"),
        },
        "rollout": {
            "canary_gate_pass": canary.get("gate_pass"),
            "abort_policy_enforced": rollout_abort_report.get("policy_enforced"),
        },
        "fleet": {
            "update_gate_pass": fleet_update_report.get("gate_pass"),
            "health_gate_pass": fleet_health_report.get("gate_pass"),
        },
        "conformance": {
            "gate_pass": conformance_report.get("gate_pass"),
            "checked_profiles": conformance_report.get("checked_profiles", []),
        },
        "lifecycle": {
            "branch_meets_target": branch_report.get("meets_target"),
            "support_meets_target": support_report.get("meets_target"),
        },
        "reliability": {
            "measured_boot_pass": measured_report.get("policy_pass"),
            "crash_symbolization_pass": crash_symbolized_report.get("gate_pass"),
        },
    }

    stable_digest_payload = {
        "schema": SCHEMA,
        "policy_id": POLICY_ID,
        "seed": seed,
        "qualified_release_count": qualified_release_count,
        "min_qualified_releases": min_qualified_releases,
        "lts_min_support_days": lts_min_support_days,
        "checks": checks,
        "tool_rc": [{"tool": run["tool"], "rc": run["rc"]} for run in runs],
        "lts_criteria": lts_declaration["criteria"],
    }
    digest = hashlib.sha256(
        json.dumps(stable_digest_payload, sort_keys=True, separators=(",", ":")).encode(
            "utf-8"
        )
    ).hexdigest()

    return {
        "schema": SCHEMA,
        "policy_id": POLICY_ID,
        "lts_policy_id": LTS_POLICY_ID,
        "created_utc": _created_utc(),
        "seed": seed,
        "qualified_release_count": qualified_release_count,
        "min_qualified_releases": min_qualified_releases,
        "lts_min_support_days": lts_min_support_days,
        "evidence_artifacts": {
            name: path.as_posix() for name, path in paths.items()
        },
        "tool_runs": runs,
        "checks": checks,
        "total_failures": total_failures,
        "evidence_summary": evidence_summary,
        "lts_declaration": lts_declaration,
        "digest": digest,
    }


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--seed", type=int, default=DEFAULT_SEED)
    parser.add_argument("--qualified-release-count", type=int, default=3)
    parser.add_argument("--min-qualified-releases", type=int, default=3)
    parser.add_argument("--lts-min-support-days", type=int, default=730)
    parser.add_argument("--artifact-dir", default="")
    parser.add_argument("--max-failures", type=int, default=0)
    parser.add_argument("--out", default="out/maturity-qualification-v1.json")
    return parser


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)
    if args.max_failures < 0:
        print("error: max-failures must be >= 0")
        return 2
    if args.qualified_release_count < 0:
        print("error: qualified-release-count must be >= 0")
        return 2
    if args.min_qualified_releases <= 0:
        print("error: min-qualified-releases must be > 0")
        return 2
    if args.lts_min_support_days <= 0:
        print("error: lts-min-support-days must be > 0")
        return 2

    out_path = Path(args.out)
    artifact_dir = Path(args.artifact_dir) if args.artifact_dir else out_path.parent

    report = run_qualification(
        seed=args.seed,
        artifact_dir=artifact_dir,
        qualified_release_count=args.qualified_release_count,
        min_qualified_releases=args.min_qualified_releases,
        lts_min_support_days=args.lts_min_support_days,
    )
    report["max_failures"] = args.max_failures
    report["qualification_pass"] = (
        report["total_failures"] <= args.max_failures
        and report["lts_declaration"]["eligible"] is True
    )

    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(f"maturity-qualification: {out_path}")
    print(f"total_failures: {report['total_failures']}")
    print(f"lts_eligible: {report['lts_declaration']['eligible']}")
    print(f"qualification_pass: {report['qualification_pass']}")
    return 0 if report["qualification_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
