"""M23 acceptance: firmware resiliency docs are present and policy-shaped."""

from pathlib import Path


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (_repo_root() / relpath).read_text(encoding="utf-8").lower()


def test_firmware_resiliency_docs_v1_present():
    root = _repo_root()
    required = [
        "docs/hw/support_matrix_v3.md",
        "docs/hw/firmware_resiliency_policy_v1.md",
        "docs/security/measured_boot_attestation_v1.md",
        "tools/collect_measured_boot_report_v1.py",
    ]
    for rel in required:
        assert (root / rel).is_file(), f"missing M23 firmware artifact: {rel}"

    policy = _read("docs/hw/firmware_resiliency_policy_v1.md")
    attestation = _read("docs/security/measured_boot_attestation_v1.md")
    matrix = _read("docs/hw/support_matrix_v3.md")
    assert "protect" in policy
    assert "detect" in policy
    assert "recover" in policy
    assert "tpm event log" in attestation
    assert "pcr" in attestation
    assert "make test-firmware-attestation-v1" in policy
    assert "schema identifier: `rugo.measured_boot_report.v1`" in attestation
    assert "firmware sub-gate: `make test-firmware-attestation-v1`" in matrix
