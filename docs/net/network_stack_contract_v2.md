# Network Stack Contract v2

Date: 2026-03-08  
Milestone: M19 Network Stack v2  
Status: active release gate

## Purpose

Define the M19 network baseline as a versioned, executable contract with
stronger interop and soak evidence than v1.

## Scope (v2 lane)

- L3:
  - IPv4 baseline compatibility retained.
  - IPv6 ND + ICMPv6 interop baseline promoted to v2 coverage.
- L4:
  - TCP interop profile with deterministic retry and close behavior.
  - UDP baseline retained for diagnostic parity.
- Service behavior:
  - deterministic DNS-stub behavior for `A` and `AAAA` lookups.
  - deterministic negative-path behavior (`NXDOMAIN`, expired cache entries).
- Diagnostics:
  - interop matrix and soak reports are required release artifacts.

## ABI boundary

- Raw frame syscalls remain in ABI:
  - `sys_net_send` (`docs/abi/syscall_v1.md` #15)
  - `sys_net_recv` (`docs/abi/syscall_v1.md` #16)
- Socket/DNS semantics for M19 are versioned in:
  - `docs/net/socket_contract_v2.md`
  - `docs/net/tcp_profile_v2.md`

## Deterministic behavior requirements

- TCP peer interop scenarios must have explicit pass/fail outcomes.
- IPv6 ND/ICMPv6 behavior must be cache/timeout deterministic.
- DNS-stub queries must provide deterministic answers and TTL behavior.
- Interop/soak outputs must be machine-readable and schema-versioned.

## Required release gates

- Local gate: `make test-network-stack-v2`
- CI gate: `Network stack v2 gate`
- Test suite:
  - `tests/net/test_tcp_interop_v2.py`
  - `tests/net/test_ipv6_interop_v2.py`
  - `tests/net/test_dns_stub_v2.py`
  - `tests/net/test_network_gate_v2.py`

## Required artifacts

- `out/net-interop-v2.json`
- `out/net-soak-v2.json`
