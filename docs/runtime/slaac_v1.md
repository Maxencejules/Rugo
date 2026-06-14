# IPv6 SLAAC (stateless address autoconfiguration) — contract v1

Status: boot-verified via `make test-slaac-v1`
Source: `kernel_rs/src/netcfg.rs` (`build_router_solicit`, `slaac_derive_global`,
`slaac_selftest`); dispatch `sys_net_query` (id 49) op 15.
Proof: `tests/runtime/test_slaac_v1.py`.

Full-OS guide Part II.6 (networking maturity), IPv6 SLAAC / Router Discovery: the
guest configures a routable global IPv6 address from a router's advertisement —
the next step beyond link-local + NDP ([`ndp_v1.md`](ndp_v1.md),
[`nud_v1.md`](nud_v1.md)).

## Behaviour

- **`build_router_solicit`**: the guest sends a Router Solicitation (ICMPv6
  type 133) to the all-routers multicast (`ff02::2` / `33:33:00:00:00:02`),
  sourced from its link-local address with a Source Link-Layer Address option,
  hop limit 255, and a correct ICMPv6 checksum.
- **`slaac_derive_global`**: from a received Router Advertisement (type 134), walk
  the options for the Prefix Information option (type 3), take its `/64` prefix,
  and form the global address = prefix (high 64 bits) + the interface **EUI-64**
  (the low 64 bits of the guest link-local address).

## Acceptance

`make test-slaac-v1`: the boot self-test builds an RS (verified wire-correct:
all-routers multicast dst, guest src, SLLA option, hop limit 255, checksum folds
to zero), then processes a synthetic RA carrying a `2001:db8::/64` prefix and
derives the global address (prefix + EUI-64), confirming the high 64 bits equal
the prefix and the low 64 bits equal the guest EUI-64 — `SLAAC: global ok`.

## v1 boundary / carry-forward

- `/64` prefix only (the SLAAC common case); other prefix lengths, multiple
  prefixes, and the M/O flags (stateful DHCPv6) are carry-forward.
- Derives + verifies the address; **assigning** it to the interface, running DAD
  on it (NS from `::`, already handled inbound by the NDP responder), honoring
  valid/preferred lifetimes, and installing the router as a default route are
  carry-forward.
