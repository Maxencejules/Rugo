# M68 Execution Backlog (Settings, Notifications, and Background UX v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Add a bounded settings and notification baseline with explicit background-
service prompts so common system configuration no longer depends on shell-only
flows.

M68 source of truth remains:

- `docs/POST_G2_EXTENDED_MILESTONES.md`
- `docs/M52_EXECUTION_BACKLOG.md`
- `docs/M67_EXECUTION_BACKLOG.md`
- this backlog

## Current State Summary

- Desktop workflows exist, but system configuration and notifications remain
  minimal.
- File and content workflows in M67 create the need for clearer desktop service
  and prompt behavior.
- There is no versioned settings-panel, notification-center, or background UX
  policy in-tree.
- M68 must define those semantics before multi-monitor and productivity work
  depends on them.

## Execution plan

- PR-1: desktop services contract freeze
- PR-2: settings and notification campaign baseline
- PR-3: desktop services gate wiring + closure

## Execution status

- PR-1: pending
- PR-2: pending
- PR-3: pending

## PR-1: Desktop Services Contract Freeze

### Objective

Define settings-panel, notification, and background-service prompt behavior
before implementation broadens desktop management claims.

### Scope

- Add docs:
  - `docs/desktop/settings_panel_contract_v1.md`
  - `docs/desktop/notification_center_contract_v1.md`
  - `docs/runtime/background_service_ux_policy_v1.md`
- Add tests:
  - `tests/desktop/test_desktop_services_docs_v1.py`

### Primary files

- `docs/desktop/settings_panel_contract_v1.md`
- `docs/desktop/notification_center_contract_v1.md`
- `docs/runtime/background_service_ux_policy_v1.md`
- `tests/desktop/test_desktop_services_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/desktop/test_desktop_services_docs_v1.py -v`

### Done criteria for PR-1

- Settings, notifications, and background prompts are explicit and versioned.
- Permission and service-prompt boundaries are reviewable before implementation.

## PR-2: Settings and Notification Campaign Baseline

### Objective

Implement deterministic evidence for settings workflows, notifications, and
background-service prompts.

### Scope

- Add tooling:
  - `tools/run_desktop_services_v1.py`
  - `tools/run_notification_center_v1.py`
- Add tests:
  - `tests/desktop/test_settings_panels_v1.py`
  - `tests/desktop/test_notification_center_v1.py`
  - `tests/desktop/test_background_service_prompts_v1.py`
  - `tests/desktop/test_settings_negative_v1.py`

### Primary files

- `tools/run_desktop_services_v1.py`
- `tools/run_notification_center_v1.py`
- `tests/desktop/test_settings_panels_v1.py`
- `tests/desktop/test_notification_center_v1.py`
- `tests/desktop/test_background_service_prompts_v1.py`
- `tests/desktop/test_settings_negative_v1.py`

### Acceptance checks

- `python tools/run_desktop_services_v1.py --out out/desktop-services-v1.json`
- `python tools/run_notification_center_v1.py --out out/notification-center-v1.json`
- `python -m pytest tests/desktop/test_settings_panels_v1.py tests/desktop/test_notification_center_v1.py tests/desktop/test_background_service_prompts_v1.py tests/desktop/test_settings_negative_v1.py -v`

### Done criteria for PR-2

- Desktop-services artifacts are deterministic and machine-readable.
- `SHELL: notify ok` and panel/workflow markers are stable.
- Background-service prompts remain explicit and auditable.

## PR-3: Desktop Services Gate + Notification Sub-gate

### Objective

Make settings and notification behavior release-blocking for the declared
desktop profile.

### Scope

- Add local gates:
  - `Makefile` target `test-desktop-services-v1`
  - `Makefile` target `test-notification-center-v1`
- Add CI steps:
  - `Desktop services v1 gate`
  - `Notification center v1 gate`
- Add aggregate tests:
  - `tests/desktop/test_desktop_services_gate_v1.py`
  - `tests/desktop/test_notification_center_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/desktop/test_desktop_services_gate_v1.py`
- `tests/desktop/test_notification_center_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-desktop-services-v1`
- `make test-notification-center-v1`

### Done criteria for PR-3

- Desktop-services and notification sub-gates are required in local and CI
  release lanes.
- M68 can be marked done only with deterministic configuration and prompt
  evidence for declared workflows.

## Non-goals for M68 backlog

- full enterprise policy management breadth
- multi-monitor layout behavior owned by M69
- productivity shell features owned by M70
