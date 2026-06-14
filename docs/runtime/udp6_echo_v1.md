# IPv6 UDP echo (IPv6 on the wire) — contract v1

Status: boot-verified via `make test-udp6-echo-v1`
Source: `kernel_rs/src/netcfg.rs` (`build_udp6_echo_reply`, `udp6_checksum`,
`udp6_echo_input`, `udp6_echo_selftest`); live RX wiring in
`kernel_rs/src/net.rs` (`net_rx_pump`); dispatch `sys_net_query` (id 49) op 17.
Proof: `tests/runtime/test_udp6_echo_v1.py`.

Full-OS guide Part II.6 (networking maturity), IPv6 on the wire: a wire IPv6
**transport** responder (UDP), beyond the link-local control plane already done
(ICMPv6 echo, NDP, NUD, SLAAC) and mirroring the IPv4 UDP echo
([`udp_echo_v1.md`](udp_echo_v1.md)).

## Behaviour

`build_udp6_echo_reply` answers an IPv6 (`0x86DD`) UDP datagram (next header 17)
addressed to the guest's link-local address on the echo port (7): it swaps the
MACs, swaps the IPv6 source/destination (reply src = guest), swaps the UDP ports,
echoes the payload, sets hop limit 255, and computes the **mandatory** IPv6 UDP
checksum (`udp6_checksum`: over the IPv6 pseudo-header + UDP segment; a computed 0
is sent as `0xFFFF` per RFC 768/2460 — unlike IPv4, where the UDP checksum may be
0). `udp6_echo_input` runs it on the live RX pump alongside the ICMPv6 responder
(each checks its own next header).

## Acceptance

`make test-udp6-echo-v1`: the boot self-test synthesizes an IPv6/UDP datagram to
the guest:7, runs the responder, and confirms the reply swapped the endpoints
(src port now 7, src = guest / dst = sender), echoed the payload, and carries a
non-zero UDP checksum that recomputes to the transmitted value — `UDP6: echo ok`,
alongside the still-green IPv4 `UDP: echo ok` and `ICMPV6: echo reply ok`.

## v1 boundary / carry-forward

- UDP echo over IPv6 (one fixed echo service). Wire IPv6 **TCP** (mirroring the
  IPv4 client/listener), an IPv6 socket API to userspace, and fragmentation /
  extension-header handling are carry-forward.
- The guest answers on its link-local address; answering on a SLAAC-derived
  global address ([`slaac_v1.md`](slaac_v1.md)) once it is assigned to the
  interface is carry-forward.
