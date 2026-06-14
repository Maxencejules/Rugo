# ARP responder — contract v1

Status: boot-verified via `make test-arp-v1`
Source: `kernel_rs/src/netcfg.rs` (`build_arp_reply`, `arp_input`,
`arp_selftest`), `kernel_rs/src/net.rs` (`net_rx_pump` ARP opcode-1 dispatch),
`kernel_rs/src/lib.rs` (boot self-test, `sys_net_query` op 5).
Proof: `tests/runtime/test_arp_v1.py`.

Full-OS implementation guide Part II.6 (networking maturity), ARP slice — the
guest answers "who-has GUEST_IP" requests, so hosts on the link can resolve and
reach it. Pairs with the ICMP responder ([`icmp_v1.md`](icmp_v1.md)): together
the guest is a properly reachable IPv4 host (resolvable + pingable).

## Behaviour

`build_arp_reply(req, out)` takes a received Ethernet frame and, if it is an ARP
**request (opcode 1)** whose target protocol address is the guest IP
(`10.0.2.15`), writes a 42-byte ARP **reply (opcode 2)**: sender = our MAC and
the guest IP, target = the requester's MAC/IP, Ethernet dst = requester,
src = our MAC.

`arp_input(frame)` is the live path: `net_rx_pump` dispatches ARP frames with
opcode 1 here; it builds the reply and transmits it (`wire_send`), emitting
`ARP: reply sent`. Opcode-2 (replies) continue to feed the DHCP/DNS/TCP
neighbor caches as before.

`arp_selftest()` (boot self-test, also `sys_net_query` op 5) synthesizes a
who-has request for the guest IP, runs `build_arp_reply`, and verifies the
reply's opcode, sender MAC/IP, and target fields, emitting `ARP: reply ok`.

## v1 boundary / carry-forward

- Replies only for the single guest IP. No gratuitous ARP, no probe/announce,
  and no ARP cache aging for our own entries.
- The neighbor cache populated from replies is still per-consumer
  (DHCP/DNS/TCP); a unified ARP table is carry-forward.

## Acceptance

`make test-arp-v1`: the boot self-test emits `ARP: reply ok`, proving the
responder builds a correct reply (opcode 2, sender = our MAC/guest IP, target =
the requester) for a who-has-GUEST_IP request.
