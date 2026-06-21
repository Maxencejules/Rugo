# ICMP echo responder — contract v1

Status: boot-verified via `make test-icmp-v1`
Source: `kernel_rs/src/netcfg.rs` (`build_icmp_echo_reply`, `icmp_input`,
`icmp_selftest`), `kernel_rs/src/net.rs` (`net_rx_pump` proto-1 dispatch),
`kernel_rs/src/lib.rs` (go_test boot self-test call, `sys_net_query` op 4).
Proof: `tests/runtime/test_icmp_v1.py`.

Full-OS implementation guide Part II.6 (networking maturity), ICMP slice —
the guest answers ICMP echo requests (ping), making it a reachable host.

## Behaviour

`build_icmp_echo_reply(req, out)` takes a full received Ethernet frame and, if
it is a well-formed IPv4/ICMP **echo request (type 8)** addressed to the guest
IP (`10.0.2.15`), writes the corresponding **echo reply (type 0)** into `out`:

- Ethernet: destination = the requester's source MAC, source = our MAC.
- IPv4: source/destination swapped, TTL reset to 64, header checksum recomputed.
- ICMP: type set to 0, checksum recomputed over the whole ICMP message
  (ident, sequence, and payload are preserved verbatim).

`icmp_input(frame)` is the live path: the RX pump (`net_rx_pump`) dispatches
IPv4 frames with `proto == 1` here; it builds the reply and transmits it with
`wire_send`, emitting `ICMP: echo reply sent`. This answers real inbound pings.

`icmp_selftest()` is the deterministic acceptance path (no external responder):
it synthesizes an echo request (ident `0x5247`, seq 1, payload `rugoping`),
runs `build_icmp_echo_reply`, and verifies the reply is type 0, echoes the
ident/seq/payload, and that both the IPv4 header checksum and the ICMP checksum
fold to zero (i.e. are wire-correct). On success it emits
`ICMP: echo reply ok seq=0x0000000000000001` and returns 1. It runs once at
go_test boot (after NIC init) and is also reachable via `sys_net_query` op 4.

## v1 boundary / carry-forward

- Echo (ping) only. No ICMP error generation (destination/port unreachable,
  time exceeded), no ICMP rate limiting, and no outbound ping client. The
  responder ignores fragmented requests (single-frame echo only).
- IPv4 only; ICMPv6 is carry-forward with the rest of IPv6.
- The self-test validates reply construction; an over-the-wire round trip
  through slirp is not asserted (host ICMP support is not guaranteed).

## Acceptance

`make test-icmp-v1`: the boot self-test emits
`ICMP: echo reply ok seq=0x0000000000000001`, proving the responder builds a
checksum-correct echo reply that preserves the request's ident/seq/payload.
