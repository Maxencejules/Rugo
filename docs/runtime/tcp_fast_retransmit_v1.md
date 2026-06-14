# TCP fast retransmit + fast recovery — contract v1

Status: boot-verified via `make test-tcp-fastrexmit-v1`
Source: `kernel_rs/src/tcp.rs` (`tcp_rt_ack` duplicate-ACK branch,
`cc_fast_recovery`, the `dup_acks` field, `tcp_fastrexmit_selftest`); dispatch
`sys_net_query` (id 49) op 16.
Proof: `tests/runtime/test_tcp_fastrexmit_v1.py`.

Full-OS guide Part II.6 (networking maturity), fast retransmit: retransmit on
three duplicate ACKs instead of waiting for the RTO. Builds on the RTO + RTT +
congestion slices ([`tcp_rto_v1.md`](tcp_rto_v1.md),
[`tcp_congestion_v1.md`](tcp_congestion_v1.md)).

## Behaviour

A **duplicate ACK** is an ACK that re-acknowledges the same point (`ack ==
snd_una`) while the outstanding segment is still unacknowledged. `tcp_rt_ack`
counts consecutive duplicates in `dup_acks`; the **third** triggers, per RFC 5681
§3.2:

- **fast retransmit**: re-send the outstanding segment immediately (not waiting
  for the RTO) and restart the RTO timer;
- **fast recovery** (`cc_fast_recovery`): `ssthresh = max(cwnd/2, 2·SMSS)`,
  `cwnd = ssthresh + 3·SMSS` (the three segments that left the network).

`dup_acks` resets when new data is acknowledged, on a fresh send, on a connection
reset, and on an RTO timeout (which exits fast recovery).

## Acceptance

`make test-tcp-fastrexmit-v1`: the boot self-test sends a segment, feeds two
duplicate ACKs (no retransmit), then the third — confirming an immediate
retransmit occurred (`rt_last_send_ok`, not via an RTO timeout) and fast recovery
set `ssthresh = 4096` and `cwnd = 4096 + 3·512` — `TCP: fast rexmit ok`,
alongside the still-green `TCP: rto ok` / `TCP: cc ok`.

## v1 boundary / carry-forward

- Single outstanding segment (inherited), so "three dup ACKs" is the trigger but
  there is one segment to resend; SACK-based selective retransmit and the full
  fast-recovery window inflation/deflation over a multi-segment window are
  carry-forward.
- No limited-transmit (RFC 3042) or PRR.
