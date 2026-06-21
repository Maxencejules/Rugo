# IPv6 Neighbor Discovery (NDP) — contract v1

Status: boot-verified via `make test-ndp-v1` + `make test-ndp-dad-v1` (go C4 lane)
Source: `kernel_rs/src/netcfg.rs` (`build_neighbor_advert`, `ndp_selftest`,
`build_dad_solicit`, `dad_selftest`, the NUD neighbor cache, `icmpv6_input` type
dispatch), live RX via `net::net_rx_pump` (ethertype 0x86DD).
ABI: `sys_net_query` (id 49) op 9 (responder), op 14 (NUD), op 20 (DAD).
Proof: `tests/runtime/test_ndp_v1.py`, `tests/runtime/test_ndp_dad_v1.py`.

Full-OS implementation guide Part II.6 (Networking), IPv6 Neighbor Discovery —
the piece that makes the guest's link-local IPv6 actually *resolvable*. ICMPv6
echo (`icmpv6_v1.md`) lets a host ping the guest once it knows the guest's MAC;
NDP is how the host learns that MAC in the first place.

## Behaviour

When a host wants to send to the guest's link-local address it issues a
**Neighbor Solicitation** (ICMPv6 type 135) to the target's solicited-node
multicast address (`ff02::1:ffXX:XXXX`, MAC `33:33:ff:XX:XX:XX`), carrying the
target address and usually a Source Link-Layer Address option.

`icmpv6_input` dispatches on the ICMPv6 type: type 128 → echo reply (existing),
type 135 → `build_neighbor_advert`. The advertisement (`build_neighbor_advert`):

- validates the frame is ICMPv6 (next header 58), type 135, and the NS **target
  address equals the guest's link-local address** (EUI-64 from the NIC MAC) —
  otherwise it is for some other node and is ignored;
- builds a **Neighbor Advertisement** (type 136) sent unicast back to the
  soliciting host (its MAC + source IPv6), hop limit 255;
- sets the **Solicited (S)** and **Override (O)** flags (`0x60`);
- sets the **Target Address** to the guest's address;
- appends a **Target Link-Layer Address** option (type 2, length 1) carrying the
  guest MAC — the answer the host was after;
- fills the ICMPv6 checksum over the IPv6 pseudo-header.

## ABI

`sys_net_query(op=9)` runs `ndp_selftest` and returns 1 on success / 0 on fail
(requires the `NETWORK` capability, like the other net ops). It also runs at
boot in the go lane, emitting `NDP: advert ok`.

## Guest-initiated DAD (`build_dad_solicit` / `dad_selftest`, op 20)

The guest also runs **Duplicate Address Detection** on its OWN address
(RFC 4862 §5.4 / RFC 4861 §4.3) before relying on it. `build_dad_solicit` builds a
Neighbor Solicitation **from the unspecified source (`::`)** for the guest's
tentative link-local target, to that target's solicited-node multicast, with **no
Source Link-Layer Address option** (mandatory when the source is `::`), hop limit
255, and an ICMPv6 pseudo-header checksum that folds to zero. `dad_selftest` (run at
C4 boot and via op 20) validates every field against known-correct values, then
transmits the probe. No host defends the guest's `fe80::` over the slirp link, so
the address is unique. Marker `NDP: dad probe ok`. (`build_neighbor_advert` already
handles the *defense* side — answering another host's DAD probe to the all-nodes
multicast with Solicited cleared.)

## DAD defense (RFC 4861 §7.2.4)

A Neighbor Solicitation whose IPv6 source is `::` — a remote host's DAD probe of the
guest's address — is answered to the all-nodes multicast (`ff02::1`, MAC
`33:33:00:00:00:01`) with the **Solicited flag cleared** (Override kept), rather than
unicast to `::` (a malformed, undeliverable packet). Normal resolution NS (real
link-local source) get a unicast Solicited+Override NA. Both paths are exercised by
`ndp_selftest` (op 9).

## v1 boundary / carry-forward

- **Resolution (NS↔NA), NUD (neighbor cache + guest-initiated solicit, op 14),
  guest DAD (op 20), and DAD defense are all implemented**, plus SLAAC / Router
  Solicitation (`slaac_v1.md`, op 15). What remains: NDP Redirect handling, neighbor
  cache reachability *timers* (the cache tracks INCOMPLETE/REACHABLE but does not yet
  age entries back to STALE/PROBE), and acting on a DAD *defense* received for the
  guest's own tentative address (today DAD is verified unique over slirp; a real
  collision response — abandoning the address — is carry-forward).

## Acceptance

`make test-ndp-v1`: the go lane boots, the transcript shows `NDP: advert ok`
(the self-test built a type-136 advertisement with S+O flags, the guest target,
the guest-MAC TLLA option, and a checksum that folds to zero), then reaches
`GOINIT: result shutdown-clean` and `RUGO: halt ok`.
