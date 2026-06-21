# TCP RTT estimation (SRTT/RTTVAR + Karn) — contract v1

Status: boot-verified via `make test-tcp-rtt-v1`
Source: `kernel_rs/src/tcp.rs` (`tcp_rtt_update`, `tcp_rt_arm`, `tcp_rt_ack`,
`tcp_rt_tick`, `TCP_TICK`, the `srtt8`/`rttvar4`/`rto_ticks`/`rtt_valid` fields,
`tcp_rtt_selftest`); dispatch `sys_net_query` (id 49) op 11.
Proof: `tests/runtime/test_tcp_rtt_v1.py`.

Full-OS guide Part II.6 (networking maturity), RTT estimation: derive the
retransmit timeout from measured round-trip times instead of a fixed constant —
RFC 6298 with Karn's algorithm. Builds directly on the fixed-RTO retransmit
machinery ([`tcp_rto_v1.md`](tcp_rto_v1.md)).

## Behaviour

RTT is measured in PIT ticks. `TCP_TICK` is a free-running counter advanced once
per `tcp_rt_tick` (before its early return), and `tcp_rt_arm` stamps the
outstanding segment's send tick. When a cumulative ACK fully covers that segment
(`tcp_rt_ack`), the elapsed ticks are the RTT sample `R`.

The estimator is RFC 6298 in integer fixed-point (no floating point):
`srtt8 = 8·SRTT`, `rttvar4 = 4·RTTVAR`.

- **First sample:** `SRTT = R`, `RTTVAR = R/2` (`srtt8 = R<<3`, `rttvar4 = R<<1`).
- **Subsequent:** `err = R − SRTT`; `SRTT += err/8`
  (`srtt8 += err`); `RTTVAR += (|err| − RTTVAR)/4`
  (`rttvar4 += |err| − (rttvar4>>2)`) — i.e. α = 1/8, β = 1/4.
- **RTO** `= SRTT + max(1, 4·RTTVAR) = (srtt8>>3) + max(1, rttvar4)`, clamped to
  `[TCP_RTO_MIN, TCP_RTO_MAX] = [4, 6000]` ticks.

`rto_ticks` (the derived base RTO, defaulting to `TCP_RTO_TICKS` until the first
sample) **drives the live retransmit timer**: `tcp_rt_arm` loads it into
`rt_ticks_left`, and the exponential backoff in `tcp_rt_tick` shifts it
(`rto_ticks << min(retries,4)`).

**Karn's algorithm:** a retransmitted segment's ACK is ambiguous (which
transmission did it acknowledge?), so RTT is sampled **only** when the
outstanding segment was sent exactly once (`rt_retries == 0`). A new connection
resets the estimate (`conn_reset`).

## Why the proof is deterministic

QEMU's user-mode network is loss-free with near-zero latency, so adaptive-RTO
behaviour is unobservable on the live wire. `tcp_rtt_selftest` drives the tick
clock explicitly: it takes two clean samples at known deltas (R=10 then R=20) and
asserts the exact fixed-point evolution — `srtt8` 80→90, `rttvar4` 20→25, RTO
30→36 — that the new RTO arms the next segment's timer (`rt_ticks_left == 30`),
and that a forced retransmit's ACK leaves the estimate **unchanged** (Karn).

## v1 boundary / carry-forward

- One outstanding segment (inherited from the RTO slice), so one RTT sample in
  flight; a full per-socket estimator with RTT sampling via timestamps (TSopt)
  and multiple in-flight segments is carry-forward.
- No RTO floor of 1 s (RFC 6298's 2.4) — the tick model uses a 40 ms floor so the
  self-test can exercise small RTTs; a wall-clock-aligned floor is carry-forward.
- Congestion control (cwnd/ssthresh) is a separate slice
  ([`tcp_congestion_v1.md`](tcp_congestion_v1.md)).

## Acceptance

`make test-tcp-rtt-v1`: the boot transcript shows `TCP: rtt ok` (both samples
produced the exact expected SRTT/RTTVAR/RTO and Karn excluded the retransmitted
sample), alongside the still-green `TCP: rto ok`, then `GOINIT: result
shutdown-clean` and `RUGO: halt ok`, with no `GOINIT: err` and no
`TCP: rto giveup`.
