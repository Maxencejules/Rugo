# TCP congestion control (slow start + cwnd) — contract v1

Status: boot-verified via `make test-tcp-cc-v1`
Source: `kernel_rs/src/tcp.rs` (`cc_on_ack`, `cc_on_timeout`, the `cwnd`/`ssthresh`
fields, `TCP_MSS`/`TCP_IW`/`TCP_INIT_SSTHRESH`, `tcp_cc_selftest`); dispatch
`sys_net_query` (id 49) op 12.
Proof: `tests/runtime/test_tcp_cc_v1.py`.

Full-OS guide Part II.6 (networking maturity), congestion control: maintain a
congestion window that grows on success and collapses on loss — RFC 5681 slow
start + congestion avoidance. Builds on the retransmit/RTO + RTT slices
([`tcp_rto_v1.md`](tcp_rto_v1.md), [`tcp_rtt_v1.md`](tcp_rtt_v1.md)).

## Behaviour

`cwnd` and `ssthresh` are byte windows; `TCP_MSS = 512`, `TCP_IW = 1·SMSS`,
initial `ssthresh = 65535` (so a connection begins in slow start). They are
driven by the same cumulative-ACK and RTO-timeout events as the retransmit timer:

- **`cc_on_ack(acked)`** (from `tcp_rt_ack` when a data segment is fully acked):
  - **slow start** (`cwnd < ssthresh`): `cwnd += min(acked, SMSS)` — roughly
    doubles per RTT.
  - **congestion avoidance** (`cwnd ≥ ssthresh`): `cwnd += max(1, SMSS²/cwnd)` —
    roughly one SMSS per RTT.
- **`cc_on_timeout()`** (from `tcp_rt_tick` when the RTO fires):
  `ssthresh = max(cwnd/2, 2·SMSS)`; `cwnd = 1·SMSS` — collapse to one segment and
  restart slow start (RFC 5681 §3.1).

A new connection resets `cwnd`/`ssthresh` to the initial window (`conn_reset`).

## Why the proof is deterministic

QEMU's user-mode network is loss-free and near-zero-latency, so the window never
naturally leaves the initial slow-start ramp on the live wire. `tcp_cc_selftest`
drives full-MSS send+ACK and forced-timeout events and asserts the exact window
arithmetic: slow start `512→1024→1536`; congestion avoidance `1024→1280`
(`+512²/1024 = 256`); timeout `cwnd 4096→512, ssthresh→2048`.

## v1 boundary / carry-forward

- **cwnd is computed but does not yet gate sending.** This v1 keeps the single
  outstanding-segment model of the RTO slice, so the send path always emits one
  segment regardless of `cwnd`. A multi-segment send window that actually clamps
  in-flight bytes to `min(cwnd, rwnd)` is carry-forward (it depends on the same
  per-socket multi-segment refactor the RTO/RTT slices defer).
- **No fast retransmit / fast recovery.** Only the RTO-timeout reaction is wired
  (3-dup-ACK detection + `cwnd = ssthresh + 3·SMSS` recovery is carry-forward).
- No appropriate-byte-counting, no PRR, no ECN.

## Acceptance

`make test-tcp-cc-v1`: the boot transcript shows `TCP: cc ok` (slow start,
congestion avoidance, and the timeout collapse all produced the exact expected
`cwnd`/`ssthresh`), alongside `TCP: rto ok` and `TCP: rtt ok`, then
`GOINIT: result shutdown-clean` and `RUGO: halt ok`, with no `GOINIT: err` and no
`TCP: rto giveup`.
