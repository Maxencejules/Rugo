"""M23 acceptance: hardware matrix v3 tier checks and contract references."""

from pathlib import Path

import pytest


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (_repo_root() / relpath).read_text(encoding="utf-8")


@pytest.mark.parametrize(
    "fixture_name,tier,machine",
    [
        ("qemu_serial_blk_q35", "tier0", "q35"),
        ("qemu_serial_blk_i440fx", "tier1", "pc/i440fx"),
    ],
)
def test_storage_smoke_matrix_v3(request, fixture_name, tier, machine):
    """Tier 0 and Tier 1 must keep deterministic storage probe/rw markers."""
    out = request.getfixturevalue(fixture_name).stdout
    assert "RUGO: boot ok" in out, (
        f"{tier} ({machine}) missing boot marker for v3 matrix run. Got:\n{out}"
    )
    assert "BLK: found virtio-blk" in out, (
        f"{tier} ({machine}) missing storage probe marker for v3. Got:\n{out}"
    )
    assert "BLK: rw ok" in out, (
        f"{tier} ({machine}) missing storage rw marker for v3. Got:\n{out}"
    )


@pytest.mark.parametrize(
    "fixture_name,tier,machine",
    [
        ("qemu_serial_net_q35", "tier0", "q35"),
        ("qemu_serial_net_i440fx", "tier1", "pc/i440fx"),
    ],
)
def test_network_smoke_matrix_v3(request, fixture_name, tier, machine):
    """Tier 0 and Tier 1 must keep deterministic network probe/runtime markers."""
    out = request.getfixturevalue(fixture_name).stdout
    assert "RUGO: boot ok" in out, (
        f"{tier} ({machine}) missing boot marker for v3 matrix run. Got:\n{out}"
    )
    assert "NET: virtio-net ready" in out, (
        f"{tier} ({machine}) missing network ready marker for v3. Got:\n{out}"
    )
    assert "NET: udp echo" in out, (
        f"{tier} ({machine}) missing UDP echo marker for v3. Got:\n{out}"
    )


def test_matrix_v3_contract_and_gate_schema():
    """Support matrix v3 must define tiers, schema, and gate bindings."""
    matrix = _read("docs/hw/support_matrix_v3.md")

    for token in [
        "Tier 0",
        "Tier 1",
        "Tier 2",
        "Tier 3",
        "Schema identifier: `rugo.hw_matrix_evidence.v3`",
        "Local gate: `make test-hw-matrix-v3`",
        "Firmware sub-gate: `make test-firmware-attestation-v1`",
        "Hardware support claims are bounded to matrix evidence only.",
    ]:
        assert token in matrix
