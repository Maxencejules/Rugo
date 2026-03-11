# M49 Execution Backlog (Input + Seat Management v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: proposed

## Goal

Implement a real input and seat-management runtime for keyboard and pointer
devices so graphical focus and event delivery are backed by live execution.

M49 source of truth remains `docs/M48_M52_GUI_IMPLEMENTATION_ROADMAP.md`,
`MILESTONES.md`, and this backlog.

## Current State Summary

- Input contracts, desktop focus expectations, and USB input hardware baselines
  already exist.
- Hardware evidence now covers relevant input-capable classes.
- The repo does not yet have a live graphical seat/event runtime for keyboard
  and pointer delivery.

## Execution plan

- PR-1: seat/input contract freeze
- PR-2: input runtime implementation + HID event campaigns
- PR-3: release gate wiring + closure

## Execution Result

- PR-1: not started
- PR-2: not started
- PR-3: not started

## PR-1: Seat/Input Contract Freeze

### Objective

Define seat ownership, event-routing, and focus-delivery rules before input
runtime implementation begins.

### Scope

- Add docs:
  - `docs/desktop/seat_input_contract_v1.md`
  - `docs/desktop/input_event_contract_v1.md`
  - `docs/desktop/focus_routing_policy_v1.md`
- Add tests:
  - `tests/desktop/test_input_seat_docs_v1.py`

### Primary files

- `docs/desktop/seat_input_contract_v1.md`
- `docs/desktop/input_event_contract_v1.md`
- `docs/desktop/focus_routing_policy_v1.md`
- `tests/desktop/test_input_seat_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/desktop/test_input_seat_docs_v1.py -v`

### Done criteria for PR-1

- Seat and input-routing semantics are explicit and versioned.
- Focus ownership and device-delivery rules are reviewable before code lands.

## PR-2: Input Runtime + HID Event Campaigns

### Objective

Implement live keyboard/pointer event delivery and collect deterministic seat
runtime evidence for declared workflows.

### Scope

- Add tooling:
  - `tools/run_input_seat_runtime_v1.py`
  - `tools/run_hid_event_path_v1.py`
- Add tests:
  - `tests/desktop/test_keyboard_event_delivery_v1.py`
  - `tests/desktop/test_pointer_motion_buttons_v1.py`
  - `tests/desktop/test_focus_routing_v1.py`
  - `tests/desktop/test_seat_hotplug_v1.py`

### Primary files

- `tools/run_input_seat_runtime_v1.py`
- `tools/run_hid_event_path_v1.py`
- `tests/desktop/test_keyboard_event_delivery_v1.py`
- `tests/desktop/test_pointer_motion_buttons_v1.py`
- `tests/desktop/test_focus_routing_v1.py`
- `tests/desktop/test_seat_hotplug_v1.py`

### Acceptance checks

- `python tools/run_input_seat_runtime_v1.py --out out/input-seat-v1.json`
- `python tools/run_hid_event_path_v1.py --out out/hid-event-path-v1.json`
- `python -m pytest tests/desktop/test_keyboard_event_delivery_v1.py tests/desktop/test_pointer_motion_buttons_v1.py tests/desktop/test_focus_routing_v1.py tests/desktop/test_seat_hotplug_v1.py -v`

### Done criteria for PR-2

- Input runtime artifacts are deterministic and machine-readable.
- Keyboard/pointer delivery and focus routing are exercised through live
  graphical paths.

## PR-3: Input Seat Gate + HID Event Sub-gate

### Objective

Make real input/seat behavior release-blocking for the declared GUI profile.

### Scope

- Add local gates:
  - `Makefile` target `test-input-seat-v1`
  - `Makefile` target `test-hid-event-path-v1`
- Add CI steps:
  - `Input seat v1 gate`
  - `HID event path v1 gate`
- Add aggregate tests:
  - `tests/desktop/test_input_seat_gate_v1.py`
  - `tests/desktop/test_hid_event_path_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/desktop/test_input_seat_gate_v1.py`
- `tests/desktop/test_hid_event_path_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-input-seat-v1`
- `make test-hid-event-path-v1`

### Done criteria for PR-3

- Real input/seat behavior is release-blocking in local and CI lanes.
- Focus and HID delivery regressions are tied to explicit runtime artifacts.

## Non-goals for M49 backlog

- touch, gesture, stylus, or IME breadth
- audio-volume/media-key integration outside declared workflows
- remote-desktop or multi-seat policy beyond the first bounded seat model
