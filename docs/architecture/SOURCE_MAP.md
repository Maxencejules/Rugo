# Runtime Source Map

This document answers four questions quickly:

1. Where is the actual runtime code?
2. Which commands prove that code is live?
3. Which evidence is direct runtime evidence versus deterministic
   qualification scaffolding?
4. Which paths are archival or supported non-default rather than the default product
   lane?

## Read This Repo In Two Layers

Layer 1: implementation
- kernel source in `arch/`, `boot/`, and `kernel_rs/src/`
- default Go userspace in `services/go/`
- supported stock-Go userspace in `services/go_std/`

Layer 2: qualification
- report generators, build helpers, and acceptance tooling in `tools/`
- QEMU fixtures, contract checks, and aggregate gates in `tests/`
- contracts, policies, ledgers, and archived backlog material in `docs/`

If a claim is backed by a boot path or a QEMU serial assertion, treat it as
live runtime evidence.

If a claim is backed by a seeded JSON report from `tools/run_*` and then a
gate test that validates that report, treat it as qualification scaffolding.

## Current Product Map

| Product surface | Current implementation paths | Direct proof path | Evidence type | Notes |
|-----------------|------------------------------|-------------------|---------------|-------|
| Boot, traps, paging, scheduler, user entry | `arch/`, `boot/`, `kernel_rs/src/lib.rs` | `make image-kernel`, `make boot-kernel`, `tests/boot/*`, `tests/trap/*`, `tests/sched/*`, `tests/user/*` | live runtime | The kernel implementation is real, but it is concentrated in one large Rust file. |
| Dynamic memory foundation: frame allocator, kernel heap, demand paging | `kernel_rs/src/mm.rs` | `make test-mm-foundation-v1`, `tests/mm/test_mm_foundation_v1.py` | live runtime | Contract: `docs/runtime/memory_v1.md`. Compiled into every lane, no feature gates. |
| Preemptive timer scheduling in the default lane | `kernel_rs/src/lib.rs` (`r4_timer_preempt`), `kernel_rs/src/sched.rs`, preemption-safe init protocol in `services/go/` | `make test-sched-preempt-v1`, `tests/sched/test_preempt_default_lane_v1.py` | live runtime | Contract: `docs/runtime/scheduler_v1.md`. PIT at 100 Hz; user tasks run with IF set in the pure go lane. |
| Dynamic task structures: heap-backed task table, demand-paged stacks | `kernel_rs/src/lib.rs` (`r4_tasks_init`, spawn slots), `kernel_rs/src/mm.rs` (stack strides + guards), `services/go/spawnstress.go` | `make test-dynamic-tasks-v1`, `tests/runtime/test_dynamic_tasks_v1.py` | live runtime | Spawn cap 32 in the go lane; 9 concurrent tasks proven at boot; slot reuse keeps service tids stable. |
| Exec-from-filesystem: `sys_spawn` (id 46), hash-verified app loading | `kernel_rs/src/lib.rs` (`sys_spawn_v1`, `exec_load_app`), `apps/base-shell/`, `tools/app_disk_v1.py` | `make test-exec-v1`, `tests/runtime/test_exec_from_fs_v1.py` | live runtime | Contract: `docs/runtime/exec_v1.md`. The shell's `run base-shell` executes a real ELF from the package store on disk. |
| Default Go bootstrap lane | `services/go/` | `make image-demo`, `make boot-demo`, `tests/go/test_go_user_service.py` | live runtime | This is the clearest proof of the Rust-kernel plus Go-userspace identity. |
| Filesystem, package, and external package run path | `kernel_rs/src/lib.rs`, `tools/mkfs.py`, `tools/pkg_bootstrap_v1.py` | `tests/fs/*`, `tests/pkg/test_pkg_install_run.py`, `tests/pkg/test_pkg_external_apps.py` | mixed | Boot and package-run proofs are live; repo metadata tooling is support code. |
| Supported stock-Go lane | `services/go_std/`, `tools/build_go_std_spike.sh`, `tools/gostd_stock_builder/` | `make image-std`, `make boot-std`, `make smoke-std`, `tests/go/test_std_go_binary.py` | live runtime, supported non-default | This is a supported build and boot lane, but it is not the default repo story. |
| Compatibility, desktop, hardware, release, fleet, and similar qualification lanes | `tools/run_*`, `tests/*gate*`, contract docs in `docs/` | usually `make test-*` | deterministic qualification | Useful repo discipline. Do not read these as proof that a correspondingly large runtime tree exists. |
| Historical baseline | `legacy/` | `make -C legacy test-qemu` | live runtime, archival | Kept for comparison and regression context. |

## Live Runtime Proof Paths

Recommended direct proofs:
- kernel-only boot: `make image-kernel` then `make boot-kernel`
- default product demo: `make image-demo` then `make boot-demo`
- default product smoke without pytest: `make smoke-demo`
- supported stock-Go lane: `make image-std` then `make boot-std`
- supported stock-Go smoke without pytest: `make smoke-std`

Representative direct runtime tests:
- `tests/boot/test_boot_banner.py`
- `tests/go/test_go_user_service.py`
- `tests/go/test_std_go_binary.py`
- `tests/runtime/test_runtime_stress_v1.py`
- `tests/pkg/test_pkg_external_apps.py`

## Qualification And Modeling

These patterns are valid, but they are not the same as runtime source depth:

- seeded report generators such as `tools/run_app_compat_matrix_v3.py`
- deterministic desktop and GUI report generators such as
  `tools/run_gui_runtime_v1.py`
- simulated reliability and hardware evidence such as
  `tools/run_kernel_soak_v1.py` and `tools/run_hw_matrix_v6.py`
- aggregate gate tests that mainly verify file presence, wiring, and report
  schemas

Read them as:
- repo-quality controls
- release/qualification scaffolding
- visibility into declared thresholds

Do not read them as:
- proof of a broad implementation tree for each named subsystem
- proof that every milestone label corresponds to a large live runtime surface

## Non-Source Paths

These paths should not be read as architecture:
- `kernel_rs/target/`
- `out/`

They are build output.
