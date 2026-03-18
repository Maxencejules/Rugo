# M40 Execution Backlog (Runtime-Backed Evidence Integrity v1)

Date: 2026-03-09  
Lane: Rugo (Rust kernel + Go user space)  
Status: done

## Goal

Convert release-bound evidence from deterministic synthetic-only reports to
runtime-backed artifacts with explicit provenance and auditability.

M40 source of truth remains `docs/M40_M44_GENERAL_PURPOSE_PARITY_ROADMAP.md`,
`MILESTONES.md`, and this backlog.

## Current State Summary

- Release-bound evidence now derives from booted runtime capture, performance
  artifacts, diagnostics, and panic capture instead of synthetic-only tooling.
- Runtime evidence and audit reports now enforce default-image provenance and
  boot-instance linkage across `qemu` and `panic` lanes.
- Hardware, desktop, and ecosystem claims remain bounded, but the release
  evidence chain is now tied to the shipped default image flow.

## Execution Result

- PR-1: complete (2026-03-10)
- PR-2: complete (2026-03-10)
- PR-3: complete (2026-03-10)

## Historical Rugo implementation summary

### Historical Rust kernel surface

- M40 was not a large new kernel-feature milestone. Its Rust-side impact was to
  make existing kernel-owned runtime markers and failure paths auditable enough
  to support provenance-backed release evidence.
- `kernel_rs/src/`, `arch/`, and `boot/` remained the runtime sources whose
  serial markers, trap behavior, and boot paths were being audited rather than
  substantially expanded here.

### Historical Go user space surface

- `services/go/`: limited direct ownership. The main userspace role was to act
  as part of the runtime evidence source being collected and audited.
- `services/go_std/`: not the primary path for this milestone.

### Historical Language-Native Verification

- `make kernel`
- `make userspace`
- `make image-demo`
- `make smoke-demo`
- `make test-evidence-integrity-v1`
- `make test-synthetic-evidence-ban-v1`

## PR-1: Evidence Integrity Contract Freeze

### Objective

Define M40 evidence provenance and synthetic-artifact policy as executable
contracts.

### Scope

- Add docs:
  - `docs/runtime/evidence_integrity_policy_v1.md`
  - `docs/runtime/runtime_evidence_schema_v1.md`
  - `docs/runtime/gate_provenance_policy_v1.md`
- Add tests:
  - `tests/runtime/test_evidence_integrity_docs_v1.py`

### Primary files

- `docs/runtime/evidence_integrity_policy_v1.md`
- `docs/runtime/runtime_evidence_schema_v1.md`
- `docs/runtime/gate_provenance_policy_v1.md`
- `tests/runtime/test_evidence_integrity_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/runtime/test_evidence_integrity_docs_v1.py -v`

### Done criteria for PR-1

- Evidence integrity policy is explicit, versioned, and test-backed.
- Runtime provenance requirements are machine-checkable.

### PR-1 completion summary

- Added evidence-integrity contract docs:
  - `docs/runtime/evidence_integrity_policy_v1.md`
  - `docs/runtime/runtime_evidence_schema_v1.md`
  - `docs/runtime/gate_provenance_policy_v1.md`
- Added executable contract checks:
  - `tests/runtime/test_evidence_integrity_docs_v1.py`

## PR-2: Runtime Evidence Collection + Audit Tooling

### Objective

Implement collection of runtime evidence from booted artifacts and enforce
provenance audits over release-bound artifacts.

### Scope

- Add tooling:
  - `tools/collect_booted_runtime_v1.py`
  - `tools/runtime_capture_common_v1.py`
  - `tools/collect_runtime_evidence_v1.py`
  - `tools/audit_gate_evidence_v1.py`
- Add tests:
  - `tests/runtime/test_booted_runtime_capture_v1.py`
  - `tests/runtime/test_runtime_evidence_collection_v1.py`
  - `tests/runtime/test_gate_evidence_audit_v1.py`
  - `tests/runtime/test_evidence_trace_linkage_v1.py`

### Primary files

- `tools/collect_booted_runtime_v1.py`
- `tools/runtime_capture_common_v1.py`
- `tools/collect_runtime_evidence_v1.py`
- `tools/audit_gate_evidence_v1.py`
- `tests/runtime/test_booted_runtime_capture_v1.py`
- `tests/runtime/test_runtime_evidence_collection_v1.py`
- `tests/runtime/test_gate_evidence_audit_v1.py`
- `tests/runtime/test_evidence_trace_linkage_v1.py`

### Acceptance checks

