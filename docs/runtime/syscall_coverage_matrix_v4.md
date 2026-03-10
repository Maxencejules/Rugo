# Runtime Syscall Coverage Matrix v4

Date: 2026-03-10  
Milestone: M41 Process + Readiness Compatibility Closure v1

This matrix tracks process/readiness compatibility commitments introduced for
`compat_profile_v5`.

## Coverage matrix

| Surface area | API / syscall | Contract source | Status | Evidence | Owner | Target |
|---|---|---|---|---|---|---|
| process lifecycle | `waitid` | `process_model_v4` | implemented | `tests/compat/test_process_model_v4.py` | `compat-owner` | M41 |
| process signaling | `sigprocmask` | `process_model_v4` | implemented | `tests/compat/test_process_model_v4.py` | `compat-owner` | M41 |
| process signaling | `sigpending` | `process_model_v4` | implemented | `tests/compat/test_process_model_v4.py` | `compat-owner` | M41 |
| readiness wait | `poll` | `readiness_io_model_v1` | implemented | `tests/compat/test_epoll_surface_v1.py` | `runtime-port-owner` | M41 |
| readiness wait | `pselect` | `readiness_io_model_v1` | implemented | `tests/compat/test_epoll_surface_v1.py` | `runtime-port-owner` | M41 |
| readiness wait | `ppoll` | `readiness_io_model_v1` | implemented | `tests/compat/test_epoll_surface_v1.py` | `runtime-port-owner` | M41 |
| readiness signaling | `eventfd` | `readiness_io_model_v1` | implemented (bounded) | `tests/compat/test_epoll_surface_v1.py` | `runtime-port-owner` | M41 |
| socket messaging | `sendmsg` | `compat_profile_v5` | implemented | `tests/compat/test_deferred_surface_behavior_v2.py` | `net-owner` | M41 |
| socket messaging | `recvmsg` | `compat_profile_v5` | implemented | `tests/compat/test_deferred_surface_behavior_v2.py` | `net-owner` | M41 |
| local sockets | `socketpair` | `compat_profile_v5` | implemented | `tests/compat/test_deferred_surface_behavior_v2.py` | `net-owner` | M41 |
| deferred process parity | `fork`, `clone` | `compat_profile_v5` | deferred, deterministic `ENOSYS` | `tests/compat/test_fork_clone_surface_v1.py` | `compat-owner` | M41 |
| deferred readiness APIs | `epoll`, `io_uring` | `compat_profile_v5` | deferred, deterministic `ENOSYS` | `tests/compat/test_epoll_surface_v1.py` | `compat-owner` | M41 |
| deferred containment | namespace/cgroup compatibility | `compat_profile_v5` | deferred, deterministic `ENOSYS` | `tests/compat/test_deferred_surface_behavior_v2.py` | `compat-owner` | M41 |
| deferred socket families | `AF_NETLINK` / raw packet parity | `compat_profile_v5` | deferred, deterministic `ENOSYS` | `tests/compat/test_deferred_surface_behavior_v2.py` | `net-owner` | M41 |
| process/readiness gate | `test-process-readiness-parity-v1` | `Makefile` | implemented | `tests/compat/test_process_readiness_gate_v1.py` | `release-owner` | M41 |
| posix sub-gate | `test-posix-gap-closure-v2` | `Makefile` | implemented | `tests/compat/test_posix_gap_closure_gate_v2.py` | `release-owner` | M41 |

## Deterministic deferred-surface policy

Rows marked `deferred` are explicit unsupported commitments. M41 policy
requires stable unsupported behavior for every run:

- syscall/API returns `-1`
- deterministic error contract is `ENOSYS`
- report rows remain stable and machine-readable

## Update policy

Coverage changes require updates to:

- this matrix
- `docs/abi/compat_profile_v5.md`
- `docs/abi/process_model_v4.md`
- `docs/abi/readiness_io_model_v1.md`
- `tests/compat/test_process_readiness_gate_v1.py`
- `tests/compat/test_posix_gap_closure_gate_v2.py`
