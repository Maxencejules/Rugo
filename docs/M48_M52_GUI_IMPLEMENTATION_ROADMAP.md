# M48-M52 GUI Implementation Roadmap (Post-M47)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: completed

## Why this document exists

M35-M47 established bounded desktop contracts, runtime qualification artifacts,
and hardware support claims for display/input classes, but an actual in-tree GUI
stack is still missing:

1. The image/build pipeline is stable, but current visuals are still centered on
   serial/QEMU captures rather than a real pixel-producing desktop path.
2. Desktop and GUI qualification reports exist, but they do not yet correspond
   to a live in-tree scanout, input, windowing, and shell implementation.
3. Hardware promotion now covers display/input-capable classes, so the next gap
   is turning that platform base into a usable graphical system.

This roadmap defines the next execution phase to move from "GUI-qualified by
contract and artifacts" to "GUI implemented in-tree and exercised end to end"
without making premature parity claims.

The historical X4 desktop backlog is now closed on the shared desktop profile
runtime qualification lane in `docs/desktop/desktop_profile_runtime_v1.md` and
`make test-desktop-profile-runtime-v1`.

## Scope and boundaries

In scope:

- Start M48-M52 as the next execution phase.
- Preserve the existing bootable image, reproducibility, and release-gate
  discipline as the baseline for GUI work.
- Implement the graphical stack incrementally from scanout and input through
  windowing, app runtime, and shell workflows.

Out of scope:

- universal GPU acceleration or 3D parity,
- immediate X11/Wayland/Win32 compatibility claims,
- audio, webcam, Wi-Fi, Bluetooth, or laptop power-management breadth as part
  of the first GUI implementation phase.

## Sequencing map

| Milestone | Focus | Primary gate |
|---|---|---|
| M48 | Display Runtime + Scanout v1 | `test-display-runtime-v1` |
| M49 | Input + Seat Management v1 | `test-input-seat-v1` |
| M50 | Window System + Composition v1 | `test-window-system-v1` |
| M51 | GUI Runtime + Toolkit Bridge v1 | `test-gui-runtime-v1` |
| M52 | Desktop Shell + Workflow Baseline v1 | `test-desktop-shell-v1` |

### Cross-cutting sub-gates (required)

| Sub-gate | Anchored milestone | Focus |
|---|---|---|
| `test-scanout-path-v1` | M48 | live framebuffer/virtio-gpu scanout and frame capture path |
| `test-hid-event-path-v1` | M49 | keyboard/pointer event routing, focus delivery, and seat ownership |
| `test-compositor-damage-v1` | M50 | surface damage tracking, z-order, and composition correctness |
| `test-toolkit-compat-v1` | M51 | app render/event loop behavior against the declared GUI runtime profile |
| `test-desktop-workflows-v1` | M52 | launcher/settings/file-open/shutdown graphical workflow baseline |

## Aggregate runtime gate

- `test-desktop-profile-runtime-v1`
  Boot-backed X4 qualification across `M35`, `M44`, and `M48-M52` on
  `out/os-go-desktop.iso`.

## Suggested cadence

- Planning cadence: 1 milestone per 8-12 weeks.
- Each milestone follows the established 3-PR pattern:
  - PR-1: contract freeze,
  - PR-2: implementation and runtime campaigns,
  - PR-3: release-gate wiring and closure.

## M48: Display Runtime + Scanout v1

### Objective

Replace display-only qualification artifacts with a real pixel-producing runtime
path that can present frames on declared devices and export machine-verifiable
captures.

### PR-1 (contract freeze)

- Docs:
  - `docs/desktop/display_runtime_contract_v1.md`
  - `docs/desktop/scanout_buffer_contract_v1.md`
  - `docs/desktop/gpu_fallback_policy_v1.md`
- Tests:
  - `tests/desktop/test_display_runtime_docs_v1.py`

### PR-2 (implementation + runtime campaigns)

- Tooling:
  - `tools/run_display_runtime_v1.py`
  - `tools/capture_display_frame_v1.py`
- Tests:
  - `tests/desktop/test_virtio_gpu_scanout_v1.py`
  - `tests/desktop/test_efifb_fallback_v1.py`
  - `tests/desktop/test_display_present_timing_v1.py`
  - `tests/desktop/test_display_frame_capture_v1.py`

### PR-3 (gate + closure)

- Gates:
  - `test-display-runtime-v1`
  - sub-gate `test-scanout-path-v1`
- Aggregate tests:
  - `tests/desktop/test_display_runtime_gate_v1.py`
  - `tests/desktop/test_scanout_path_gate_v1.py`

### Done criteria

- Declared display devices can present real frames through an auditable runtime
  path.
- Frame capture and timing evidence are machine-readable and reproducible.
- Serial-only visuals are no longer the sole evidence for graphical bring-up.

## M49: Input + Seat Management v1

### Objective

Turn input qualification from report-only evidence into a real keyboard/pointer
seat runtime with deterministic focus and delivery semantics.

### PR-1 (contract freeze)

- Docs:
  - `docs/desktop/seat_input_contract_v1.md`
  - `docs/desktop/input_event_contract_v1.md`
  - `docs/desktop/focus_routing_policy_v1.md`
- Tests:
  - `tests/desktop/test_input_seat_docs_v1.py`

### PR-2 (implementation + runtime campaigns)

- Tooling:
  - `tools/run_input_seat_runtime_v1.py`
  - `tools/run_hid_event_path_v1.py`
- Tests:
  - `tests/desktop/test_keyboard_event_delivery_v1.py`
  - `tests/desktop/test_pointer_motion_buttons_v1.py`
  - `tests/desktop/test_focus_routing_v1.py`
  - `tests/desktop/test_seat_hotplug_v1.py`

