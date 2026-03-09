# Interop Matrix v2

Date: 2026-03-08  
Milestone: M19 Network Stack v2

## Purpose

Define v2 interop and soak report schemas, target thresholds, and scenario
coverage used by the network release gate.

## Interop report contract

- Tool: `tools/run_net_interop_matrix_v2.py`
- Output: `out/net-interop-v2.json`
- Schema: `rugo.net_interop_matrix.v2`
- Required fields:
  - `created_utc`
  - `total_cases`
  - `passed_cases`
  - `failed_cases`
  - `pass_rate`
  - `target_pass_rate`
  - `meets_target`
  - `cases[]` (`peer`, `scenario`, `transport`, `status`, `notes`)

## Soak report contract

- Tool: `tools/run_net_soak_v2.py`
- Output: `out/net-soak-v2.json`
- Schema: `rugo.net_soak_report.v2`
- Required fields:
  - `seed`
  - `iterations`
  - fault counters (`packet_loss`, `reorder`, `duplicate`, retry classes)
  - `total_failures`
  - `max_failures`
  - `meets_target`

## Required v2 scenarios

- TCP:
  - handshake
  - bulk transfer
  - reconnect after reset
- IPv6:
  - ND exchange
  - ICMPv6 echo payload parity
  - dual-stack preference/fallback
- DNS stub:
  - `A` lookup
  - `AAAA` lookup
  - `NXDOMAIN` and TTL-expiry behavior

## Thresholds

- Interop pass rate target: `>= 0.95`
- Soak max failures target: `0`

## Evidence

- `tests/net/test_dns_stub_v2.py`
- `tests/net/test_network_gate_v2.py`
- `make test-network-stack-v2`
