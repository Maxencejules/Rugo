# TCP multi-segment send window — contract v1

Status: boot-verified via `make test-tcp-sndwin-v1`
Source: `kernel_rs/src/tcp.rs` (`SndWindow`, `tcp_sndwin_selftest`); dispatch
`sys_net_query` (id 49) op 19.
Proof: `tests/runtime/test_tcp_sndwin_v1.py`.

Full-OS guide Part II.6 (networking maturity): a real sliding send window over
**multiple outstanding segments**, bounded by `min(cwnd, rwnd)`, beyond the live
connection's single-outstanding-segment model
([`tcp_rto_v1.md`](tcp_rto_v1.md), [`tcp_congestion_v1.md`](tcp_congestion_v1.md)).

## Behaviour

`SndWindow` tracks `una`/`nxt`, the congestion window `cwnd`, the peer receive
window `rwnd`, and up to 16 outstanding segment lengths:

- **`usable()`** = `min(cwnd, rwnd) − inflight()` where `inflight = nxt − una`.
- **`send(len)`**: allowed only if `len` fits `usable()` and a segment slot is
  free; records the segment and advances `nxt`.
- **`ack(cumulative)`**: retires every outstanding segment fully covered by the
  ACK (wrapping arithmetic), sliding `una` forward and freeing window space.

## Acceptance

`make test-tcp-sndwin-v1`: the boot self-test fills the window with three 512-byte
segments (1536 ≤ cwnd 2000) and confirms a fourth is refused (cwnd-bound), then a
cumulative ACK for two segments slides `una` and frees space (in-flight 512),
then two more segments fit and a third overflows, and finally shrinking `rwnd`
below the in-flight blocks further sending — `TCP: sndwin ok`, with the
single-segment `TCP: rto ok` / `TCP: cc ok` still green.

## v1 boundary / carry-forward

- The **window accounting** (multi-segment in-flight tracking, cwnd/rwnd bounding,
  cumulative-ACK retirement) is implemented + verified. Unifying it with the live
  connection so the wire path actually transmits multiple segments — i.e.
  replacing the single retransmit slot with per-segment RTO/retransmit slots and a
  send buffer — is the larger refactor and is carry-forward (the single-segment
  live path and its RTO/RTT/CC/fast-retransmit tests are deliberately unchanged).
- No SACK / selective retransmit; no Nagle / delayed-ACK.
