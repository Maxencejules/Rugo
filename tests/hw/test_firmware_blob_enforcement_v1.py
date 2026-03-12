"""M53 PR-2: deterministic firmware blob enforcement checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_native_driver_diagnostics_v1 as diagnostics  # noqa: E402


def _firmware_audit(data: dict, audit_id: str) -> dict:
    rows = [entry for entry in data["firmware_audits"] if entry["audit_id"] == audit_id]
    assert len(rows) == 1
    return rows[0]


def _out_path(name: str) -> Path:
    path = ROOT / "out" / "pytest-m53" / name
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        path.unlink()
    return path


def test_firmware_blob_enforcement_v1_report_enforces_signed_manifests():
    report = diagnostics.run_diagnostics(seed=20260311)
    policy = report["firmware_policy"]
    assert report["summary"]["firmware"]["pass"] is True
    assert policy["policy_id"] == "rugo.firmware_blob_policy.v1"
    assert policy["manifest_schema"] == "rugo.firmware_manifest.v1"
    assert policy["manifest_required"] is True
    assert policy["signature_required"] is True
    assert policy["measured_boot_reference_required"] is True
    assert policy["storage_outside_kernel_image"] is True

    assert _firmware_audit(report, "gpu_guc_signed")["marker"] == "FW: allow signed"
    assert _firmware_audit(report, "wifi_ucode_unsigned")["marker"] == "FW: denied unsigned"
    assert _firmware_audit(report, "gpu_vbios_missing_manifest")["decision"] == (
        "denied_missing_manifest"
    )
    assert _firmware_audit(report, "nvme_admin_hash_mismatch")["decision"] == (
        "denied_hash_mismatch"
    )


def test_firmware_blob_enforcement_v1_detects_unsigned_allow_regression():
    out = _out_path("native-driver-firmware-fail.json")
    rc = diagnostics.main(
        [
            "--inject-failure",
            "firmware_unsigned_denied",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    row = _firmware_audit(data, "wifi_ucode_unsigned")
    assert row["status"] == "fail"
    assert row["decision"] == "allow_unexpected"
    assert "firmware_unsigned_denied" in data["failures"]