### PR-3 (gate + closure)

- Gates:
  - `test-input-seat-v1`
  - sub-gate `test-hid-event-path-v1`
- Aggregate tests:
  - `tests/desktop/test_input_seat_gate_v1.py`
  - `tests/desktop/test_hid_event_path_gate_v1.py`

### Done criteria

- Keyboard and pointer input flow through a real seat/event runtime.
- Focus ownership and delivery semantics are deterministic under declared
  desktop workflows.
- Input evidence is collected from live graphical paths rather than synthetic
  result generation.

## M50: Window System + Composition v1

### Objective

Implement an in-tree window/surface/compositor stack that owns lifecycle,
damage, visibility, and composition behavior for the bounded desktop profile.

### PR-1 (contract freeze)

- Docs:
  - `docs/desktop/surface_lifecycle_contract_v1.md`
  - `docs/desktop/compositor_damage_policy_v1.md`
  - `docs/desktop/window_manager_contract_v2.md`
- Tests:
  - `tests/desktop/test_window_system_docs_v1.py`

### PR-2 (implementation + runtime campaigns)

- Tooling:
  - `tools/run_window_system_runtime_v1.py`
  - `tools/run_compositor_damage_v1.py`
- Tests:
  - `tests/desktop/test_surface_lifecycle_v1.py`
  - `tests/desktop/test_window_zorder_v1.py`
  - `tests/desktop/test_compositor_damage_regions_v1.py`
  - `tests/desktop/test_window_resize_move_v1.py`

### PR-3 (gate + closure)

- Gates:
  - `test-window-system-v1`
  - sub-gate `test-compositor-damage-v1`
- Aggregate tests:
  - `tests/desktop/test_window_system_gate_v1.py`
  - `tests/desktop/test_compositor_damage_gate_v1.py`

### Done criteria

- Window lifecycle behavior is backed by a real surface/compositor runtime.
- Damage, clipping, and z-order rules are versioned and regression-gated.
- Multiple client windows can coexist without breaking deterministic focus and
  rendering semantics.

## M51: GUI Runtime + Toolkit Bridge v1

### Objective

Expose a stable bounded GUI runtime that lets in-tree apps and declared toolkit
profiles render, receive events, and pass reproducible runtime qualification.

### PR-1 (contract freeze)

- Docs:
  - `docs/desktop/gui_runtime_contract_v1.md`
  - `docs/desktop/toolkit_profile_v1.md`
  - `docs/desktop/font_text_rendering_policy_v1.md`
- Tests:
  - `tests/desktop/test_gui_runtime_docs_v1.py`

### PR-2 (implementation + runtime campaigns)

- Tooling:
  - `tools/run_gui_runtime_v1.py`
  - `tools/run_toolkit_compat_v1.py`
- Tests:
  - `tests/desktop/test_gui_app_launch_render_v1.py`
  - `tests/desktop/test_font_text_rendering_v1.py`
  - `tests/desktop/test_toolkit_event_loop_v1.py`
  - `tests/desktop/test_gui_runtime_negative_v1.py`

### PR-3 (gate + closure)

- Gates:
  - `test-gui-runtime-v1`
  - sub-gate `test-toolkit-compat-v1`
- Aggregate tests:
  - `tests/desktop/test_gui_runtime_gate_v1.py`
  - `tests/desktop/test_toolkit_compat_gate_v1.py`

### Done criteria

- Declared GUI apps can launch, render, and process input through the in-tree
  runtime.
- Text, font, and event-loop behavior are explicit and test-backed.
- Toolkit compatibility claims remain bounded to declared profiles and live
  runtime evidence.

## M52: Desktop Shell + Workflow Baseline v1

### Objective

Deliver a minimal usable graphical shell and a small set of daily-use workflows
on top of the implemented display/input/window/runtime stack.

### PR-1 (contract freeze)

- Docs:
  - `docs/desktop/desktop_shell_contract_v1.md`
  - `docs/desktop/session_workflow_profile_v1.md`
  - `docs/build/graphical_installer_ux_v1.md`
- Tests:
  - `tests/desktop/test_desktop_shell_docs_v1.py`

### PR-2 (implementation + workflow campaigns)

- Tooling:
  - `tools/run_desktop_shell_workflows_v1.py`
  - `tools/run_graphical_installer_smoke_v1.py`
- Tests:
  - `tests/desktop/test_shell_launcher_workflow_v1.py`
  - `tests/desktop/test_file_open_save_workflow_v1.py`
  - `tests/desktop/test_settings_workflow_v1.py`
  - `tests/build/test_graphical_installer_smoke_v1.py`

### PR-3 (gate + closure)

- Gates:
  - `test-desktop-shell-v1`
  - sub-gate `test-desktop-workflows-v1`
- Aggregate tests:
  - `tests/desktop/test_desktop_shell_gate_v1.py`
  - `tests/desktop/test_desktop_workflows_gate_v1.py`

### Done criteria

- A minimal shell can launch apps, switch focus, and complete declared workflows.
- Graphical installer/session workflows are bounded and regression-gated.
- The repo can demonstrate a real graphical desktop path without claiming
  universal desktop parity.

## Exit criteria for M48-M52 phase

The phase should be considered complete only when:

1. All M48-M52 primary gates are green in local and CI lanes.
2. Display, input, windowing, and GUI runtime evidence come from live
   implementations rather than report-only simulation.
3. The desktop shell can complete declared daily-use workflows on the supported
   graphical path.
4. Unsupported graphics, toolkit, and workflow classes remain explicit and
   non-claiming.
