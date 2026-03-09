"""M23 aggregate gate: firmware attestation artifacts and report are valid."""

import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import collect_measured_boot_report_v1 as measured_boot  # noqa: E402


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_firmware_attestation_gate_v1(tmp_path: Path):
    required = [
        "docs/M23_EXECUTION_BACKLOG.md",
        "docs/hw/firmware_resiliency_policy_v1.md",
        "docs/security/measured_boot_attestation_v1.md",
        "tools/collect_measured_boot_report_v1.py",
        "tests/hw/test_firmware_resiliency_docs_v1.py",
        "tests/hw/test_measured_boot_attestation_v1.py",
        "tests/hw/test_tpm_eventlog_schema_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing gate artifact: {rel}"

    roadmap = _read("docs/M21_M34_MATURITY_PARITY_ROADMAP.md")
    makefile = _read("Makefile")
    ci = _read(".github/workflows/ci.yml")
    backlog = _read("docs/M23_EXECUTION_BACKLOG.md")
    milestones = _read("MILESTONES.md")
    status = _read("docs/STATUS.md")

    assert "test-firmware-attestation-v1" in roadmap
    assert "test-firmware-attestation-v1" in makefile
    for entry in [
        "tools/collect_measured_boot_report_v1.py --out $(OUT)/measured-boot-v1.json",
        "tests/hw/test_firmware_resiliency_docs_v1.py",
        "tests/hw/test_measured_boot_attestation_v1.py",
        "tests/hw/test_tpm_eventlog_schema_v1.py",
        "tests/hw/test_firmware_attestation_gate_v1.py",
    ]:
        assert entry in makefile
    assert "pytest-firmware-attestation-v1.xml" in makefile

    assert "Firmware attestation v1 gate" in ci
    assert "make test-firmware-attestation-v1" in ci
    assert "firmware-attestation-v1-artifacts" in ci
    assert "out/pytest-firmware-attestation-v1.xml" in ci
    assert "out/measured-boot-v1.json" in ci

    assert "Status: done" in backlog
    assert "M23" in milestones
    assert "M23" in status

    out = tmp_path / "measured-boot-v1.json"
    rc = measured_boot.main(["--out", str(out)])
    assert rc == 0
    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.measured_boot_report.v1"
    assert data["policy_pass"] is True
