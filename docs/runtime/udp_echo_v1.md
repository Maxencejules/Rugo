# UDP echo responder — contract v1

Status: boot-verified via `make test-udp-echo-v1`
Source: `kernel_rs/src/netcfg.rs` (`build_udp_echo_reply`, `udp_echo_input`,
`udp_echo_selftest`), `kernel_rs/src/net.rs` (`net_rx_pump` UDP dispatch),
`kernel_rs/src/lib.rs` (boot self-test, `sys_net_query` op 8).
Proof: `tests/runtime/test_udp_echo_v1.py`.

Full-OS implementation guide Part II.6 (networking maturity), UDP-server slice.
Together with the ARP, ICMP, ICMPv6, and TCP-accept responders, the guest is a
comprehensively reachable host (resolvable, pingable over v4/v6, and answering
both TCP connections and UDP datagrams).

## Behaviour

`build_udp_echo_reply` takes a received Ethernet frame and, if it is an
IPv4/UDP datagram addressed to the guest on the **echo port (7)**, builds the
reply: MACs swapped, IPv4 src/dst swapped, UDP src/dst ports swapped, payload
copied verbatim, IPv4 header checksum recomputed (the UDP checksum is left 0,
which IPv4 permits). `udp_echo_input` is the live path — `net_rx_pump` calls it
for every IPv4/UDP frame (alongside the DHCP/DNS client `udp_input`); it acts
only on port-7 datagrams and `wire_send`s the reply (`UDP: echo sent`).

`udp_echo_selftest` (boot self-test, also `sys_net_query` op 8) synthesizes a
UDP datagram to port 7, runs the responder, and verifies the endpoints are
swapped (reply source port = 7), the payload is echoed, and the reply's IPv4
header checksum folds to zero — emitting `UDP: echo ok`.

## v1 boundary / carry-forward

- Fixed echo port (7), single datagram, no UDP checksum on the reply (0 =
  unused). No general UDP socket API (`bind`/`recvfrom`/`sendto`) — this is a
  fixed-purpose responder, like the ICMP/ARP responders.

## Acceptance

`make test-udp-echo-v1`: the boot self-test emits `UDP: echo ok`, proving the
responder swaps endpoints, echoes the payload, and produces a checksum-correct
IPv4 reply.
