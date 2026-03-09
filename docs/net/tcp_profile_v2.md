# TCP Profile v2

Date: 2026-03-08  
Milestone: M19 Network Stack v2

## Purpose

Freeze a deterministic TCP interop profile for common peers and mixed workload
network behavior in M19.

## Peer interop baseline

- Reference peers:
  - `linux-6.8`
  - `freebsd-14.1`
  - `windows-2025`
- Required scenarios:
  - three-way handshake
  - 1 MiB data transfer with bounded retransmission
  - half-close and full-close path
  - reconnect after reset

## Negotiation baseline

- Minimum negotiated MSS: `1220`
- Window scaling: required
- Timestamps: required
- Unsupported negotiation extensions in this milestone must fail
  deterministically.

## Retry and timeout policy

- Initial retry interval baseline: deterministic and bounded.
- Max retries before timeout: deterministic and bounded per scenario.
- Interop scenarios with packet loss up to `2.5%` must remain passable with
  bounded retries.

## Interop target

- Interop pass target: `>= 0.95` (`tools/run_net_interop_matrix_v2.py`)
- Soak max failure target: `0` (`tools/run_net_soak_v2.py`)

## Evidence

- `tests/net/test_tcp_interop_v2.py`
- `tests/net/test_network_gate_v2.py`
- `docs/net/interop_matrix_v2.md`
