# M57 Execution Backlog (Secondary Architecture Shadow Bring-Up v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Establish an experimental multi-arch shadow lane for `aarch64` without
changing the x86-64 release floor or support-claim policy.

M57 source of truth remains:

- `docs/POST_G2_EXTENDED_MILESTONES.md`
- `docs/M53_EXECUTION_BACKLOG.md`
- `docs/M11_EXECUTION_BACKLOG.md`
- this backlog

## Current State Summary

- Runtime/toolchain maturity exists for the x86-64 Rugo target, and M53
  defines the native-driver baseline future arch work must not bypass.
- The current release floor is explicitly x86-64 only.
- There is no versioned contract for a shadow `aarch64` bring-up, arch-neutral
  syscall profile, or promotion policy that keeps multi-arch experimental.
- M57 must make the shadow lane explicit before any portability claims grow.

## Execution plan

- PR-1: multi-arch shadow contract freeze
- PR-2: aarch64 boot and ABI smoke baseline
- PR-3: shadow-gate wiring + closure

## Execution status

- PR-1: pending
- PR-2: pending
- PR-3: pending

## PR-1: Multi-arch Shadow Contract Freeze

### Objective

Define the arch-neutral policy and ABI boundary for a shadow `aarch64` target.

### Scope

- Add docs:
  - `docs/runtime/arch_port_contract_v1.md`
  - `docs/hw/multi_arch_support_policy_v1.md`
  - `docs/abi/arch_neutral_syscall_profile_v1.md`
- Add tests:
  - `tests/runtime/test_arch_port_docs_v1.py`

### Primary files

- `docs/runtime/arch_port_contract_v1.md`
- `docs/hw/multi_arch_support_policy_v1.md`
- `docs/abi/arch_neutral_syscall_profile_v1.md`
- `tests/runtime/test_arch_port_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/runtime/test_arch_port_docs_v1.py -v`

### Done criteria for PR-1

- Shadow multi-arch scope is explicit and versioned.
- The x86-64 release floor remains protected from implicit multi-arch support
  broadening.

## PR-2: AArch64 Boot and ABI Smoke Baseline

### Objective

Implement deterministic shadow evidence for boot, syscall profile, and Go
runtime smoke on `aarch64`.

### Scope

- Add tooling:
  - `tools/run_multiarch_shadow_v1.py`
- Add tests:
  - `tests/boot/test_aarch64_boot_smoke_v1.py`
  - `tests/runtime/test_arch_neutral_syscall_profile_v1.py`
  - `tests/go/test_go_port_shadow_v1.py`
  - `tests/runtime/test_multiarch_negative_paths_v1.py`

### Primary files

- `tools/run_multiarch_shadow_v1.py`
- `tests/boot/test_aarch64_boot_smoke_v1.py`
- `tests/runtime/test_arch_neutral_syscall_profile_v1.py`
- `tests/go/test_go_port_shadow_v1.py`
- `tests/runtime/test_multiarch_negative_paths_v1.py`

### Acceptance checks

- `python tools/run_multiarch_shadow_v1.py --out out/multiarch-shadow-v1.json`
- `python -m pytest tests/boot/test_aarch64_boot_smoke_v1.py tests/runtime/test_arch_neutral_syscall_profile_v1.py tests/go/test_go_port_shadow_v1.py tests/runtime/test_multiarch_negative_paths_v1.py -v`

### Done criteria for PR-2

- Shadow multi-arch artifacts are deterministic and machine-readable.
- `ARCH: aarch64 boot ok` and arch-neutral ABI markers are stable.
- Unsupported or deferred arch paths fail deterministically rather than
  widening the claim boundary.

## PR-3: Shadow Multi-arch Gate + Arch-neutral ABI Sub-gate

### Objective

Run the shadow architecture lane in local and CI without changing the main
release floor.

### Scope

- Add local gates:
  - `Makefile` target `test-multiarch-shadow-v1`
  - `Makefile` target `test-arch-neutral-abi-v1`
- Add CI steps:
  - `Multiarch shadow v1 gate`
  - `Arch neutral abi v1 gate`
- Add aggregate tests:
  - `tests/runtime/test_multiarch_shadow_gate_v1.py`
  - `tests/runtime/test_arch_neutral_abi_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/runtime/test_multiarch_shadow_gate_v1.py`
- `tests/runtime/test_arch_neutral_abi_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-multiarch-shadow-v1`
- `make test-arch-neutral-abi-v1`

### Done criteria for PR-3

- Multi-arch shadow and arch-neutral ABI sub-gates are required in local and CI
  shadow lanes.
- M57 can be marked done only when the shadow lane stays explicitly non-claiming
  and does not alter the x86-64 release floor.

## Non-goals for M57 backlog

- promoting `aarch64` to Tier 0 or Tier 1
- broad bare-metal board support beyond the shadow contract
- replacing x86-64-specific release gates with cross-arch assumptions
