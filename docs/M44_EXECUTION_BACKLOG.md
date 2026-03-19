# M44 Execution Backlog (Real Desktop + Ecosystem Qualification v2)

Date: 2026-03-10  
Lane: Rugo (Rust kernel + Go user space)  
Status: done

## Goal

Promote desktop/app/package claims from model-heavy simulation to runtime-backed
qualification with signed and reproducible workload evidence.

M44 source of truth remains `docs/M40_M44_GENERAL_PURPOSE_PARITY_ROADMAP.md`,
`MILESTONES.md`, and this backlog.

## Current State Summary

- Desktop v1 and ecosystem v1 gates were stable, but still bounded and
  model-heavy in several campaign paths.
- Runtime-backed qualification tooling, docs, and deterministic tests are now
  implemented for desktop/app/package v2 claims.
- M44 closure is now wired as release-blocking in local and CI lanes.

## Execution plan

- PR-1: contract freeze
- PR-2: runtime qualification tooling + campaigns
- PR-3: release gate wiring + closure

## Execution Result

- PR-1: complete (2026-03-10)
- PR-2: complete (2026-03-10)
- PR-3: complete (2026-03-10)

## Historical Rugo implementation summary

### Historical Rust kernel surface

- M44 was not primarily a new kernel-feature milestone. Its Rust-side role was
  to provide the stable runtime floor whose display, process, package, and file
  behavior could be qualified with reproducible workload evidence.
- `kernel_rs/src/`, `arch/`, and `boot/` remained the audited runtime sources
  rather than the main implementation focus.

### Historical Go user space surface

- `services/go/`: this milestone was more userspace-facing than M40-M43 because
  desktop/app/package qualification exercised real shell, GUI, and package
  workflows on the default Go userspace path.
- `services/go_std/`: not the primary path for this milestone.

### Historical Language-Native Verification

- `make kernel`
- `make userspace`
- `make image-demo`
- `make smoke-demo`
- `make test-real-ecosystem-desktop-v2`
- `make test-real-app-catalog-v2`

## PR-1: Desktop/Ecosystem v2 Contract Freeze

### Objective

Define desktop profile v2 and ecosystem/distribution v2 expectations as
executable contracts.

### Scope

- Add docs:
  - `docs/desktop/desktop_profile_v2.md`
  - `docs/abi/app_compat_tiers_v2.md`
  - `docs/pkg/ecosystem_scale_policy_v2.md`
  - `docs/pkg/distribution_workflow_v2.md`
- Add tests:
  - `tests/desktop/test_desktop_docs_v2.py`
  - `tests/pkg/test_ecosystem_scale_docs_v2.py`

### Primary files

- `docs/desktop/desktop_profile_v2.md`
- `docs/abi/app_compat_tiers_v2.md`
- `docs/pkg/ecosystem_scale_policy_v2.md`
- `docs/pkg/distribution_workflow_v2.md`
- `tests/desktop/test_desktop_docs_v2.py`
- `tests/pkg/test_ecosystem_scale_docs_v2.py`

### Acceptance checks

- `python -m pytest tests/desktop/test_desktop_docs_v2.py tests/pkg/test_ecosystem_scale_docs_v2.py -v`

### Done criteria for PR-1

- Desktop/ecosystem v2 boundaries are explicit, versioned, and test-backed.
- Workload qualification thresholds are machine-verifiable.

### PR-1 completion summary

- Added contract docs:
  - `docs/desktop/desktop_profile_v2.md`
  - `docs/abi/app_compat_tiers_v2.md`
  - `docs/pkg/ecosystem_scale_policy_v2.md`
  - `docs/pkg/distribution_workflow_v2.md`
- Added executable contract checks:
  - `tests/desktop/test_desktop_docs_v2.py`
  - `tests/pkg/test_ecosystem_scale_docs_v2.py`

## PR-2: Real Qualification Campaign Tooling + Tests

### Objective

Implement deterministic runtime qualification campaigns for desktop GUI and app
catalog/package workflows.

### Scope

- Add tooling:
  - `tools/run_real_gui_app_matrix_v2.py`
  - `tools/run_real_pkg_install_campaign_v2.py`
  - `tools/run_real_catalog_audit_v2.py`
