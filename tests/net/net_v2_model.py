"""Deterministic reference models for M19 network stack v2 semantics."""

from __future__ import annotations

from dataclasses import dataclass


TCP_V2_SCENARIOS = frozenset(
    {
        "three_way_handshake",
        "bulk_transfer_1mib",
        "half_close",
        "reconnect_after_rst",
    }
)


@dataclass(frozen=True)
class TcpPeerProfile:
    name: str
    min_mss: int
    window_scaling: bool
    timestamps: bool


class TcpInteropV2Model:
    """Small deterministic TCP interop model for M19 contract tests."""

    def __init__(self):
        self.peers = {
            "linux-6.8": TcpPeerProfile(
                name="linux-6.8",
                min_mss=1460,
                window_scaling=True,
                timestamps=True,
            ),
            "freebsd-14.1": TcpPeerProfile(
                name="freebsd-14.1",
                min_mss=1440,
                window_scaling=True,
                timestamps=True,
            ),
            "windows-2025": TcpPeerProfile(
                name="windows-2025",
                min_mss=1400,
                window_scaling=True,
                timestamps=True,
            ),
        }

    def run_case(self, peer: str, scenario: str, loss_pct: float = 0.0) -> dict:
        profile = self.peers.get(peer)
        if profile is None:
            return {
                "peer": peer,
                "scenario": scenario,
                "status": "fail",
                "reason": "unknown_peer",
                "retries": 0,
                "negotiated_mss": 0,
            }
        if scenario not in TCP_V2_SCENARIOS:
            return {
                "peer": peer,
                "scenario": scenario,
                "status": "fail",
                "reason": "unknown_scenario",
                "retries": 0,
                "negotiated_mss": 0,
            }
        if loss_pct < 0.0 or loss_pct > 100.0:
            return {
                "peer": peer,
                "scenario": scenario,
                "status": "fail",
                "reason": "invalid_loss",
                "retries": 0,
                "negotiated_mss": 0,
            }

        retries = 0
        if scenario in {"bulk_transfer_1mib", "reconnect_after_rst"}:
            retries = min(3, int(loss_pct // 0.5))

        status = "pass" if loss_pct <= 2.5 else "fail"
        reason = "ok" if status == "pass" else "loss_budget_exceeded"
        return {
            "peer": peer,
            "scenario": scenario,
            "status": status,
            "reason": reason,
            "retries": retries,
            "negotiated_mss": max(1220, profile.min_mss),
            "window_scaling": profile.window_scaling,
            "timestamps": profile.timestamps,
        }

    def summarize(self, cases: list[dict], target_pass_rate: float = 0.95) -> dict:
        total = len(cases)
        passed = sum(1 for case in cases if case.get("status") == "pass")
        failed = total - passed
        pass_rate = round((passed / total), 4) if total else 0.0
        return {
            "schema": "rugo.net_tcp_interop_report.v2",
            "total_cases": total,
            "passed_cases": passed,
            "failed_cases": failed,
            "pass_rate": pass_rate,
            "target_pass_rate": target_pass_rate,
            "meets_target": total > 0 and pass_rate >= target_pass_rate,
        }


@dataclass
class NeighborEntryV2:
    mac: str
    reachable_s: int


class IPv6InteropV2Model:
    """Deterministic ND and dual-stack baseline model for M19."""

    def __init__(self):
        self.cache: dict[str, NeighborEntryV2] = {}

    def exchange_ns_na(self, target_ip: str, mac: str) -> int:
        if not target_ip or not mac:
            return -1
        self.cache[target_ip] = NeighborEntryV2(mac=mac, reachable_s=45)
        return 0

    def resolve(self, target_ip: str) -> str | None:
        entry = self.cache.get(target_ip)
        return entry.mac if entry else None

    def tick(self, delta_s: int) -> None:
        if delta_s <= 0:
            return
        stale: list[str] = []
        for target_ip, entry in self.cache.items():
            entry.reachable_s = max(0, entry.reachable_s - delta_s)
            if entry.reachable_s == 0:
                stale.append(target_ip)
        for target_ip in stale:
            del self.cache[target_ip]

    def icmpv6_echo(self, payload: bytes) -> tuple[int, bytes]:
        if payload is None:
            return -1, b""
        return 0, payload

    def select_stack(self, prefer_ipv6: bool, has_aaaa: bool) -> str:
        if prefer_ipv6 and has_aaaa:
            return "ipv6"
        return "ipv4"


@dataclass(frozen=True)
class DnsAnswer:
    name: str
    qtype: str
    value: str
    ttl_s: int


class DnsStubV2Model:
    """Deterministic DNS-stub cache model for v2 service behavior tests."""

    def __init__(self):
        self._zone = {
            ("svc.rugo.local", "A"): DnsAnswer(
                name="svc.rugo.local",
                qtype="A",
                value="10.0.2.15",
                ttl_s=30,
            ),
            ("svc.rugo.local", "AAAA"): DnsAnswer(
                name="svc.rugo.local",
                qtype="AAAA",
                value="2001:db8::15",
                ttl_s=30,
            ),
        }
        self._cache: dict[tuple[str, str], DnsAnswer] = {}

    def query(self, name: str, qtype: str) -> tuple[int, list[str], int]:
        key = (name.strip().lower(), qtype.strip().upper())
        cached = self._cache.get(key)
        if cached is not None and cached.ttl_s > 0:
            return 0, [cached.value], cached.ttl_s

        answer = self._zone.get(key)
        if answer is None:
            return -2, [], 0

        self._cache[key] = answer
        return 0, [answer.value], answer.ttl_s

    def tick(self, delta_s: int) -> None:
        if delta_s <= 0:
            return
        updated: dict[tuple[str, str], DnsAnswer] = {}
        for key, answer in self._cache.items():
            next_ttl = max(0, answer.ttl_s - delta_s)
            if next_ttl > 0:
                updated[key] = DnsAnswer(
                    name=answer.name,
                    qtype=answer.qtype,
                    value=answer.value,
                    ttl_s=next_ttl,
                )
        self._cache = updated
