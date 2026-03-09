# Socket Contract v2

Date: 2026-03-08  
Milestone: M19 Network Stack v2

## Scope

Define deterministic socket and DNS-stub behavior for the M19 lane.

## Supported socket surface

- Domains:
  - `AF_INET`
  - `AF_INET6`
- Types:
  - `SOCK_STREAM` (TCP interop profile v2)
  - `SOCK_DGRAM` (UDP parity and service diagnostics)
- Lifecycle calls:
  - `socket`, `bind`, `listen`, `connect`, `accept`, `shutdown`, `close`
- Data calls:
  - `send`, `recv`, `sendto`, `recvfrom`
- Readiness:
  - `POLLIN`, `POLLOUT`, `POLLERR`, `POLLHUP`

## Blocking and non-blocking semantics

- Blocking operations may wait for readiness but must return deterministic
  timeout-class outcomes when timeout bounds are reached.
- Non-blocking operations return immediately when not ready.
- Unsupported options/extensions must fail deterministically as unsupported.

## DNS stub behavior

- Query classes in v2 baseline:
  - `A`
  - `AAAA`
- Required behavior:
  - deterministic answers for configured service names,
  - deterministic `NXDOMAIN` for unknown names,
  - TTL-bounded cache behavior with explicit expiry.
- Resolver fallback:
  - when no `AAAA` answer exists, dual-stack fallback must choose IPv4.

## Error model

M19 follows deterministic error classes from `docs/abi/syscall_v1.md`:

- `E_INVAL`, `E_RANGE`, `E_FAULT`, `E_AGAIN`, `E_TIMEOUT`, `E_UNSUP`, `E_IO`

## Evidence

- Contract tests:
  - `tests/net/test_tcp_interop_v2.py`
  - `tests/net/test_ipv6_interop_v2.py`
  - `tests/net/test_dns_stub_v2.py`
- Related baseline docs:
  - `docs/net/network_stack_contract_v2.md`
  - `docs/net/tcp_profile_v2.md`
  - `docs/net/interop_matrix_v2.md`
