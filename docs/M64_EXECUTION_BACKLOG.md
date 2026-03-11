# M64 Execution Backlog (Wireless Control Plane + Roaming v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Add a user-space wireless control plane with explicit roaming, WPA3, and
loss-recovery semantics on top of the adapter baseline from M56.

M64 source of truth remains:

- `docs/POST_G2_EXTENDED_MILESTONES.md`
- `docs/M56_EXECUTION_BACKLOG.md`
- `docs/M63_EXECUTION_BACKLOG.md`
- this backlog

## Current State Summary

- M56 defines the adapter and firmware boundary for Wi-Fi support.
- M63 adds security-sensitive tunnel behavior that depends on stable wireless
  events.
- There is no versioned v2 Wi-Fi control-plane contract, roaming policy, or
  wireless socket extension contract in-tree.
- M64 must land those contracts before desktop and routing work depends on
  stable wireless mobility behavior.

## Execution plan

- PR-1: wireless control-plane contract freeze
- PR-2: roaming and loss-recovery campaign baseline
- PR-3: wireless-stack gate wiring + closure

## Execution status

- PR-1: pending
- PR-2: pending
- PR-3: pending

## PR-1: Wireless Control-plane Contract Freeze

### Objective

Define the Go-managed wireless control plane and roaming semantics before
implementation broadens laptop or mobile connectivity claims.

### Scope

- Add docs:
  - `docs/net/wifi_control_plane_contract_v2.md`
  - `docs/net/roaming_policy_v1.md`
  - `docs/abi/wireless_socket_extensions_v1.md`
- Add tests:
  - `tests/net/test_wifi_control_plane_docs_v2.py`

### Primary files

- `docs/net/wifi_control_plane_contract_v2.md`
- `docs/net/roaming_policy_v1.md`
- `docs/abi/wireless_socket_extensions_v1.md`
- `tests/net/test_wifi_control_plane_docs_v2.py`

### Acceptance checks

- `python -m pytest tests/net/test_wifi_control_plane_docs_v2.py -v`

### Done criteria for PR-1

- Wireless control-plane, roaming, and WPA3-related boundaries are explicit and
  versioned.
- Deferred or unsupported wireless behaviors remain machine-verifiable.

## PR-2: Roaming and Loss-recovery Campaign Baseline

### Objective

Implement deterministic evidence for scan, roam, WPA3 handoff, and packet-loss
recovery behavior.

### Scope

- Add tooling:
  - `tools/run_wireless_stack_v1.py`
  - `tools/run_roaming_campaign_v1.py`
- Add tests:
  - `tests/net/test_wifi_scan_roam_v1.py`
  - `tests/net/test_wpa3_handshake_v1.py`
  - `tests/net/test_wifi_loss_recovery_v1.py`
  - `tests/net/test_wireless_socket_extensions_v1.py`

### Primary files

- `tools/run_wireless_stack_v1.py`
- `tools/run_roaming_campaign_v1.py`
- `tests/net/test_wifi_scan_roam_v1.py`
- `tests/net/test_wpa3_handshake_v1.py`
- `tests/net/test_wifi_loss_recovery_v1.py`
- `tests/net/test_wireless_socket_extensions_v1.py`

### Acceptance checks

- `python tools/run_wireless_stack_v1.py --out out/wireless-stack-v1.json`
- `python tools/run_roaming_campaign_v1.py --out out/wifi-roaming-v1.json`
- `python -m pytest tests/net/test_wifi_scan_roam_v1.py tests/net/test_wpa3_handshake_v1.py tests/net/test_wifi_loss_recovery_v1.py tests/net/test_wireless_socket_extensions_v1.py -v`

### Done criteria for PR-2

- Wireless-stack artifacts are deterministic and machine-readable.
- `WIFI: roam ok` and loss-recovery markers remain stable for declared
  adapters.
- Later desktop and routing work can reference one versioned roaming policy.

## PR-3: Wireless Stack Gate + Roaming Sub-gate

### Objective

Make the wireless control-plane baseline release-blocking for declared adapter
profiles.

### Scope

- Add local gates:
  - `Makefile` target `test-wireless-stack-v1`
  - `Makefile` target `test-wifi-roaming-v1`
- Add CI steps:
  - `Wireless stack v1 gate`
  - `Wi-Fi roaming v1 gate`
- Add aggregate tests:
  - `tests/net/test_wireless_stack_gate_v1.py`
  - `tests/net/test_wifi_roaming_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/net/test_wireless_stack_gate_v1.py`
- `tests/net/test_wifi_roaming_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-wireless-stack-v1`
- `make test-wifi-roaming-v1`

### Done criteria for PR-3

- Wireless-stack and roaming sub-gates are required in local and CI release
  lanes.
- M64 can be marked done only with deterministic roam and recovery evidence
  bound to audited adapter classes.

## Non-goals for M64 backlog

- full cellular or Bluetooth control-plane work
- general routing/NAT/qdisc work owned by M65
- broad desktop UX claims outside the declared wireless boundary
