# TCP retransmission / RTO — contract v1

Status: boot-verified via `make test-tcp-rto-v1` (go C4 runtime lane)
Source: `kernel_rs/src/tcp.rs` (`tcp_rt_arm`, `tcp_rt_ack`, `tcp_rt_tick`,
`tcp_rto_selftest`, the `snd_una` + retransmit slot in `TcpConn`), live tick in
`kernel_rs/src/lib.rs` (`r4_timer_preempt` → `tcp::tcp_rt_tick`).
ABI: `sys_net_query` (id 49) op 10.
Proof: `tests/runtime/test_tcp_rto_v1.py`.

Full-OS implementation guide Part II.6 (Networking), TCP reliability — the
retransmission timer that makes the wire-TCP client survive loss. The original
TCP slice noted "no retransmission (QEMU's user-mode network is loss-free)" as a
carry-forward; this closes it.

## Behaviour

The connection keeps a single **retransmit slot** holding the oldest
unacknowledged segment (its flags, sequence, and bytes) plus `snd_una` (oldest
unacked sequence):

- **Arm on send.** `tcp_connect`/`on_arp_reply` (SYN), `tcp_send` (data), and
  `tcp_close` (FIN) record the segment via `tcp_rt_arm` and start the RTO
  countdown (`TCP_RTO_TICKS` = 50 PIT ticks ≈ 500 ms). `tcp_send` refuses a new
  send while a segment is still outstanding (one in flight at a time, v1).
- **Clear on ACK.** `tcp_input` parses the ACK field on every segment carrying
  the ACK flag and calls `tcp_rt_ack`: when the peer's cumulative ACK covers the
  whole outstanding segment (`ack - rt_seq >= span`, wrapping-safe, where `span`
  counts payload + 1 per SYN/FIN), the timer clears and `snd_una` advances. A
  stale/duplicate ACK leaves the timer armed.
- **Retransmit on timeout.** `tcp_rt_tick`, called once per PIT tick while a
  connection is active (after the RX pump, so an inbound ACK is processed
  first), decrements the countdown; at zero it re-sends the stored segment,
  increments the retry count, and backs off exponentially (RTO × 2^min(retries,4)).
- **Give up.** After `TCP_MAX_RETRIES` (5) the peer is unreachable: the
  connection sends an **RST** (RFC 1122 §4.2.3.5, so the peer releases its half
  rather than waiting for its own timers), logs `TCP: rto giveup`, and resets all
  connection state.
- **Deferred close.** `tcp_close` while a data segment is still unacknowledged
  does **not** send the FIN immediately — that would clobber the single
  retransmit slot and lose the data on loss. It sets a pending-close flag; the
  FIN is sent (and armed) once the data's ACK clears the slot.

## ABI

`sys_net_query(op=10)` runs `tcp_rto_selftest` (requires the `NETWORK`
capability) and returns 1/0. It also runs at boot, emitting `TCP: rto ok`.

## v1 boundary / carry-forward

- **One outstanding segment.** A single retransmit slot, not a full send buffer /
  sliding window; `tcp_send` blocks a second send until the first is acked.
- **Fixed RTO, no RTT estimation.** A constant base RTO with exponential
  backoff; no Jacobson/Karels SRTT/RTTVAR estimation, no Karn's algorithm, no
  fast retransmit (3-dup-ACK) and no congestion control (slow start / cwnd).
- **Loss-free live path.** QEMU user-net never drops, so the live timeout path
  is not exercised on the wire; the self-test drives the tick countdown directly
  to prove retransmit-then-clear deterministically.

## Acceptance

`make test-tcp-rto-v1`: the go lane boots, the transcript shows `TCP: rto ok`
(the self-test sent a segment, observed exactly one retransmission after the RTO
elapsed with no ACK, then saw the timer clear and `snd_una` advance on the ACK,
with no further retransmit), and never `TCP: rto giveup`; then reaches
`GOINIT: result shutdown-clean` and `RUGO: halt ok`.
