# M49 Execution Backlog (Input + Seat Management v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: done

## Goal

Implement a real input and seat-management runtime for keyboard and pointer
devices so graphical focus and event delivery are backed by live execution.

M49 source of truth remains `docs/M48_M52_GUI_IMPLEMENTATION_ROADMAP.md`,
`MILESTONES.md`, and this backlog.

## Current State Summary

- Input contracts, desktop focus expectations, and USB input hardware baselines
  already exist.
- Hardware evidence now covers relevant input-capable classes.
- A deterministic seat runtime and HID event-path campaign now back graphical
  keyboard/pointer delivery, focus routing, and hotplug evidence.

## Execution plan

- PR-1: seat/input contract freeze
- PR-2: input runtime implementation + HID event campaigns
- PR-3: release gate wiring + closure

## Execution Result

- PR-1: complete (2026-03-11)
- PR-2: complete (2026-03-11)
- PR-3: complete (2026-03-11)

## Historical Rugo implementation summary

### Historical Rust kernel surface

- `kernel_rs/src/`: live input delivery, seat ownership hooks, HID-path
  handling, and deterministic hotplug behavior for declared input devices.
- `arch/` and `boot/`: low-level USB and interrupt behavior needed for stable
  keyboard and pointer event delivery.

### Historical Go user space surface

- `services/go/`: focus-aware event consumers and graphical workflows that
  depended on live keyboard and pointer delivery.
- `services/go_std/`: not the primary path for this milestone.

### Historical Language-Native Verification

- `make kernel`
- `make userspace`
- `make image-demo`
- `make smoke-demo`
- `make test-input-seat-v1`
- `make test-hid-event-path-v1`

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

### PR-1 completion summary

- Added contract docs:
  - `docs/desktop/seat_input_contract_v1.md`
  - `docs/desktop/input_event_contract_v1.md`
  - `docs/desktop/focus_routing_policy_v1.md`
- Added executable doc checks:
  - `tests/desktop/test_input_seat_docs_v1.py`

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

### PR-2 completion summary

- Added deterministic seat runtime tooling:
  - `tools/run_input_seat_runtime_v1.py`
  - `tools/run_hid_event_path_v1.py`
- Added executable runtime and HID path checks:
  - `tests/desktop/test_keyboard_event_delivery_v1.py`
  - `tests/desktop/test_pointer_motion_buttons_v1.py`
  - `tests/desktop/test_focus_routing_v1.py`
  - `tests/desktop/test_seat_hotplug_v1.py`

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

### PR-3 completion summary

- Added aggregate gate checks:
  - `tests/desktop/test_input_seat_gate_v1.py`
  - `tests/desktop/test_hid_event_path_gate_v1.py`
- Added local gates:
  - `make test-input-seat-v1`
  - `make test-hid-event-path-v1`
- Added CI qualification gates and artifacts:
  - `Input seat v1 gate`
  - `HID event path v1 gate`
- Updated closure docs:
  - `MILESTONES.md`
  - `docs/STATUS.md`
  - `README.md`

## X4 runtime-backed closure addendum (2026-03-18)

- This backlog is now governed by the shared desktop profile runtime
  qualification lane.
- The historical seat and HID reports now feed the boot-backed desktop image
  and the shared X4 aggregate gate.
- Future input breadth must land through the same desktop-profile runtime
  qualification path before it is treated as an expanded X4 claim.

## Non-goals for M49 backlog

- touch, gesture, stylus, or IME breadth
- audio-volume/media-key integration outside declared workflows
- remote-desktop or multi-seat policy beyond the first bounded seat model
