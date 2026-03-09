"""M19 aggregate gate: network stack v2 contract and gate wiring."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_network_stack_v2_gate_wiring_and_artifacts():
    required = [
        "docs/M19_EXECUTION_BACKLOG.md",
        "docs/net/network_stack_contract_v2.md",
        "docs/net/socket_contract_v2.md",
        "docs/net/tcp_profile_v2.md",
        "docs/net/interop_matrix_v2.md",
        "tools/run_net_interop_matrix_v2.py",
        "tools/run_net_soak_v2.py",
        "tests/net/v2_model.py",
        "tests/net/test_tcp_interop_v2.py",
        "tests/net/test_ipv6_interop_v2.py",
        "tests/net/test_dns_stub_v2.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M19 artifact: {rel}"

    makefile = _read("Makefile")
    ci = _read(".github/workflows/ci.yml")
    backlog = _read("docs/M19_EXECUTION_BACKLOG.md")
    milestones = _read("MILESTONES.md")
    status = _read("docs/STATUS.md")

    assert "test-network-stack-v2" in makefile
    for entry in [
        "tools/run_net_interop_matrix_v2.py --out $(OUT)/net-interop-v2.json",
        "tools/run_net_soak_v2.py --out $(OUT)/net-soak-v2.json",
        "tests/net/test_tcp_interop_v2.py",
        "tests/net/test_ipv6_interop_v2.py",
        "tests/net/test_dns_stub_v2.py",
        "tests/net/test_network_gate_v2.py",
    ]:
        assert entry in makefile
    assert "pytest-network-stack-v2.xml" in makefile

    assert "Network stack v2 gate" in ci
    assert "make test-network-stack-v2" in ci
    assert "network-stack-v2-artifacts" in ci
    assert "out/pytest-network-stack-v2.xml" in ci
    assert "out/net-interop-v2.json" in ci
    assert "out/net-soak-v2.json" in ci

    assert "Status: done" in backlog
    assert "M19" in milestones
    assert "M19" in status