- Add tests:
  - `tests/desktop/test_gui_runtime_qualification_v2.py`
  - `tests/pkg/test_pkg_install_success_rate_v2.py`
  - `tests/pkg/test_catalog_reproducibility_v2.py`
  - `tests/pkg/test_distribution_workflow_v2.py`

### Primary files

- `tools/run_real_gui_app_matrix_v2.py`
- `tools/run_real_pkg_install_campaign_v2.py`
- `tools/run_real_catalog_audit_v2.py`
- `tests/desktop/test_gui_runtime_qualification_v2.py`
- `tests/pkg/test_pkg_install_success_rate_v2.py`
- `tests/pkg/test_catalog_reproducibility_v2.py`
- `tests/pkg/test_distribution_workflow_v2.py`

### Acceptance checks

- `python tools/run_real_gui_app_matrix_v2.py --out out/real-gui-matrix-v2.json`
- `python tools/run_real_pkg_install_campaign_v2.py --out out/real-pkg-install-v2.json`
- `python tools/run_real_catalog_audit_v2.py --out out/real-catalog-audit-v2.json`
- `python -m pytest tests/desktop/test_gui_runtime_qualification_v2.py tests/pkg/test_pkg_install_success_rate_v2.py tests/pkg/test_catalog_reproducibility_v2.py tests/pkg/test_distribution_workflow_v2.py -v`

### Done criteria for PR-2

- Runtime qualification artifacts are deterministic and machine-readable.
- Desktop/app/package quality thresholds are executable and auditable.

### PR-2 completion summary

- Added deterministic runtime qualification tooling:
  - `tools/run_real_gui_app_matrix_v2.py`
  - `tools/run_real_pkg_install_campaign_v2.py`
  - `tools/run_real_catalog_audit_v2.py`
- Added executable runtime qualification checks:
  - `tests/desktop/test_gui_runtime_qualification_v2.py`
  - `tests/pkg/test_pkg_install_success_rate_v2.py`
  - `tests/pkg/test_catalog_reproducibility_v2.py`
  - `tests/pkg/test_distribution_workflow_v2.py`

## PR-3: Real Qualification Gate + App-Catalog Sub-gate

### Objective

Make runtime-backed desktop/ecosystem v2 qualification release-blocking.

### Scope

- Add local gates:
  - `Makefile` target `test-real-ecosystem-desktop-v2`
  - `Makefile` target `test-real-app-catalog-v2`
- Add CI steps:
  - `Real ecosystem desktop v2 gate`
  - `Real app catalog v2 gate`
- Add aggregate tests:
  - `tests/desktop/test_real_desktop_gate_v2.py`
  - `tests/pkg/test_real_catalog_gate_v2.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/desktop/test_real_desktop_gate_v2.py`
- `tests/pkg/test_real_catalog_gate_v2.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-real-ecosystem-desktop-v2`
- `make test-real-app-catalog-v2`

### Done criteria for PR-3

- Real desktop/ecosystem and app-catalog sub-gates are required in local and CI
  release lanes.
- M44 can be marked done only with runtime-qualified ecosystem artifacts.

### PR-3 completion summary

- Added aggregate gate checks:
  - `tests/desktop/test_real_desktop_gate_v2.py`
  - `tests/pkg/test_real_catalog_gate_v2.py`
- Added local gates:
  - `make test-real-ecosystem-desktop-v2`
  - `make test-real-app-catalog-v2`
- Added CI gates and artifacts:
  - `Real ecosystem desktop v2 gate`
  - `Real app catalog v2 gate`
- Updated closure docs:
  - `MILESTONES.md`
  - `docs/STATUS.md`
  - `README.md`

## X4 runtime-backed closure addendum (2026-03-18)

- This backlog is now governed by the shared desktop profile runtime
  qualification lane.
- The bounded real-desktop and ecosystem v2 artifacts now sit beside the
  boot-backed desktop-profile image and are aggregated by
  `make test-desktop-profile-runtime-v1`.
- Future desktop/app breadth must attach to the same boot-backed desktop
  profile before it is treated as a broadened X4 claim.

## Non-goals for M44 backlog

- Universal GUI/app compatibility claims outside declared v2 classes.
- Relaxing signed provenance and reproducibility controls for throughput.
