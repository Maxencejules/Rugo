# M56 Execution Backlog (Wireless Adapter + Firmware Baseline v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Add a first real Wi-Fi baseline with explicit adapter, firmware, and control-
plane boundaries while keeping policy in Go user space.

M56 source of truth remains:

- `docs/POST_G2_EXTENDED_MILESTONES.md`
- `docs/M53_EXECUTION_BACKLOG.md`
- `docs/M46_EXECUTION_BACKLOG.md`
- this backlog

## Current State Summary

- Wired and USB/removable-media baselines exist from M46, but wireless remains
  outside the support matrix.
- M53 defines firmware provenance and native-driver safety obligations that
  Wi-Fi work must inherit.
- There is no versioned Wi-Fi adapter, supplicant boundary, or signed firmware
  provenance contract in-tree.
- M56 must land those contracts before networking milestones depend on Wi-Fi
  policy or roaming behavior.

## Execution plan

- PR-1: Wi-Fi and firmware contract freeze
- PR-2: Wi-Fi association and power-state campaign baseline
- PR-3: Wi-Fi gate wiring + closure

## Execution status

- PR-1: pending
- PR-2: pending
- PR-3: pending

## PR-1: Wi-Fi and Firmware Contract Freeze

### Objective

Define the adapter, control-plane, and firmware-provenance boundary for
wireless support.

### Scope

- Add docs:
  - `docs/hw/wifi_driver_contract_v1.md`
  - `docs/net/wifi_control_plane_contract_v1.md`
  - `docs/security/firmware_provenance_policy_v1.md`
- Add tests:
  - `tests/hw/test_wifi_driver_docs_v1.py`
  - `tests/net/test_wifi_control_plane_docs_v1.py`

### Primary files

- `docs/hw/wifi_driver_contract_v1.md`
- `docs/net/wifi_control_plane_contract_v1.md`
- `docs/security/firmware_provenance_policy_v1.md`
- `tests/hw/test_wifi_driver_docs_v1.py`
- `tests/net/test_wifi_control_plane_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/hw/test_wifi_driver_docs_v1.py tests/net/test_wifi_control_plane_docs_v1.py -v`

### Done criteria for PR-1

- Wi-Fi adapter, auth, firmware, and negative-path behavior are explicit and
  versioned.
- Signed-firmware, blob-free, and revoked-firmware paths are policy-bounded.

## PR-2: Wi-Fi Association and Power-State Campaign Baseline

### Objective

Implement deterministic evidence collection for probe, association, power-save,
and firmware enforcement behavior.

### Scope

- Add tooling:
  - `tools/run_wifi_baseline_v1.py`
  - `tools/run_wifi_firmware_audit_v1.py`
- Add tests:
  - `tests/hw/test_wifi_adapter_probe_v1.py`
  - `tests/net/test_wifi_auth_assoc_v1.py`
  - `tests/net/test_wifi_power_save_v1.py`
  - `tests/net/test_wifi_firmware_policy_v1.py`
  - `tests/net/test_wifi_negative_paths_v1.py`

### Primary files

- `tools/run_wifi_baseline_v1.py`
- `tools/run_wifi_firmware_audit_v1.py`
- `tests/hw/test_wifi_adapter_probe_v1.py`
- `tests/net/test_wifi_auth_assoc_v1.py`
- `tests/net/test_wifi_power_save_v1.py`
- `tests/net/test_wifi_firmware_policy_v1.py`
- `tests/net/test_wifi_negative_paths_v1.py`

### Acceptance checks

- `python tools/run_wifi_baseline_v1.py --out out/wifi-baseline-v1.json`
- `python tools/run_wifi_firmware_audit_v1.py --out out/wifi-firmware-audit-v1.json`
- `python -m pytest tests/hw/test_wifi_adapter_probe_v1.py tests/net/test_wifi_auth_assoc_v1.py tests/net/test_wifi_power_save_v1.py tests/net/test_wifi_firmware_policy_v1.py tests/net/test_wifi_negative_paths_v1.py -v`

### Done criteria for PR-2

- Wi-Fi baseline artifacts are deterministic and machine-readable.
- `WIFI: firmware ok`, `WIFI: assoc ok`, and revoked-firmware denial markers
  are stable and auditable.
- Tier-promotion evidence can distinguish adapter class, firmware provenance,
  and power-state behavior.

## PR-3: Wi-Fi Gate + Firmware Audit Sub-gate

### Objective

Make wireless baseline and firmware provenance checks release-blocking.

### Scope

- Add local gates:
  - `Makefile` target `test-wifi-baseline-v1`
  - `Makefile` target `test-wifi-firmware-audit-v1`
- Add CI steps:
  - `Wi-Fi baseline v1 gate`
  - `Wi-Fi firmware audit v1 gate`
- Add aggregate tests:
  - `tests/hw/test_wifi_gate_v1.py`
  - `tests/security/test_wifi_firmware_audit_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/hw/test_wifi_gate_v1.py`
- `tests/security/test_wifi_firmware_audit_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-wifi-baseline-v1`
- `make test-wifi-firmware-audit-v1`

### Done criteria for PR-3

- Wi-Fi and firmware audit sub-gates are required in local and CI release
  lanes.
- M56 can be marked done only with audited adapter evidence and explicit
  firmware provenance reporting.

## Non-goals for M56 backlog

- full wireless roaming and multi-SSID policy owned by M64
- broad laptop power-management parity beyond declared Wi-Fi scope
- shipping unsigned or unaudited firmware in default release images
