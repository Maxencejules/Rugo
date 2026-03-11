# M67 Execution Backlog (File Manager + Content Workflows v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Add a bounded file manager and content-handler workflow baseline for declared
desktop use cases.

M67 source of truth remains:

- `docs/POST_G2_EXTENDED_MILESTONES.md`
- `docs/M52_EXECUTION_BACKLOG.md`
- `docs/M66_EXECUTION_BACKLOG.md`
- this backlog

## Current State Summary

- The desktop shell supports bounded workflows, but file management remains
  rudimentary.
- Accessibility work in M66 defines the usability baseline the file manager
  must inherit.
- There is no versioned file-manager, content-handler, or removable-media UI
  contract in-tree.
- M67 must define those boundaries before settings, notifications, and broader
  productivity workflows build on them.

## Execution plan

- PR-1: file manager contract freeze
- PR-2: file and content workflow campaign baseline
- PR-3: file manager gate wiring + closure

## Execution status

- PR-1: pending
- PR-2: pending
- PR-3: pending

## PR-1: File Manager Contract Freeze

### Objective

Define file browsing, content launch, and removable-media interaction semantics
before implementation broadens desktop claims.

### Scope

- Add docs:
  - `docs/desktop/file_manager_contract_v1.md`
  - `docs/pkg/content_handler_contract_v1.md`
  - `docs/storage/removable_media_ui_policy_v1.md`
- Add tests:
  - `tests/desktop/test_file_manager_docs_v1.py`

### Primary files

- `docs/desktop/file_manager_contract_v1.md`
- `docs/pkg/content_handler_contract_v1.md`
- `docs/storage/removable_media_ui_policy_v1.md`
- `tests/desktop/test_file_manager_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/desktop/test_file_manager_docs_v1.py -v`

### Done criteria for PR-1

- File browsing, content launch, and removable-media workflows are explicit and
  versioned.
- Automount and content-exec boundaries are reviewable before implementation.

## PR-2: File and Content Workflow Campaign Baseline

### Objective

Implement deterministic evidence for file operations, MIME launch, and
removable-media UI behavior.

### Scope

- Add tooling:
  - `tools/run_file_manager_workflows_v1.py`
  - `tools/run_mime_handler_audit_v1.py`
- Add tests:
  - `tests/desktop/test_file_manager_workflows_v1.py`
  - `tests/desktop/test_mime_launch_v1.py`
  - `tests/desktop/test_removable_media_ui_v1.py`
  - `tests/desktop/test_file_manager_negative_v1.py`

### Primary files

- `tools/run_file_manager_workflows_v1.py`
- `tools/run_mime_handler_audit_v1.py`
- `tests/desktop/test_file_manager_workflows_v1.py`
- `tests/desktop/test_mime_launch_v1.py`
- `tests/desktop/test_removable_media_ui_v1.py`
- `tests/desktop/test_file_manager_negative_v1.py`

### Acceptance checks

- `python tools/run_file_manager_workflows_v1.py --out out/file-manager-v1.json`
- `python tools/run_mime_handler_audit_v1.py --out out/mime-handler-v1.json`
- `python -m pytest tests/desktop/test_file_manager_workflows_v1.py tests/desktop/test_mime_launch_v1.py tests/desktop/test_removable_media_ui_v1.py tests/desktop/test_file_manager_negative_v1.py -v`

### Done criteria for PR-2

- File-manager artifacts are deterministic and machine-readable.
- `FM: copy ok` and `FM: mount ok` markers are stable.
- Content-handler behavior is explicit enough to support later desktop app
  workflows without implicit privilege broadening.

## PR-3: File Manager Gate + Content Handler Sub-gate

### Objective

Make file and content workflows release-blocking for the declared desktop
profile.

### Scope

- Add local gates:
  - `Makefile` target `test-file-manager-v1`
  - `Makefile` target `test-content-handlers-v1`
- Add CI steps:
  - `File manager v1 gate`
  - `Content handlers v1 gate`
- Add aggregate tests:
  - `tests/desktop/test_file_manager_gate_v1.py`
  - `tests/desktop/test_content_handlers_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/desktop/test_file_manager_gate_v1.py`
- `tests/desktop/test_content_handlers_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-file-manager-v1`
- `make test-content-handlers-v1`

### Done criteria for PR-3

- File-manager and content-handler sub-gates are required in local and CI
  release lanes.
- M67 can be marked done only with deterministic file and content workflow
  evidence for the declared desktop profile.

## Non-goals for M67 backlog

- full office/media suite breadth
- settings and notification work owned by M68
- multi-monitor layout work owned by M69
