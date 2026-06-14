# IP routing table (longest-prefix match) — contract v1

Status: boot-verified via `make test-routing-v1`
Source: `kernel_rs/src/net.rs` (`r4_net_find_route`, `r4_net_prefix_match`,
`R4_NET_ROUTES`, `route_selftest`); dispatch `sys_net_query` (id 49) op 13.
Proof: `tests/runtime/test_routing_v1.py`.

Full-OS guide Part II.6 (networking maturity), routing: select the egress route
for a destination by **longest-prefix match** over a routing table.

## Behaviour

`R4_NET_ROUTES` holds up to 8 routes `{family, prefix_len, if_index, dest}`.
`r4_net_find_route(family, dest)` scans for every route whose `prefix_len` bits of
`dest` match (`r4_net_prefix_match`, bit-masked per byte for IPv4/IPv6) and
returns the one with the **longest** prefix — so a `/24` wins over a `/8` which
wins over the `0.0.0.0/0` default. Routes are added via the network-config syscall
(capability-gated); this slice exercises the selection logic.

## Acceptance

`make test-routing-v1`: the boot self-test installs `0.0.0.0/0`, `10.0.0.0/8`,
and `10.0.2.0/24` (deliberately out of prefix order so success depends on
longest-match, not table position) and confirms `10.0.2.5 → /24`, `10.5.5.5 → /8`,
`8.8.8.8 → default` — `ROUTE: selftest ok`, with no `ROUTE: selftest fail`. The
live route table is saved and restored, so the test leaves no residue.

## v1 boundary / carry-forward

- Selection only (the table + longest-prefix match). Per-route next-hop gateway
  resolution via ARP/NDP before transmit, route metrics, and a userspace `route`
  tool are carry-forward.
- 8 routes, IPv4 + IPv6, on-link interface routes; no policy routing / multiple
  tables.
