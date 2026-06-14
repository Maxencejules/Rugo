# Networking maturity — DHCP DORA — contract v2

Status: boot-verified via `make test-dhcp-dora-v1`
Source: `kernel_rs/src/netcfg.rs` (`Q_DHCP_REQ` state, OFFER→REQUEST,
ACK handling).
Proof: `tests/runtime/test_dhcp_dora_v1.py` (and the unchanged
`tests/runtime/test_netcfg_runtime_v1.py`).

Full-OS implementation guide Part II.6 (networking maturity), the DHCP
slice. Extends the v1 DISCOVER→OFFER client ([`netcfg_v1.md`](netcfg_v1.md))
to the full **DORA** exchange.

## Flow

1. **Discover** — `start_dhcp` broadcasts DISCOVER (unchanged).
2. **Offer** — on the first BOOTREPLY with our xid, the kernel records the
   offered IP, emits `DHCP: offer ip=0x<ip>`, and immediately broadcasts a
   **Request** (option 53 = 3) echoing the offered address (option 50) and
   the server id (option 54 = the slirp gateway `10.0.2.2`), then enters
   `Q_DHCP_REQ` and emits `DHCP: request sent`.
3. **Ack** — on the next BOOTREPLY with our xid, the kernel confirms the
   lease, emits `DHCP: ack ip=0x<ip>`, and completes (`Q_DONE`).

The state machine sequences OFFER vs ACK by state, not by parsing option 53
(the slirp exchange is deterministic). The poll path (`sys_net_query` op 3)
returns the leased address once `Q_DONE`.

## v1 boundary / carry-forward

- No lease-time parsing or renewal timer (T1/T2 refresh) yet — carry-forward.
- Server id is the known slirp gateway rather than parsed from option 54.
- The rest of networking maturity (TCP retransmit/RTO, wire listen/accept,
  ICMP echo, routing, IPv6) remains carry-forward.

## Acceptance

`make test-dhcp-dora-v1`: `dhcpcheck` drives DISCOVER→OFFER→REQUEST→ACK and
the markers `DHCP: offer`, `DHCP: request sent`, `DHCP: ack` all appear with
the canonical `10.0.2.15` lease, then `NETD: dhcp ok`. The v1 netcfg test
(`test_netcfg_runtime_v1`) still passes unchanged.
