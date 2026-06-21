# TCP passive open (listener) — contract v1

Status: boot-verified via `make test-tcp-listen-v1`
Source: `kernel_rs/src/tcp.rs` (`ST_LISTEN`, `ST_SYN_RCVD`, the listener arms in
`tcp_input`, `tcp_listen_selftest`, `build_seg`), `kernel_rs/src/lib.rs`
(boot self-test, `sys_net_query` op 6).
Proof: `tests/runtime/test_tcp_listen_v1.py`.

Full-OS implementation guide Part II.6 (networking maturity), TCP listener
slice — the server side of the handshake (passive open), complementing the
existing client (active open) in [`netcfg_v1.md`](netcfg_v1.md)/tcp.rs.

## Behaviour

Two states are added to the connection state machine:

- **ST_LISTEN**: on a bare **SYN** (SYN set, ACK clear), the connection records
  the client's port and `rcv_nxt = seq+1`, sends a **SYN|ACK** (its ISN in
  `snd_nxt`, then `snd_nxt += 1`), and moves to **ST_SYN_RCVD**
  (`TCP: syn-rcvd`).
- **ST_SYN_RCVD**: the client's **ACK** completes the three-way handshake →
  **ST_ESTABLISHED** (`TCP: established`).

These arms live in the live `tcp_input`, so the guest can accept an inbound
connection. `tcp_listen_selftest()` (boot self-test, also `sys_net_query`
op 6) binds a listener to :8080 with a synthetic peer, feeds a SYN then an ACK
(via `build_seg`; `tcp_input` validates no checksums), asserts the state reached
ESTABLISHED, emits `TCP: listen ok`, and resets the connection so the outbound
client path (`tcp_connect`) stays usable.

## v1 boundary / carry-forward

- **One connection, single pre-bound peer.** The self-test pre-sets the peer;
  a real `listen()`/`accept()` that accepts from *any* client, a multi-
  connection table, and a backlog queue are carry-forward.
- No retransmission/RTO, no window management, no delayed/again ACK tuning
  (QEMU user-net is loss-free) — same boundary as the client path.
- No userspace socket API yet (no `listen`/`accept` syscalls); this slice
  proves the in-kernel passive-open state machine.

## Acceptance

`make test-tcp-listen-v1`: the boot self-test emits `TCP: syn-rcvd`,
`TCP: established`, and `TCP: listen ok`, proving SYN → SYN|ACK → ACK →
ESTABLISHED. The existing client test (`test_tcp_runtime_v1`) still passes,
confirming the self-test leaves the connection CLOSED.
