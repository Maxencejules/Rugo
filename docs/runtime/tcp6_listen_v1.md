# IPv6 TCP passive open (IPv6 on the wire) — contract v1

Status: boot-verified via `make test-tcp6-listen-v1`
Source: `kernel_rs/src/netcfg.rs` (`Tcp6Conn`, `tcp6_input`, `tcp6_tx`,
`tcp6_checksum`, `build_tcp6_seg`, `tcp6_listen_selftest`); dispatch
`sys_net_query` (id 49) op 18.
Proof: `tests/runtime/test_tcp6_listen_v1.py`.

Full-OS guide Part II.6 (networking maturity), IPv6 on the wire — TCP: a
passive-open (listener) three-way handshake over IPv6, the transport counterpart
to the IPv6 UDP echo ([`udp6_echo_v1.md`](udp6_echo_v1.md)) and the IPv6 control
plane (ICMPv6/NDP/NUD/SLAAC), mirroring the IPv4 listener
([`tcp_listen_v1.md`](tcp_listen_v1.md)).

## Behaviour

A minimal IPv6 TCP connection (`Tcp6Conn`) handshake responder:

- **`tcp6_input`**: for a segment addressed to the guest's link-local address
  (next header 6), `LISTEN` + a bare SYN → reply `SYN|ACK` (`tcp6_tx`) and enter
  `SYN_RCVD`; `SYN_RCVD` + the client ACK → `ESTABLISHED`.
- **`tcp6_tx` / `tcp6_checksum`**: build a 20-byte IPv6 TCP segment (hop limit
  255) with the **mandatory** TCP checksum over the IPv6 pseudo-header
  (src+dst+length+next-header 6) + the segment.

## Acceptance

`make test-tcp6-listen-v1`: the boot self-test binds a listener on `:8080`, feeds
a SYN (→ `TCP6: syn-rcvd` + a SYN|ACK whose checksum folds to zero over the reply
addresses), then the client ACK (→ `TCP6: established`), and reports `TCP6: listen
ok` — alongside the still-green IPv4 `TCP: listen ok`. The connection state is
reset afterward so nothing leaks.

## v1 boundary / carry-forward

- **Handshake only** (LISTEN → SYN_RCVD → ESTABLISHED). Data transfer, teardown
  (FIN/RST), retransmission/RTO, and a multi-connection IPv6 socket table are
  carry-forward (the IPv4 path has the data/teardown/RTO machinery; unifying them
  over a shared connection table is the larger refactor).
- A userspace **bind**/listen syscall for IPv6 (so a live inbound SYN is
  answered, not just the boot self-test) is carry-forward; `tcp6_input` is the
  responder it would drive.
