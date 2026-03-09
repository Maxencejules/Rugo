"""M23 acceptance: measured boot attestation report generation."""

import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import collect_measured_boot_report_v1 as tool  # noqa: E402


def test_measured_boot_attestation_report_schema(tmp_path: Path):
    out = tmp_path / "measured-boot-v1.json"
    rc = tool.main(["--out", str(out)])
    assert rc == 0
    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.measured_boot_report.v1"
    assert data["firmware_policy"] == "rugo.firmware_resiliency_policy.v1"
    assert data["attestation_contract_id"] == "rugo.measured_boot_attestation.v1"
    assert data["policy_pass"] is True
    assert set([0, 2, 4, 7]).issubset(set(data["pcrs"]))
    assert data["attestation_verdict"]["status"] == "pass"


def test_measured_boot_attestation_rejects_missing_required_pcr(tmp_path: Path):
    out = tmp_path / "measured-boot-v1-missing-pcr.json"
    rc = tool.main(["--pcrs", "0,2,4", "--out", str(out)])
    assert rc == 1
    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["policy_pass"] is False
    assert "missing_pcr_7" in data["failures"]
    assert data["attestation_verdict"]["status"] == "fail"
