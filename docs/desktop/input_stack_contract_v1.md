# Input Stack Contract v1

Date: 2026-03-09  
Milestone: M35 Desktop + Interactive UX Baseline v1  
Status: active release gate

## Objective

Define deterministic keyboard and pointer baseline behavior for interactive
desktop sessions.

## Contract identifiers

- Input stack contract ID: `rugo.input_stack_contract.v1`
- Parent desktop profile ID: `rugo.desktop_profile.v1`
- Input baseline report schema: `rugo.desktop_smoke_report.v1`
- USB/removable child contract ID: `rugo.usb_input_removable_contract.v1`

## Required input checks

- `input_keyboard_latency`
  - keyboard event p95 latency must be `<= 12 ms`.
- `input_pointer_latency`
  - pointer move/click event p95 latency must be `<= 14 ms`.
- `input_focus_delivery`
  - focused window must receive keyboard and pointer events reliably.
- `input_repeat_consistency`
  - deterministic key repeat and pointer stream sequencing is required.

## Reliability thresholds

- input delivery success ratio must be `>= 0.995`
- dropped input events must be `<= 2` for baseline campaign

## Input device bridge requirements

- Desktop smoke reports used for M46 qualification must expose `input_class`,
  `input_device`, and `desktop_input_checks`.
- `desktop_input_checks` must bind the qualifying input checks to a named input
  device class instead of a generic "input ready" marker.
- USB qualification for M46 is bounded to `usb-hid`.
- A qualification run that omits `input_class` or fails
  `desktop_input_checks` cannot be used for hardware support claims.
- Removable-media evidence remains separate and is bounded by
  `docs/hw/usb_input_removable_contract_v1.md`.

## Tooling and gate wiring

- Smoke runner: `tools/run_desktop_smoke_v1.py`
- Local desktop gate: `make test-desktop-stack-v1`
- CI desktop gate: `Desktop stack v1 gate`
