# Runtime ABI Stability Policy v2

Date: 2026-03-09  
Milestone: M21 ABI + API Stability Program v3  
Policy ID: `rugo.runtime_abi_policy.v2`

## Scope

This policy governs syscall ABI stability for the M21+ release line and defines
obligations for preserving runtime/user-space compatibility across multiple
tagged releases.

## Stability window

Stability window: `2026-03-09` through `2027-12-31` (v3 line).

Within this window:

- syscall IDs in `docs/abi/syscall_v3.md` are frozen;
- argument meaning and deterministic failure behavior remain
  backward-compatible;
- removals are not allowed in `v3.x`;
- deprecation must follow `docs/runtime/deprecation_window_policy_v1.md`.

## Compatibility obligations

- Runtime-facing behavior must remain backward-compatible for at least three
  tagged releases within the active ABI line.
- New syscall behavior in `v3.x` is additive only and must not invalidate
  existing binaries.
- Any ABI break requires a major ABI-line bump and migration documentation.

## Change-control process

1. Propose the change in a design note under `docs/runtime/`.
2. Generate source-of-truth reports from:
   - `tools/extract_kernel_syscalls.py`
   - `tools/extract_go_std_syscalls.py`
3. Run `tools/check_abi_diff_v3.py` and capture the diff artifact.
4. Run `tools/check_syscall_compat_v3.py` with the kernel and userspace
   reports to verify policy obligations.
5. If breakage exists, provide:
   - major ABI-line bump plan,
   - migration document under `docs/abi/`,
   - compatibility exception rationale.
6. Land only when `make test-abi-stability-v3` is green.

## Release gate requirements

- `tests/runtime/test_abi_docs_v3.py`
- `tests/runtime/test_abi_window_v3.py`
- `tests/runtime/test_abi_diff_gate_v3.py`
- `tests/runtime/test_abi_source_truth_v3.py`
- `tests/compat/test_abi_compat_matrix_v3.py`
- `tests/runtime/test_abi_stability_gate_v3.py`

Local gate: `make test-abi-stability-v3`.

CI gate step: `ABI stability v3 gate`.