- `python tools/collect_booted_runtime_v1.py --image out/os-go.iso --kernel out/kernel-go.elf --panic-image out/os-panic.iso --out out/booted-runtime-v1.json`
- `python tools/run_perf_baseline_v1.py --runtime-capture out/booted-runtime-v1.json --out out/perf-baseline-v1.json`
- `python tools/check_perf_regression_v1.py --baseline out/perf-baseline-v1.json --runtime-capture out/booted-runtime-v1.json --out out/perf-regression-v1.json`
- `python tools/collect_trace_bundle_v2.py --runtime-capture out/booted-runtime-v1.json --window-seconds 300 --out out/trace-bundle-v2.json`
- `python tools/collect_diagnostic_snapshot_v2.py --runtime-capture out/booted-runtime-v1.json --trace-bundle out/trace-bundle-v2.json --out out/diagnostic-snapshot-v2.json`
- `python tools/collect_crash_dump_v1.py --release-image out/os-go.iso --kernel out/kernel-go.elf --panic-image out/os-panic.iso --out out/crash-dump-v1.json`
- `python tools/symbolize_crash_dump_v1.py --dump out/crash-dump-v1.json --out out/crash-dump-symbolized-v1.json`
- `python tools/collect_runtime_evidence_v1.py --runtime-capture out/booted-runtime-v1.json --trace-bundle out/trace-bundle-v2.json --diagnostic-snapshot out/diagnostic-snapshot-v2.json --crash-dump out/crash-dump-v1.json --crash-symbolized out/crash-dump-symbolized-v1.json --perf-baseline out/perf-baseline-v1.json --perf-regression out/perf-regression-v1.json --out out/runtime-evidence-v1.json`
- `python tools/audit_gate_evidence_v1.py --evidence out/runtime-evidence-v1.json --out out/gate-evidence-audit-v1.json`
- `python -m pytest tests/runtime/test_booted_runtime_capture_v1.py tests/runtime/test_runtime_evidence_collection_v1.py tests/runtime/test_gate_evidence_audit_v1.py tests/runtime/test_evidence_trace_linkage_v1.py -v`

### Done criteria for PR-2

- Runtime evidence artifacts are machine-readable, runtime-backed, and linked
  to execution traces.
- Provenance audit fails when synthetic-only artifacts, detached traces, or
  default-image drift are provided.

### PR-2 completion summary

- Added boot-backed runtime evidence tooling:
  - `tools/collect_booted_runtime_v1.py`
  - `tools/runtime_capture_common_v1.py`
  - `tools/collect_runtime_evidence_v1.py`
  - `tools/audit_gate_evidence_v1.py`
- Added executable runtime evidence checks:
  - `tests/runtime/test_booted_runtime_capture_v1.py`
  - `tests/runtime/test_runtime_evidence_collection_v1.py`
  - `tests/runtime/test_gate_evidence_audit_v1.py`
  - `tests/runtime/test_evidence_trace_linkage_v1.py`

## PR-3: Evidence Integrity Gate + Synthetic-Evidence Ban Sub-gate

### Objective

Make evidence provenance enforcement release-blocking.

### Scope

- Add local gates:
  - `Makefile` target `test-evidence-integrity-v1`
  - `Makefile` target `test-synthetic-evidence-ban-v1`
- Add CI steps:
  - `Evidence integrity v1 gate`
  - `Synthetic evidence ban v1 gate`
- Add aggregate tests:
  - `tests/runtime/test_evidence_integrity_gate_v1.py`
  - `tests/runtime/test_synthetic_evidence_ban_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/runtime/test_evidence_integrity_gate_v1.py`
- `tests/runtime/test_synthetic_evidence_ban_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-evidence-integrity-v1`
- `make test-synthetic-evidence-ban-v1`

### Done criteria for PR-3

- Evidence-integrity and synthetic-ban sub-gates are required in local and CI
  release lanes.
- M40 can be marked done only with runtime-provenance artifacts.

### PR-3 completion summary

- Added aggregate gate checks:
  - `tests/runtime/test_evidence_integrity_gate_v1.py`
  - `tests/runtime/test_synthetic_evidence_ban_v1.py`
- Added local gates:
  - `make test-evidence-integrity-v1`
  - `make test-synthetic-evidence-ban-v1`
  - runtime capture artifact: `out/booted-runtime-v1.json`
- Added CI gates and artifacts:
  - `Evidence integrity v1 gate`
  - `Synthetic evidence ban v1 gate`
- Updated closure docs:
  - `MILESTONES.md`
  - `docs/STATUS.md`
  - `README.md`

## Non-goals for M40 backlog

- Expanding syscall/hardware scope without evidence-provenance closure first.
- Claiming Linux/Windows parity from synthetic campaign outputs.
