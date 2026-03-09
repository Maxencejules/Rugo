"""M19 acceptance: IPv6 interop v2 baseline behavior."""

from pathlib import Path
import sys

sys.path.append(str(Path(__file__).resolve().parent))
from v2_model import IPv6InteropV2Model  # noqa: E402


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_ipv6_interop_docs_declare_required_v2_behavior():
    contract = _read("docs/net/network_stack_contract_v2.md")
    matrix = _read("docs/net/interop_matrix_v2.md")

    for token in [
        "IPv6 ND + ICMPv6 interop baseline promoted to v2 coverage",
        "deterministic DNS-stub behavior",
        "tests/net/test_ipv6_interop_v2.py",
    ]:
        assert token in contract

    for token in [
        "dual-stack preference/fallback",
        "ICMPv6 echo payload parity",
        "rugo.net_interop_matrix.v2",
    ]:
        assert token in matrix


def test_ipv6_neighbor_exchange_and_cache_expiry():
    model = IPv6InteropV2Model()
    target = "2001:db8::10"
    assert model.exchange_ns_na(target_ip=target, mac="52:54:00:12:34:56") == 0
    assert model.resolve(target) == "52:54:00:12:34:56"

    model.tick(44)
    assert model.resolve(target) == "52:54:00:12:34:56"
    model.tick(1)
    assert model.resolve(target) is None


def test_icmpv6_and_dual_stack_selection_are_deterministic():
    model = IPv6InteropV2Model()
    rc, payload = model.icmpv6_echo(b"m19-v2")
    assert rc == 0
    assert payload == b"m19-v2"

    assert model.select_stack(prefer_ipv6=True, has_aaaa=True) == "ipv6"
    assert model.select_stack(prefer_ipv6=True, has_aaaa=False) == "ipv4"
    assert model.select_stack(prefer_ipv6=False, has_aaaa=True) == "ipv4"
