# M8 Execution Backlog (Compatibility Profile v1)

Date: 2026-03-04  
Lane: Rugo (Rust kernel + Go user space)

## Goal

Drive M8 from post-G2 baseline to a concrete Compatibility Profile v1:

- Freeze a versioned syscall/ABI contract (`syscall_v1`).
- Establish ELF/user-process behavior needed for third-party apps.
- Deliver a first POSIX-oriented compatibility subset with test coverage.
- Bootstrap a package/install path for external user-space binaries.

M8 source of truth remains `MILESTONES.md`, `docs/abi/*`, and this backlog.

## Current State Summary

- M0-M7 and G1-G2 are complete and stable in QEMU.
- ABI docs now include v1 contracts (`docs/abi/syscall_v1.md`, `docs/abi/process_thread_model_v1.md`).
- App execution now has a documented compatibility-profile v1 baseline for loader/process/fd primitives.
- Compatibility Profile v1 document and executable conformance suite now covers loader/process/fd/time/signal/socket under `tests/compat/`.
- Package/repository v1 bootstrap contract and external app lane now exist (`docs/pkg/package_format_v1.md`, `tools/pkg_bootstrap_v1.py`, `tests/pkg/test_pkg_external_apps.py`).

## Execution Result

- PR-1: complete (2026-03-04)
- PR-2: complete (2026-03-04)
- PR-3: complete (2026-03-04)

## PR-1: ABI v1 Contract + Compatibility Test Skeleton

### Objective

Create the Compatibility Profile v1 contract and test harness before kernel/userspace behavior changes.

### Scope

- Add `docs/abi/syscall_v1.md` with:
  - syscall versioning and compatibility rules,
  - deterministic error model,
  - deprecation policy.
- Add compatibility profile doc (`docs/abi/compat_profile_v1.md`) with:
  - required POSIX-oriented API subset,
  - explicit unsupported list.
- Add test skeletons under `tests/compat/`:
  - process lifecycle,
  - file I/O subset,
  - time/signal subset,
  - socket API subset.

### Primary files

- `docs/abi/syscall_v1.md` (new)
- `docs/abi/compat_profile_v1.md` (new)
- `tests/compat/*` (new)
- `MILESTONES.md`
- `docs/STATUS.md`

### Acceptance checks

- `python -m pytest tests/compat -v` (initial skeleton collection must pass)
- Existing baseline remains green:
  - `python -m pytest tests/go/test_go_user_service.py tests/go/test_std_go_binary.py -v`

### Done criteria for PR-1

- Compatibility Profile v1 contract exists, versioned, and reviewable.
- `tests/compat/` suite exists with clear TODO markers and deterministic skip/fail semantics.

## PR-2: ELF/User-Process Semantics + Core Compatibility Primitives

### Objective

Implement kernel/runtime behavior required to run non-demo ELF user programs in a predictable way.

### Scope

- Harden ELF loader semantics:
  - relocation and segment validation policy,
  - aux-vector/startup contract documentation.
- Add process/thread primitives needed by compatibility profile:
  - exit/wait semantics,
  - argv/envp delivery contract,
  - file descriptor table v1 behavior.
- Add/extend syscalls required by profile subset:
  - file/open/read/write/close core semantics,
  - poll/epoll or equivalent wait primitive baseline.

### Primary files

- `kernel_rs/src/lib.rs`
- `docs/abi/syscall_v1.md`
- `docs/abi/process_thread_model_v1.md` (new or upgraded)
- `tests/compat/test_process_*`
- `tests/compat/test_fd_*`
- `tests/compat/test_loader_*`

### Acceptance checks

- `python -m pytest tests/compat/test_loader_*.py -v`
- `python -m pytest tests/compat/test_process_*.py tests/compat/test_fd_*.py -v`
- Regression sanity:
  - `make test-qemu` (or equivalent compatibility subset lane in CI)

### Done criteria for PR-2

- ELF/process/fd behavior required by Compatibility Profile v1 is implemented and test-backed.
- No regressions in existing milestone suites.

## PR-3: POSIX Subset Closure + Package Bootstrap + M8 Gate

### Objective

Close M8 by proving a useful compatibility subset and external app install/run flow.

### Scope

- Implement remaining M8 profile APIs (as scoped in PR-1 docs).
- Add package/repository v1 contract:
  - signed metadata format,
  - package layout and install semantics.
- Add at least one external app suite lane:
  - build, package, install, run in QEMU.
- Promote compatibility checks to release-gating CI.

### Primary files

- `docs/abi/compat_profile_v1.md`
- `docs/pkg/package_format_v1.md` (new)
- `tools/` (package/bootstrap helper scripts)
- `tests/compat/test_posix_subset.py` (or split files)
- `tests/pkg/test_pkg_external_apps.py` (new)
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `python -m pytest tests/compat -v`
- `python -m pytest tests/pkg/test_pkg_external_apps.py -v`
- Regression sanity:
  - `python -m pytest tests/go/test_go_user_service.py tests/go/test_std_go_binary.py -v`

### Done criteria for PR-3

- Compatibility Profile v1 APIs and semantics are documented and implemented.
- External packaged user-space apps run through the v1 compatibility path.
- M8 status is marked `done` in milestone/status docs.

## Sequencing Notes

- PR-1 first: contract and test surface must be clear before implementation.
- PR-2 second: loader/process/fd semantics unblock meaningful compatibility tests.
- PR-3 last: subset closure and package bootstrap provide real M8 completion evidence.

## Non-goals for M8 backlog

- Full Linux syscall ABI emulation.
- Complete POSIX conformance across all options/extensions.
- Full desktop UX stack (graphics, compositor, input method framework).
- Broad bare-metal hardware matrix closure (owned by M9).
