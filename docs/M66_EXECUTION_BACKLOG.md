# M66 Execution Backlog (Accessibility + Assistive Hooks v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Add a lightweight but explicit accessibility baseline with assistive hooks,
keyboard navigation, and machine-readable accessibility events.

M66 source of truth remains:

- `docs/POST_G2_EXTENDED_MILESTONES.md`
- `docs/M52_EXECUTION_BACKLOG.md`
- `docs/M55_EXECUTION_BACKLOG.md`
- this backlog

## Current State Summary

- The desktop shell and GUI runtime are usable, but accessibility remains out
  of scope.
- GPU and input expansion in M55 and earlier milestones make richer desktop
  workflows possible, increasing the need for explicit accessibility behavior.
- There is no versioned accessibility tree, assistive event API, or focus
  semantics revision in-tree.
- M66 must define those boundaries before file management and productivity
  workflows depend on them.

## Execution plan

- PR-1: accessibility contract freeze
- PR-2: assistive and keyboard campaign baseline
- PR-3: accessibility gate wiring + closure

## Execution status

- PR-1: pending
- PR-2: pending
- PR-3: pending

## PR-1: Accessibility Contract Freeze

### Objective

Define assistive-tree, navigation, and focus semantics before implementation
widens desktop usability claims.

### Scope

- Add docs:
  - `docs/desktop/accessibility_contract_v1.md`
  - `docs/desktop/assistive_event_api_v1.md`
  - `docs/desktop/focus_semantics_v2.md`
- Add tests:
  - `tests/desktop/test_accessibility_docs_v1.py`

### Primary files

- `docs/desktop/accessibility_contract_v1.md`
- `docs/desktop/assistive_event_api_v1.md`
- `docs/desktop/focus_semantics_v2.md`
- `tests/desktop/test_accessibility_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/desktop/test_accessibility_docs_v1.py -v`

### Done criteria for PR-1

- Accessibility tree, keyboard navigation, and assistive events are explicit
  and versioned.
- Visual-only behavior is replaced with reviewable machine-readable semantics.

## PR-2: Assistive and Keyboard Campaign Baseline

### Objective

Implement deterministic evidence for assistive hooks, keyboard navigation, and
high-contrast behavior.

### Scope

- Add tooling:
  - `tools/run_accessibility_campaign_v1.py`
  - `tools/run_focus_a11y_audit_v1.py`
- Add tests:
  - `tests/desktop/test_screen_reader_hooks_v1.py`
  - `tests/desktop/test_keyboard_navigation_v1.py`
  - `tests/desktop/test_high_contrast_policy_v1.py`
  - `tests/desktop/test_accessibility_negative_v1.py`

### Primary files

- `tools/run_accessibility_campaign_v1.py`
- `tools/run_focus_a11y_audit_v1.py`
- `tests/desktop/test_screen_reader_hooks_v1.py`
- `tests/desktop/test_keyboard_navigation_v1.py`
- `tests/desktop/test_high_contrast_policy_v1.py`
- `tests/desktop/test_accessibility_negative_v1.py`

### Acceptance checks

- `python tools/run_accessibility_campaign_v1.py --out out/accessibility-v1.json`
- `python tools/run_focus_a11y_audit_v1.py --out out/focus-a11y-v1.json`
- `python -m pytest tests/desktop/test_screen_reader_hooks_v1.py tests/desktop/test_keyboard_navigation_v1.py tests/desktop/test_high_contrast_policy_v1.py tests/desktop/test_accessibility_negative_v1.py -v`

### Done criteria for PR-2

- Accessibility artifacts are deterministic and machine-readable.
- `A11Y: tree ok` and keyboard-navigation markers are stable.
- Later desktop workflows can reference explicit assistive and focus contracts.

## PR-3: Accessibility Gate + Assistive Sub-gate

### Objective

Make the accessibility baseline release-blocking for declared desktop profiles.

### Scope

- Add local gates:
  - `Makefile` target `test-accessibility-v1`
  - `Makefile` target `test-assistive-hooks-v1`
- Add CI steps:
  - `Accessibility v1 gate`
  - `Assistive hooks v1 gate`
- Add aggregate tests:
  - `tests/desktop/test_accessibility_gate_v1.py`
  - `tests/desktop/test_assistive_hooks_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/desktop/test_accessibility_gate_v1.py`
- `tests/desktop/test_assistive_hooks_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-accessibility-v1`
- `make test-assistive-hooks-v1`

### Done criteria for PR-3

- Accessibility and assistive sub-gates are required in local and CI release
  lanes.
- M66 can be marked done only with deterministic accessibility evidence for the
  declared desktop workflow profile.

## Non-goals for M66 backlog

- full enterprise accessibility-suite breadth
- file-management workflows owned by M67
- multi-monitor and HiDPI behavior owned by M69
