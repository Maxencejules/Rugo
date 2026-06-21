# IPv6 / ICMPv6 echo responder — contract v1

Status: boot-verified via `make test-icmpv6-v1`
Source: `kernel_rs/src/netcfg.rs` (`guest_ip6`, `icmpv6_checksum`,
`build_icmpv6_echo_reply`, `icmpv6_input`, `icmpv6_selftest`),
`kernel_rs/src/net.rs` (`net_rx_pump` ethertype-0x86DD dispatch),
`kernel_rs/src/lib.rs` (boot self-test, `sys_net_query` op 7).
Proof: `tests/runtime/test_icmpv6_v1.py`.

Full-OS implementation guide Part II.6 (networking maturity), IPv6 slice — the
first IPv6 datagram handling: answer ICMPv6 echo requests (ping6) on the guest's
link-local address, making it a dual-stack reachable host.

## Behaviour

The guest's IPv6 address is the **link-local** `fe80::/64` formed from the NIC
MAC via EUI-64 (`guest_ip6`). `build_icmpv6_echo_reply` takes a received
Ethernet frame and, if it is an IPv6 (`0x86DD`) packet with next-header 58
(ICMPv6), ICMPv6 **type 128 (echo request)**, addressed to `guest_ip6`, builds
the **type 129 (echo reply)**: MACs swapped, IPv6 src/dst swapped (src =
guest_ip6), hop limit 255, and the **mandatory ICMPv6 checksum** recomputed over
the IPv6 pseudo-header (src + dst + upper-layer length + next-header) plus the
message.

`icmpv6_input` is the live path: `net_rx_pump` dispatches `0x86DD` frames here;
it builds and `wire_send`s the reply (`ICMPV6: echo reply sent`).
`icmpv6_selftest` (boot self-test, also `sys_net_query` op 7) synthesizes a
request to `guest_ip6`, runs the responder, and verifies type 129, the echoed
payload, and that the reply's ICMPv6 checksum folds to zero — emitting
`ICMPV6: echo reply ok`.

## v1 boundary / carry-forward

- **Echo only**, link-local address only. **No Neighbor Discovery (NDP)** —
  the guest does not answer Neighbor Solicitations, so a host cannot resolve the
  guest's IPv6 MAC to actually deliver a real ping6 yet (the self-test proves
  the responder; live NDP + a global/SLAAC address are carry-forward).
- No ICMPv6 errors, no UDP/TCP over IPv6, no fragmentation, no extension
  headers (next-header must be 58 directly).

## Acceptance

`make test-icmpv6-v1`: the boot self-test emits `ICMPV6: echo reply ok`,
proving the responder builds a checksum-correct type-129 reply (with the IPv6
pseudo-header checksum) that echoes the request's payload.
