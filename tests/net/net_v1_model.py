"""Deterministic reference models for M12 network stack v1 semantics."""

from __future__ import annotations

from dataclasses import dataclass


TCP_STATES = {
    "CLOSED",
    "LISTEN",
    "SYN_SENT",
    "SYN_RECEIVED",
    "ESTABLISHED",
    "FIN_WAIT_1",
    "FIN_WAIT_2",
    "CLOSE_WAIT",
    "LAST_ACK",
    "TIME_WAIT",
}


class TcpConnectionModel:
    """Small deterministic TCP transition model for M12 contract tests."""

    def __init__(self):
        self.state = "CLOSED"
        self.unacked_bytes = 0
        self.transition_log: list[str] = []

    def _set_state(self, state: str) -> None:
        assert state in TCP_STATES
        self.state = state
        self.transition_log.append(state)

    def listen(self) -> int:
        if self.state != "CLOSED":
            return -1
        self._set_state("LISTEN")
        return 0

    def active_open(self) -> int:
        if self.state != "CLOSED":
            return -1
        self._set_state("SYN_SENT")
        return 0

    def recv_syn(self) -> int:
        if self.state != "LISTEN":
            return -1
        self._set_state("SYN_RECEIVED")
        return 0

    def recv_syn_ack(self) -> int:
        if self.state != "SYN_SENT":
            return -1
        self._set_state("ESTABLISHED")
        return 0

    def recv_ack(self) -> int:
        if self.state == "SYN_RECEIVED":
            self._set_state("ESTABLISHED")
            return 0
        if self.state == "FIN_WAIT_1":
            self._set_state("FIN_WAIT_2")
            return 0
        if self.state == "LAST_ACK":
            self._set_state("CLOSED")
            return 0
        if self.state == "ESTABLISHED":
            self.unacked_bytes = 0
            return 0
        return -1

    def send_data(self, length: int) -> int:
        if self.state != "ESTABLISHED" or length <= 0:
            return -1
        self.unacked_bytes += length
        return length

    def recv_fin(self) -> int:
        if self.state == "ESTABLISHED":
            self._set_state("CLOSE_WAIT")
            return 0
        if self.state == "FIN_WAIT_2":
            self._set_state("TIME_WAIT")
            return 0
        return -1

    def close(self) -> int:
        if self.state == "ESTABLISHED":
            self._set_state("FIN_WAIT_1")
            return 0
        if self.state == "CLOSE_WAIT":
            self._set_state("LAST_ACK")
            return 0
        return -1

    def time_wait_expire(self) -> int:
        if self.state != "TIME_WAIT":
            return -1
        self._set_state("CLOSED")
        return 0

    def recv_rst(self) -> int:
        if self.state == "CLOSED":
            return -1
        self._set_state("CLOSED")
        self.unacked_bytes = 0
        return 0


@dataclass
class RetransmissionPolicyModel:
    """Deterministic RTO/backoff model for M12 retransmission policy."""

    initial_rto_ms: int = 200
    max_rto_ms: int = 3200
    max_retries: int = 5

    def __post_init__(self) -> None:
        self.in_flight_bytes = 0
        self.retry_count = 0
        self.current_rto_ms = self.initial_rto_ms
        self.elapsed_ms = 0
        self.failed = False

    def send(self, length: int) -> int:
        if length <= 0 or self.failed:
            return -1
        self.in_flight_bytes += length
        self.retry_count = 0
        self.current_rto_ms = self.initial_rto_ms
        self.elapsed_ms = 0
        return length

    def ack_all(self) -> int:
        if self.in_flight_bytes == 0:
            return -1
        self.in_flight_bytes = 0
        self.retry_count = 0
        self.current_rto_ms = self.initial_rto_ms
        self.elapsed_ms = 0
        return 0

    def tick(self, delta_ms: int) -> list[str]:
        if delta_ms < 0:
            return ["invalid_delta"]
        if self.in_flight_bytes == 0 or self.failed:
            return []
        self.elapsed_ms += delta_ms
        events: list[str] = []
        while self.elapsed_ms >= self.current_rto_ms and self.in_flight_bytes > 0:
            self.elapsed_ms -= self.current_rto_ms
            self.retry_count += 1
            events.append("retransmit")
            if self.retry_count >= self.max_retries:
                self.failed = True
                events.append("timeout")
                break
            self.current_rto_ms = min(self.current_rto_ms * 2, self.max_rto_ms)
        return events


@dataclass
class NeighborEntry:
    mac: str
    reachable_s: int


class IPv6NeighborModel:
    """Deterministic ND + ICMPv6 essentials model for M12 baseline tests."""

    def __init__(self):
        self.cache: dict[str, NeighborEntry] = {}

    def send_ns(self, target_ip: str) -> dict[str, str]:
        return {"type": "NS", "target": target_ip}

    def recv_na(self, target_ip: str, mac: str) -> int:
        if not target_ip or not mac:
            return -1
        self.cache[target_ip] = NeighborEntry(mac=mac, reachable_s=30)
        return 0

    def resolve(self, target_ip: str) -> str | None:
        ent = self.cache.get(target_ip)
        return ent.mac if ent else None

    def tick(self, delta_s: int) -> None:
        stale: list[str] = []
        for ip, ent in self.cache.items():
            ent.reachable_s = max(0, ent.reachable_s - max(0, delta_s))
            if ent.reachable_s == 0:
                stale.append(ip)
        for ip in stale:
            del self.cache[ip]

    def icmpv6_echo_reply(self, payload: bytes) -> tuple[int, bytes]:
        if payload is None:
            return -1, b""
        return 0, payload
