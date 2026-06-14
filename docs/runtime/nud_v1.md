# IPv6 neighbor cache + NUD (guest-initiated) — contract v1

Status: boot-verified via `make test-nud-v1`
Source: `kernel_rs/src/netcfg.rs` (`build_neighbor_solicit`, `nud_ingest_advert`,
`nud_lookup`, `nud_find`, `NEIGH_CACHE`, `nud_selftest`); dispatch
`sys_net_query` (id 49) op 14.
Proof: `tests/runtime/test_nud_v1.py`.

Full-OS guide Part II.6 (networking maturity), IPv6: the guest **initiating**
neighbor resolution and maintaining a neighbor cache — the carry-forward beyond
the NDP *responder* ([`ndp_v1.md`](ndp_v1.md), which only answers a host's
solicitation). RFC 4861 Neighbor Unreachability Detection.

## Behaviour

`NEIGH_CACHE` (8 entries `{ip6, mac, state}`, states `INCOMPLETE`/`REACHABLE`):

- **`build_neighbor_solicit(target)`**: the guest sends its own Neighbor
  Solicitation — Ethernet dst = the target's solicited-node multicast MAC
  (`33:33:ff:XX:XX:XX`), IPv6 dst = `ff02::1:ffXX:XXXX`, src = the guest, hop
  limit 255, with a Source Link-Layer Address option carrying the guest MAC and a
  correct ICMPv6 checksum. Records an `INCOMPLETE` cache entry awaiting the reply.
- **`nud_ingest_advert(frame)`**: on a received Neighbor Advertisement (type 136),
  learns the advertiser's MAC from its Target Link-Layer Address option and marks
  the neighbor `REACHABLE`.
- **`nud_lookup(target)`**: returns the cached MAC once `REACHABLE`.

## Acceptance

`make test-nud-v1`: the boot self-test builds an NS for a target (lookup misses
while `INCOMPLETE`), verifies the NS is wire-correct (solicited-node multicast
dst, guest src, type 135, target field, SLLA = guest MAC, checksum folds to zero),
then ingests a matching NA and confirms the lookup resolves to the advertised MAC
(`REACHABLE`) — `NUD: resolve ok`. The NDP responder ([`ndp_v1.md`](ndp_v1.md))
stays green.

## v1 boundary / carry-forward

- States `INCOMPLETE`/`REACHABLE` only; the full NUD state machine
  (`STALE`/`DELAY`/`PROBE` with reachability timers and retransmitted probes) and
  retransmission of the solicitation are carry-forward.
- Wiring the cache into the IPv6 transmit path (resolve-before-send, queue pending
  packets) is carry-forward; this slice proves the solicit + cache + ingest cycle.
