# DHCP + DNS Client Contract v1

Status: live runtime (boot-verified)
Source: `kernel_rs/src/netcfg.rs`, UDP routing in `kernel_rs/src/net.rs`,
shell commands in `services/go/netcfgcheck.go`
Proof: `make test-netcfg-v1`, `tests/runtime/test_netcfg_runtime_v1.py`

Completes gap-analysis build-list item 6: alongside wire TCP, the
network-configuration clients the item names — DHCP and DNS — send real
UDP over the VirtIO NIC.

## Syscall surface

`sys_net_query` (id 49, additive v3.2 window; requires the network
capability):

- op 1: DHCP DISCOVER (broadcast from 0.0.0.0:68 to 255.255.255.255:67)
- op 2: DNS A query — a2 = name pointer, a3 = `len | port << 16`;
  port 53 targets the slirp resolver (10.0.2.3), any other port targets
  the gateway (10.0.2.2, the host side, where the acceptance test runs
  its own resolver)
- op 3: poll — `u64::MAX` while pending, then the parsed IPv4 once

One query outstanding at a time; the PIT-tick RX pump plus an
opportunistic pump in op 3 drive reception. UDP checksums are 0
(permitted on IPv4).

## Mechanics

- DHCP: BOOTP DISCOVER with the broadcast flag and option 53; the first
  BOOTREPLY matching the transaction id yields `yiaddr`
  (`DHCP: offer ip=0x<addr>`). QEMU's built-in DHCP server answers with
  10.0.2.15 — fully offline. Full DORA (REQUEST/ACK) and lease
  plumbing are the carry-forward.
- DNS: one A/IN question (dotted name split into labels); the answer
  parser handles compression pointers and takes the first A record
  (`DNS: a=0x<addr>`). The DNS path ARP-resolves its server first.

## Marker contract

| Marker | Meaning |
|---|---|
| `DHCP: offer ip=0x<addr>` | OFFER parsed; the offered IPv4 address |
| `DNS: a=0x<addr>` | A record parsed from the response |
| `NETD: dhcp ok` / `NETD: dhcp err` | shell `dhcpcheck` verdict |
| `NETD: dns ok` / `NETD: dns err` | shell `dnscheck <name> <port>` verdict |
