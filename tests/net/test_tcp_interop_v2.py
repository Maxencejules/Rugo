"""M19 acceptance: TCP interop v2 contract and behavior."""

from pathlib import Path
import sys

sys.path.append(str(Path(__file__).resolve().parent))
from v2_model import TcpInteropV2Model  # noqa: E402


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_network_stack_v2_docs_reference_tcp_interop_lane():
    contract = _read("docs/net/network_stack_contract_v2.md")
    profile = _read("docs/net/tcp_profile_v2.md")

    for token in [
        "Status: active release gate",
        "tests/net/test_tcp_interop_v2.py",
        "test-network-stack-v2",
    ]:
        assert token in contract

    for token in [
        "Interop pass target: `>= 0.95`",
        "three-way handshake",
        "windows-2025",
    ]:
        assert token in profile


def test_tcp_interop_model_accepts_core_peers():
    model = TcpInteropV2Model()
    for peer in ["linux-6.8", "freebsd-14.1", "windows-2025"]:
        result = model.run_case(peer=peer, scenario="three_way_handshake")
        assert result["status"] == "pass"
        assert result["negotiated_mss"] >= 1220
        assert result["window_scaling"] is True
        assert result["timestamps"] is True


def test_tcp_interop_model_handles_loss_with_bounded_retries():
    model = TcpInteropV2Model()
    case = model.run_case(
        peer="linux-6.8",
        scenario="bulk_transfer_1mib",
        loss_pct=1.5,
    )
    assert case["status"] == "pass"
    assert case["retries"] >= 1
    assert case["retries"] <= 3

    fail_case = model.run_case(
        peer="linux-6.8",
        scenario="bulk_transfer_1mib",
        loss_pct=3.2,
    )
    assert fail_case["status"] == "fail"
    assert fail_case["reason"] == "loss_budget_exceeded"


def test_tcp_interop_summary_meets_v2_target():
    model = TcpInteropV2Model()
    results = [
        model.run_case("linux-6.8", "three_way_handshake"),
        model.run_case("linux-6.8", "bulk_transfer_1mib", loss_pct=1.0),
        model.run_case("freebsd-14.1", "half_close"),
        model.run_case("windows-2025", "reconnect_after_rst", loss_pct=1.5),
    ]
    summary = model.summarize(results, target_pass_rate=0.95)

    assert summary["schema"] == "rugo.net_tcp_interop_report.v2"
    assert summary["pass_rate"] == 1.0
    assert summary["failed_cases"] == 0
    assert summary["meets_target"] is True
