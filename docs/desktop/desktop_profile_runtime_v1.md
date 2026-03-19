# Desktop Profile Runtime Qualification v1

Date: 2026-03-18  
Track: X4 Desktop and Workflow Breadth  
Lane: Rugo (Rust kernel + Go user space)  
Status: active aggregate gate

## Goal

Turn the historical X4 desktop backlog into one shared runtime-backed
qualification bundle with:

- a bootable desktop-profile image on the real Rugo userspace lane,
- bounded display, input, window/compositor, GUI, and shell markers on that
  booted image,
- supporting runtime reports for the desktop stack layers,
- bounded ecosystem and installer workflows tied to the same desktop profile.

## Report identity

Qualification report schema: `rugo.desktop_profile_runtime_report.v1`.
Qualification policy ID: `rugo.desktop_profile_runtime.v1`.
Desktop profile ID: `rugo.desktop_profile.v2`.
Runtime capture schema: `rugo.booted_runtime_capture.v1`.
Runtime tool: `tools/run_desktop_profile_runtime_v1.py`.

## Historical backlog coverage

The X4 runtime-backed closure covers:

- `M35`
- `M44`
- `M48`
- `M49`
- `M50`
- `M51`
- `M52`

Each backlog must appear in `backlog_closure` with `Runtime-backed` status and
its required runtime checks plus supporting reports.

## Required top-level fields

- `schema`
- `track_id`
- `policy_id`
- `desktop_profile_id`
- `created_utc`
- `seed`
- `gate`
- `capture`
- `checks`
- `summary`
- `backlog_closure`
- `boot_profiles`
- `runtime_components`
- `source_reports`
- `artifact_refs`
- `injected_failures`
- `failures`
- `total_failures`
- `gate_pass`
- `digest`

## Runtime checks

The aggregate report must expose these runtime checks:

- `desktop_bootstrap`
- `display_scanout`
- `input_seat`
- `window_compositor`
- `gui_runtime`
- `shell_workflows`
- `graphical_installer`

`desktop_bootstrap` must confirm `DESKBOOT: profile desktop_v1` and
`DESKBOOT: ready` on both `cold_boot` and `replay_boot`.

`display_scanout` must confirm:

- `DESKDISP: probe virtio-gpu-pci`
- `DESKDISP: mode 1280x720@60`
- `DESKDISP: frame ok`

`input_seat` must confirm:

- `DESKSEAT: seat0 ready`
- `DESKSEAT: focus desktop.shell.launcher`

`window_compositor` must confirm:

- `DESKCOMP: workspace visible`
- `DESKCOMP: files.panel occluded`
- `DESKCOMP: settings.panel focused`

`gui_runtime` must confirm:

- `DESKGUI: toolkit rugo.widgets.retain.v1`
- `DESKGUI: font rugo-sans`

`shell_workflows` must confirm:

- `DSHELL: launcher Files`
- `DSHELL: file save ok`
- `DSHELL: settings apply ok`
- `DSHELL: shutdown guard ok`

`graphical_installer` must confirm:

- `DINST: recovery entry visible`

## Runtime capture requirements

The capture must include both:

- a `cold_boot` profile
- a `replay_boot` profile

The aggregate report must bind those boots to the desktop-profile image and to
the bounded desktop markers listed above.

Boot image: `out/os-go-desktop.iso`.
Kernel image: `out/kernel-go-desktop.elf`.
Primary runtime capture: `out/desktop-profile-capture-v1.json`.
Primary report: `out/desktop-profile-runtime-v1.json`.

## Supporting source reports

The aggregate report must bind the historical X4 lane to these supporting
artifacts:

- `out/desktop-smoke-v1.json`
- `out/gui-app-matrix-v1.json`
- `out/display-runtime-v1.json`
- `out/input-seat-v1.json`
- `out/window-system-v1.json`
- `out/gui-runtime-v1.json`
- `out/toolkit-compat-v1.json`
- `out/desktop-shell-v1.json`
- `out/graphical-installer-v1.json`
- `out/real-gui-matrix-v2.json`
- `out/real-pkg-install-v2.json`
- `out/real-catalog-audit-v2.json`

## Gate binding

- Local gate: `make test-desktop-profile-runtime-v1`.
- CI gate: `Desktop profile runtime v1 gate`.
- CI artifact: `desktop-profile-runtime-v1-artifacts`.
- JUnit report: `out/pytest-desktop-profile-runtime-v1.xml`.
