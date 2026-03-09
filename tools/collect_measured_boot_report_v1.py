#!/usr/bin/env python3
"""Generate measured-boot attestation report for M23."""

from __future__ import annotations

import argparse
import hashlib
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Set


def _digest(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def _parse_pcrs(raw: str) -> List[int]:
    return [int(v.strip()) for v in raw.split(",") if v.strip()]


def build_report(
    platform: str,
    pcrs: List[int],
    policy_profile: str,
    required_pcrs: List[int] | None = None,
) -> Dict[str, object]:
    required: Set[int] = set(required_pcrs or [0, 2, 4, 7])
    pcr_set = set(pcrs)
    missing = sorted(required - pcr_set)
    event_log = [
        {
            "index": 0,
            "pcr": 0,
            "type": "firmware",
            "component": "uefi_firmware",
            "digest": _digest("firmware-v1"),
        },
        {
            "index": 1,
            "pcr": 2,
            "type": "bootloader",
            "component": "limine",
            "digest": _digest("bootloader-v1"),
        },
        {
            "index": 2,
            "pcr": 4,
            "type": "kernel",
            "component": "rugo-kernel",
            "digest": _digest("kernel-v1"),
        },
        {
            "index": 3,
            "pcr": 7,
            "type": "secure-config",
            "component": "secure-boot-policy",
            "digest": _digest("secure-config-v1"),
        },
    ]

    failures = [f"missing_pcr_{n}" for n in missing]
    policy_pass = len(missing) == 0
    verdict = {
        "status": "pass" if policy_pass else "fail",
        "reasons": ["all_required_pcrs_present"] if policy_pass else failures,
    }

    return {
        "schema": "rugo.measured_boot_report.v1",
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "firmware_policy": "rugo.firmware_resiliency_policy.v1",
        "attestation_contract_id": "rugo.measured_boot_attestation.v1",
        "platform": platform,
        "policy_profile": policy_profile,
        "pcr_bank": "sha256",
        "expected_pcrs": sorted(required),
        "pcrs": sorted(pcr_set),
        "tpm_event_log": event_log,
        "event_count": len(event_log),
        "policy_pass": policy_pass,
        "failures": failures,
        "attestation_verdict": verdict,
    }


def _build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--platform", default="qemu-q35")
    p.add_argument("--pcrs", default="0,2,4,7")
    p.add_argument("--required-pcrs", default="0,2,4,7")
    p.add_argument("--policy-profile", default="firmware-attestation-v1")
    p.add_argument("--out", default="out/measured-boot-v1.json")
    return p


def main(argv: List[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)
    pcrs = _parse_pcrs(args.pcrs)
    required = _parse_pcrs(args.required_pcrs)
    report = build_report(
        platform=args.platform,
        pcrs=pcrs,
        policy_profile=args.policy_profile,
        required_pcrs=required,
    )
    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(f"measured-boot-report: {out_path}")
    print(f"policy_pass: {report['policy_pass']}")
    return 0 if report["policy_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
