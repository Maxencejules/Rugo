# M62 Execution Backlog (Packet Filter + Firewall Primitives v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Add a bounded firewall baseline with kernel packet-filter primitives and a Go
policy daemon boundary, without expanding into an unbounded rule engine.

M62 source of truth remains:

- `docs/POST_G2_EXTENDED_MILESTONES.md`
- `docs/net/network_stack_contract_v1.md`
- `docs/M56_EXECUTION_BACKLOG.md`
- this backlog

## Current State Summary

- Network stack maturity exists through v2, but firewall semantics remain out
  of scope.
- M56 begins Wi-Fi support, increasing the need for explicit interface zoning
  and policy boundaries.
- There is no versioned packet-filter hook contract, socket policy extension,
  or user-space network-policy API in-tree.
- M62 must define those surfaces before VPN, roaming, or routing features build
  on them.

## Execution plan

- PR-1: firewall contract freeze
- PR-2: stateful rule and zone campaign baseline
- PR-3: firewall gate wiring + closure

## Execution status

- PR-1: pending
- PR-2: pending
- PR-3: pending

## PR-1: Firewall Contract Freeze

### Objective

Define packet-filter, state-table, and socket-policy behavior before
implementation broadens network policy claims.

### Scope

- Add docs:
  - `docs/net/firewall_hook_contract_v1.md`
  - `docs/net/net_policy_daemon_api_v1.md`
  - `docs/abi/socket_policy_extensions_v1.md`
- Add tests:
  - `tests/net/test_firewall_docs_v1.py`

### Primary files

- `docs/net/firewall_hook_contract_v1.md`
- `docs/net/net_policy_daemon_api_v1.md`
- `docs/abi/socket_policy_extensions_v1.md`
- `tests/net/test_firewall_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/net/test_firewall_docs_v1.py -v`

### Done criteria for PR-1

- Firewall hooks, zone semantics, and policy-daemon boundaries are explicit and
  versioned.
- Unsupported or deferred rule classes are described as deterministic failures.

## PR-2: Stateful Rule and Zone Campaign Baseline

### Objective

Implement deterministic evidence for stateful filtering, zoning, and negative
paths.

### Scope

- Add tooling:
  - `tools/run_firewall_campaign_v1.py`
  - `tools/run_firewall_policy_audit_v1.py`
- Add tests:
  - `tests/net/test_firewall_stateful_rules_v1.py`
  - `tests/net/test_firewall_zone_policy_v1.py`
  - `tests/net/test_firewall_negative_paths_v1.py`
  - `tests/net/test_socket_policy_extensions_v1.py`

### Primary files

- `tools/run_firewall_campaign_v1.py`
- `tools/run_firewall_policy_audit_v1.py`
- `tests/net/test_firewall_stateful_rules_v1.py`
- `tests/net/test_firewall_zone_policy_v1.py`
- `tests/net/test_firewall_negative_paths_v1.py`
- `tests/net/test_socket_policy_extensions_v1.py`

### Acceptance checks

- `python tools/run_firewall_campaign_v1.py --out out/firewall-campaign-v1.json`
- `python tools/run_firewall_policy_audit_v1.py --out out/firewall-policy-audit-v1.json`
- `python -m pytest tests/net/test_firewall_stateful_rules_v1.py tests/net/test_firewall_zone_policy_v1.py tests/net/test_firewall_negative_paths_v1.py tests/net/test_socket_policy_extensions_v1.py -v`

### Done criteria for PR-2

- Firewall artifacts are deterministic and machine-readable.
- `FW: allow ok` and `FW: drop ok` markers remain stable for declared rule
  classes.
- The Go policy daemon can be validated against one explicit kernel hook model.

## PR-3: Firewall Gate + Policy Audit Sub-gate

### Objective

Make the firewall baseline release-blocking for declared network profiles.

### Scope

- Add local gates:
  - `Makefile` target `test-firewall-v1`
  - `Makefile` target `test-firewall-policy-audit-v1`
- Add CI steps:
  - `Firewall v1 gate`
  - `Firewall policy audit v1 gate`
- Add aggregate tests:
  - `tests/net/test_firewall_gate_v1.py`
  - `tests/net/test_firewall_policy_audit_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/net/test_firewall_gate_v1.py`
- `tests/net/test_firewall_policy_audit_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-firewall-v1`
- `make test-firewall-policy-audit-v1`

### Done criteria for PR-3

- Firewall and policy-audit sub-gates are required in local and CI release
  lanes.
- M62 can be marked done only with deterministic filtering evidence and no
  undocumented policy broadening.

## Non-goals for M62 backlog

- full nftables or iptables feature parity
- encrypted tunnel support owned by M63
- routing and traffic shaping owned by M65
