# M63 Execution Backlog (VPN Tunnel Primitives v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Add bounded VPN tunnel primitives with explicit key-rotation and kill-switch
semantics while keeping peer policy and orchestration in Go.

M63 source of truth remains:

- `docs/POST_G2_EXTENDED_MILESTONES.md`
- `docs/M62_EXECUTION_BACKLOG.md`
- `docs/M28_EXECUTION_BACKLOG.md`
- this backlog

## Current State Summary

- Firewall primitives in M62 establish the policy boundary tunnels will depend
  on.
- Network maturity and security hardening exist, but there is no versioned VPN
  tunnel ABI or key-rotation policy in-tree.
- Remote-access semantics are still a roadmap-only concept rather than a gated
  contract.
- M63 must freeze those boundaries before wireless roaming and edge routing
  build on them.

## Execution plan

- PR-1: VPN primitive contract freeze
- PR-2: handshake and rekey campaign baseline
- PR-3: VPN gate wiring + closure

## Execution status

- PR-1: pending
- PR-2: pending
- PR-3: pending

## PR-1: VPN Primitive Contract Freeze

### Objective

Define tunnel-session, key rotation, and kill-switch behavior before
implementation lands.

### Scope

- Add docs:
  - `docs/net/wireguard_primitives_contract_v1.md`
  - `docs/net/tunnel_socket_abi_v1.md`
  - `docs/security/vpn_key_rotation_policy_v1.md`
- Add tests:
  - `tests/net/test_vpn_docs_v1.py`
  - `tests/security/test_vpn_key_policy_v1.py`

### Primary files

- `docs/net/wireguard_primitives_contract_v1.md`
- `docs/net/tunnel_socket_abi_v1.md`
- `docs/security/vpn_key_rotation_policy_v1.md`
- `tests/net/test_vpn_docs_v1.py`
- `tests/security/test_vpn_key_policy_v1.py`

### Acceptance checks

- `python -m pytest tests/net/test_vpn_docs_v1.py tests/security/test_vpn_key_policy_v1.py -v`

### Done criteria for PR-1

- Tunnel handshake, rekey, and kill-switch semantics are explicit and
  versioned.
- Unsupported tunnel modes are rejected as deterministic policy outcomes.

## PR-2: Handshake and Rekey Campaign Baseline

### Objective

Implement deterministic evidence for handshake, rekey, roaming-safe endpoints,
and kill-switch behavior.

### Scope

- Add tooling:
  - `tools/run_vpn_baseline_v1.py`
  - `tools/run_vpn_roaming_drill_v1.py`
- Add tests:
  - `tests/net/test_wireguard_handshake_v1.py`
  - `tests/net/test_wireguard_rekey_v1.py`
  - `tests/net/test_vpn_killswitch_v1.py`
  - `tests/net/test_vpn_roaming_v1.py`

### Primary files

- `tools/run_vpn_baseline_v1.py`
- `tools/run_vpn_roaming_drill_v1.py`
- `tests/net/test_wireguard_handshake_v1.py`
- `tests/net/test_wireguard_rekey_v1.py`
- `tests/net/test_vpn_killswitch_v1.py`
- `tests/net/test_vpn_roaming_v1.py`

### Acceptance checks

- `python tools/run_vpn_baseline_v1.py --out out/vpn-baseline-v1.json`
- `python tools/run_vpn_roaming_drill_v1.py --out out/vpn-roaming-v1.json`
- `python -m pytest tests/net/test_wireguard_handshake_v1.py tests/net/test_wireguard_rekey_v1.py tests/net/test_vpn_killswitch_v1.py tests/net/test_vpn_roaming_v1.py -v`

### Done criteria for PR-2

- VPN artifacts are deterministic and machine-readable.
- `VPN: handshake ok` and `VPN: rekey ok` markers are stable across seeded runs.
- Firewall and route consumers can reference explicit tunnel session semantics.

## PR-3: VPN Gate + Roaming Sub-gate

### Objective

Make VPN primitives release-blocking for declared remote-access profiles.

### Scope

- Add local gates:
  - `Makefile` target `test-vpn-baseline-v1`
  - `Makefile` target `test-vpn-roaming-v1`
- Add CI steps:
  - `VPN baseline v1 gate`
  - `VPN roaming v1 gate`
- Add aggregate tests:
  - `tests/net/test_vpn_gate_v1.py`
  - `tests/net/test_vpn_roaming_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/net/test_vpn_gate_v1.py`
- `tests/net/test_vpn_roaming_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-vpn-baseline-v1`
- `make test-vpn-roaming-v1`

### Done criteria for PR-3

- VPN and roaming sub-gates are required in local and CI release lanes.
- M63 can be marked done only with deterministic tunnel evidence and explicit
  kill-switch behavior.

## Non-goals for M63 backlog

- full mesh-routing or overlay control plane beyond the declared tunnel scope
- Wi-Fi control-plane work owned by M64
- broad edge routing and QoS owned by M65
