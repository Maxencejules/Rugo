# M19 Execution Backlog (Network Stack v2)

Date: 2026-03-06  
Lane: Rugo (Rust kernel + Go user space)  
Status: done

## Goal

Deliver practical network behavior for common multi-purpose workloads with
stronger interop and soak guarantees.

M19 source of truth remains `docs/M15_M20_MULTIPURPOSE_PLAN.md`,
`MILESTONES.md`, and this backlog.

## Current State Summary

- Network stack v1 is complete with contract and soak/interop gating.
- M19 extends these contracts to v2 scope and thresholds.
- Existing QEMU/net harness patterns can be reused.

## Execution Result

- PR-1: complete (2026-03-08)
- PR-2: complete (2026-03-08)
- PR-3: complete (2026-03-08)

## PR-1: Protocol + Socket Contract v2

### Objective

Freeze protocol/socket v2 behavior as executable contracts.

### Scope

- Add docs:
  - `docs/net/network_stack_contract_v2.md`
  - `docs/net/socket_contract_v2.md`
  - `docs/net/tcp_profile_v2.md`
- Add tests:
  - `tests/net/test_tcp_interop_v2.py`
  - `tests/net/test_ipv6_interop_v2.py`

### Primary files

- `docs/net/network_stack_contract_v2.md`
- `docs/net/socket_contract_v2.md`
- `docs/net/tcp_profile_v2.md`
- `tests/net/test_tcp_interop_v2.py`
- `tests/net/test_ipv6_interop_v2.py`

### Acceptance checks

- `python -m pytest tests/net/test_tcp_interop_v2.py tests/net/test_ipv6_interop_v2.py -v`

### Done criteria for PR-1

- Network/socket contracts are versioned and test-backed.
- Interop baselines are deterministic and reproducible.

### PR-1 completion summary

- Added v2 contract docs for:
  - network stack scope and release-gate requirements,
  - socket semantics and DNS-stub baseline behavior,
  - TCP interop profile and retry bounds.
- Added executable checks for:
  - deterministic TCP peer interop paths,
  - deterministic IPv6 ND/ICMPv6 and dual-stack behavior.

## PR-2: Service Behavior + Diagnostics

### Objective

Strengthen diagnostic and interop/soak evidence collection for v2 network lane.

### Scope

- Add tooling:
  - `tools/run_net_interop_matrix_v2.py`
  - `tools/run_net_soak_v2.py`
- Add tests:
  - `tests/net/test_dns_stub_v2.py`
- Add docs:
  - `docs/net/interop_matrix_v2.md`

### Primary files

- `tools/run_net_interop_matrix_v2.py`
- `tools/run_net_soak_v2.py`
- `tests/net/test_dns_stub_v2.py`
- `docs/net/interop_matrix_v2.md`

### Acceptance checks

- `python tools/run_net_interop_matrix_v2.py --out out/net-interop-v2.json`
- `python tools/run_net_soak_v2.py --out out/net-soak-v2.json`
- `python -m pytest tests/net/test_dns_stub_v2.py -v`

### Done criteria for PR-2

- Network interop/soak artifacts are stable and machine-readable.
- DNS/service baseline behavior is deterministic and test-backed.

### PR-2 completion summary

- Added deterministic tooling:
  - `tools/run_net_interop_matrix_v2.py`
  - `tools/run_net_soak_v2.py`
- Added deterministic artifact schemas:
  - `rugo.net_interop_matrix.v2`
  - `rugo.net_soak_report.v2`
- Added DNS and diagnostics contract docs/tests:
  - `docs/net/interop_matrix_v2.md`
  - `tests/net/test_dns_stub_v2.py`

## PR-3: Network Gate + Milestone Closure

### Objective

Make network stack v2 a release-blocking gate.

### Scope

- Add aggregate test:
  - `tests/net/test_network_gate_v2.py`
- Add local gate:
  - `Makefile` target `test-network-stack-v2`
- Add CI gate:
  - `.github/workflows/ci.yml` step `Network stack v2 gate`

### Primary files

- `tests/net/test_network_gate_v2.py`
- `Makefile`
- `.github/workflows/ci.yml`
- `MILESTONES.md`
- `docs/STATUS.md`

### Acceptance checks

- `make test-network-stack-v2`

### Done criteria for PR-3

- Network v2 gate is required in local and CI release lanes.
- M19 status can be marked done with linked evidence.

### PR-3 completion summary

- Added aggregate gate test:
  - `tests/net/test_network_gate_v2.py`
- Added local gate:
  - `make test-network-stack-v2`
  - JUnit output: `out/pytest-network-stack-v2.xml`
- Added CI gate + artifact upload:
  - step: `Network stack v2 gate`
  - artifact: `network-stack-v2-artifacts`
- Updated milestone/status documents to mark M19 done with evidence links.

## Non-goals for M19 backlog

- Full routing/firewall/NAT platform parity.
- Broad non-virtio NIC family expansion in this milestone.
