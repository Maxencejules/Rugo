# Wire TCP Contract v1

Status: live runtime (boot-verified)
Source: `kernel_rs/src/tcp.rs`, RX pump + socket diversion in
`kernel_rs/src/net.rs`, shell `tcpcheck` in `services/go/netcheck_tcp.go`
Proof: `make test-tcp-v1`, `tests/runtime/test_tcp_runtime_v1.py`

Begins closing gap-analysis build-list item 6 ("TCP — the single biggest
functional absence"): the default lane carries real TCP segments over the
VirtIO NIC. The acceptance test's payload round-trips between the guest
kernel and a host-side listener owned by pytest — the bytes only arrive if
a real handshake and real data segments cross QEMU's user-mode network.

## Mechanics

- `AF_INET` stream `sys_socket_connect` diverts to the wire TCP machine
  (the `AF_INET6` loopback rendezvous keeps its existing test surface).
- One outbound connection at a time: ARP-resolve the gateway, SYN →
  SYN/ACK → ACK, PSH/ACK data both ways with in-order receive buffering
  (1 KiB ring), FIN teardown, RST abort.
- The connection is driven by the PIT-tick RX pump (`net_rx_pump`, 8
  frames per tick while a connection is active) plus an opportunistic
  pump inside the socket syscalls; user code retries send/recv with
  yields until the handshake completes.
- Deterministic ephemeral port (`0xC000 | (dst_port & 0xFFF)`), fixed ISS;
  no retransmission (QEMU's user-mode network is loss-free).

## v1 carry-forward (documented, not implemented)

Retransmission/window management, listeners (accept from the wire),
multiple concurrent connections, DHCP and DNS clients (gap §3.6 names
both — they ride on this transport next), IPv6 on the wire.

## Marker contract

| Marker | Meaning |
|---|---|
| `TCP: syn sent` | SYN transmitted (after ARP resolution when needed) |
| `TCP: established` | SYN/ACK received and ACKed |
| `TCP: rst` | connection aborted by peer |
| `TCP: closed` | FIN teardown completed |
| `NETT: tcp ok` / `NETT: tcp err` | shell `tcpcheck <port>` round-trip result |
