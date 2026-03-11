# M69 Execution Backlog (Multi-Monitor + HiDPI Workspace v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Add a bounded multi-monitor and HiDPI workspace baseline for declared virtio
and native GPU profiles.

M69 source of truth remains:

- `docs/POST_G2_EXTENDED_MILESTONES.md`
- `docs/M55_EXECUTION_BACKLOG.md`
- `docs/M68_EXECUTION_BACKLOG.md`
- this backlog

## Current State Summary

- GPU acceleration and desktop configuration work establish the prerequisites
  for multi-head workflows.
- Multi-monitor and scaling behavior is not yet an explicit, test-gated
  contract.
- There is no versioned multi-monitor, HiDPI, or display-profile policy in the
  post-M52 desktop plan.
- M69 must define those semantics before productivity and session-restore
  workflows depend on them.

## Execution plan

- PR-1: multi-monitor contract freeze
- PR-2: layout and scaling campaign baseline
- PR-3: multi-monitor gate wiring + closure

## Execution status

- PR-1: pending
- PR-2: pending
- PR-3: pending

## PR-1: Multi-monitor Contract Freeze

### Objective

Define multi-head layout, HiDPI scaling, and display-profile semantics before
implementation broadens workstation claims.

### Scope

- Add docs:
  - `docs/desktop/multi_monitor_contract_v1.md`
  - `docs/desktop/hidpi_scaling_policy_v1.md`
  - `docs/desktop/display_profile_contract_v1.md`
- Add tests:
  - `tests/desktop/test_multi_monitor_docs_v1.py`

### Primary files

- `docs/desktop/multi_monitor_contract_v1.md`
- `docs/desktop/hidpi_scaling_policy_v1.md`
- `docs/desktop/display_profile_contract_v1.md`
- `tests/desktop/test_multi_monitor_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/desktop/test_multi_monitor_docs_v1.py -v`

### Done criteria for PR-1

- Multi-monitor, scaling, and display-profile behavior is explicit and
  versioned.
- Hotplug and unsupported-layout paths are reviewable before implementation.

## PR-2: Layout and Scaling Campaign Baseline

### Objective

Implement deterministic evidence for layout, scaling, and GPU/display hotplug
behavior.

### Scope

- Add tooling:
  - `tools/run_multi_monitor_campaign_v1.py`
  - `tools/run_hidpi_profile_audit_v1.py`
- Add tests:
  - `tests/desktop/test_multi_monitor_layout_v1.py`
  - `tests/desktop/test_hidpi_scaling_v1.py`
  - `tests/desktop/test_gpu_hotplug_display_v1.py`
  - `tests/desktop/test_multi_monitor_negative_v1.py`

### Primary files

- `tools/run_multi_monitor_campaign_v1.py`
- `tools/run_hidpi_profile_audit_v1.py`
- `tests/desktop/test_multi_monitor_layout_v1.py`
- `tests/desktop/test_hidpi_scaling_v1.py`
- `tests/desktop/test_gpu_hotplug_display_v1.py`
- `tests/desktop/test_multi_monitor_negative_v1.py`

### Acceptance checks

- `python tools/run_multi_monitor_campaign_v1.py --out out/multi-monitor-v1.json`
- `python tools/run_hidpi_profile_audit_v1.py --out out/hidpi-profile-v1.json`
- `python -m pytest tests/desktop/test_multi_monitor_layout_v1.py tests/desktop/test_hidpi_scaling_v1.py tests/desktop/test_gpu_hotplug_display_v1.py tests/desktop/test_multi_monitor_negative_v1.py -v`

### Done criteria for PR-2

- Multi-monitor artifacts are deterministic and machine-readable.
- `DISPLAY: head2 ok` and scaling markers remain stable.
- Desktop profile consumers can reference explicit display-profile IDs.

## PR-3: Multi-monitor Gate + Display Profile Sub-gate

### Objective

Make multi-head and HiDPI behavior release-blocking for declared desktop GPU
profiles.

### Scope

- Add local gates:
  - `Makefile` target `test-multi-monitor-v1`
  - `Makefile` target `test-display-profiles-v1`
- Add CI steps:
  - `Multi monitor v1 gate`
  - `Display profiles v1 gate`
- Add aggregate tests:
  - `tests/desktop/test_multi_monitor_gate_v1.py`
  - `tests/desktop/test_display_profiles_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/desktop/test_multi_monitor_gate_v1.py`
- `tests/desktop/test_display_profiles_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-multi-monitor-v1`
- `make test-display-profiles-v1`

### Done criteria for PR-3

- Multi-monitor and display-profile sub-gates are required in local and CI
  release lanes.
- M69 can be marked done only with deterministic multi-head and scaling
  evidence for declared GPU profiles.

## Non-goals for M69 backlog

- full docking/suspend workstation policy breadth
- productivity shell behavior owned by M70
- universal native GPU parity beyond declared profiles
