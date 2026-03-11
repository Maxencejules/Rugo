# M65 Execution Backlog (Routing, NAT, and Traffic Control v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Add a bounded routing and traffic-control baseline for edge and gateway use
without expanding into an unbounded appliance feature set.

M65 source of truth remains:

- `docs/POST_G2_EXTENDED_MILESTONES.md`
- `docs/M62_EXECUTION_BACKLOG.md`
- `docs/M64_EXECUTION_BACKLOG.md`
- this backlog

## Current State Summary

- M62-M64 establish firewall, VPN, and wireless control-plane semantics.
- The network stack does not yet expose a versioned routing, NAT, or qdisc
  contract.
- Edge-node and gateway behavior remains outside current release claims.
- M65 must add those data-plane and counter semantics before later desktop and
  fleet work depends on them.

## Execution plan

- PR-1: routing and traffic-control contract freeze
- PR-2: NAT and qdisc campaign baseline
- PR-3: routing/QoS gate wiring + closure

## Execution status

- PR-1: pending
- PR-2: pending
- PR-3: pending

## PR-1: Routing and Traffic-control Contract Freeze

### Objective

Define forwarding, NAT, qdisc, and flow-stat semantics before implementation
widens edge/gateway claims.

### Scope

- Add docs:
  - `docs/net/routing_dataplane_contract_v1.md`
  - `docs/net/traffic_control_contract_v1.md`
  - `docs/abi/network_flow_stats_v1.md`
- Add tests:
  - `tests/net/test_routing_tc_docs_v1.py`

### Primary files

- `docs/net/routing_dataplane_contract_v1.md`
- `docs/net/traffic_control_contract_v1.md`
- `docs/abi/network_flow_stats_v1.md`
- `tests/net/test_routing_tc_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/net/test_routing_tc_docs_v1.py -v`

### Done criteria for PR-1

- Routing, NAT, and traffic-control semantics are explicit and versioned.
- Counter and unsupported-topology behavior are reviewable before
  implementation lands.

## PR-2: NAT and Qdisc Campaign Baseline

### Objective

Implement deterministic evidence for forwarding, NAT, shaping, and loss
recovery under seeded topologies.

### Scope

- Add tooling:
  - `tools/run_routing_tc_campaign_v1.py`
  - `tools/run_lossy_link_service_recovery_v1.py`
- Add tests:
  - `tests/net/test_nat_forwarding_v1.py`
  - `tests/net/test_traffic_shaping_v1.py`
  - `tests/net/test_lossy_link_service_recovery_v1.py`
  - `tests/net/test_network_flow_stats_v1.py`

### Primary files

- `tools/run_routing_tc_campaign_v1.py`
- `tools/run_lossy_link_service_recovery_v1.py`
- `tests/net/test_nat_forwarding_v1.py`
- `tests/net/test_traffic_shaping_v1.py`
- `tests/net/test_lossy_link_service_recovery_v1.py`
- `tests/net/test_network_flow_stats_v1.py`

### Acceptance checks

- `python tools/run_routing_tc_campaign_v1.py --out out/routing-tc-v1.json`
- `python tools/run_lossy_link_service_recovery_v1.py --out out/lossy-link-recovery-v1.json`
- `python -m pytest tests/net/test_nat_forwarding_v1.py tests/net/test_traffic_shaping_v1.py tests/net/test_lossy_link_service_recovery_v1.py tests/net/test_network_flow_stats_v1.py -v`

### Done criteria for PR-2

- Routing and traffic-control artifacts are deterministic and machine-readable.
- `ROUTE: forward ok` and `QDISC: rate ok` markers remain stable.
- Gateway-style profiles can be evaluated without widening claims beyond the
  declared topology contract.

## PR-3: Network Stack v3 Gate + Traffic-control Sub-gate

### Objective

Make routing and traffic-control checks release-blocking for declared edge
profiles.

### Scope

- Add local gates:
  - `Makefile` target `test-network-stack-v3`
  - `Makefile` target `test-traffic-control-v1`
- Add CI steps:
  - `Network stack v3 gate`
  - `Traffic control v1 gate`
- Add aggregate tests:
  - `tests/net/test_network_stack_gate_v3.py`
  - `tests/net/test_traffic_control_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/net/test_network_stack_gate_v3.py`
- `tests/net/test_traffic_control_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-network-stack-v3`
- `make test-traffic-control-v1`

### Done criteria for PR-3

- Routing and traffic-control sub-gates are required in local and CI release
  lanes.
- M65 can be marked done only with deterministic forwarding, NAT, and shaping
  evidence for declared topologies.

## Non-goals for M65 backlog

- full enterprise routing-suite parity
- desktop UX work owned by M66-M70
- fleet-management policy beyond declared gateway data-plane semantics
