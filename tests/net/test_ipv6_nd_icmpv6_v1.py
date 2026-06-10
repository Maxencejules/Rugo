"""M12 acceptance: IPv6 ND + ICMPv6 baseline semantics."""

from pathlib import Path
import sys

sys.path.append(str(Path(__file__).resolve().parent))
from net_v1_model import IPv6NeighborModel  # noqa: E402


def _read_ipv6_doc() -> str:
    path = Path(__file__).resolve().parents[2] / "docs" / "net" / "ipv6_baseline_v1.md"
    return path.read_text(encoding="utf-8")


def test_ipv6_doc_declares_nd_and_icmpv6_baseline():
    text = _read_ipv6_doc()
    assert "Neighbor Solicitation" in text
    assert "Neighbor Advertisement" in text
    assert "ICMPv6 essentials" in text


def test_neighbor_discovery_cache_lifecycle():
    model = IPv6NeighborModel()
    target = "fe80::10"
    ns = model.send_ns(target)
    assert ns["type"] == "NS"
    assert ns["target"] == target

    assert model.recv_na(target, "52:54:00:12:34:56") == 0
    assert model.resolve(target) == "52:54:00:12:34:56"

    model.tick(29)
    assert model.resolve(target) == "52:54:00:12:34:56"
    model.tick(1)
    assert model.resolve(target) is None


def test_icmpv6_echo_reply_is_payload_preserving():
    model = IPv6NeighborModel()
    rc, payload = model.icmpv6_echo_reply(b"hello-v6")
    assert rc == 0
    assert payload == b"hello-v6"
