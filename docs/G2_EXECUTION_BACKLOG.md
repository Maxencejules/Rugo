# G2 Execution Backlog (Track B)

Date: 2026-03-03  
Lane: Rugo (Rust kernel + Go user space)

## Goal

Drive G2 from current prep/spike state to milestone completion:

- Standard Go toolchain produces binaries for `GOOS=rugo` + `GOARCH=amd64`.
- Go runtime syscall + threading model works on Rugo.
- Standard Go hello runs and prints `GOSTD: ok`.

G2 source of truth remains `MILESTONES.md` and `docs/abi/*`.

## Current State Summary

- Spike path exists (`go_std_test`, `make image-go-std`, `tests/go/test_std_go_binary.py`).
- Spike bridges in place: debug write, time, yield, thread-exit.
- Kernel still has non-final runtime primitives (`sys_thread_spawn`, `sys_vm_map`, `sys_vm_unmap`) that block full runtime/toolchain port.
- Build path still uses TinyGo compatibility bridge, not stock Go output for `GOOS=rugo`.

## PR-1: Kernel Runtime Primitives for G2

### Objective

Implement real v0 behavior for runtime-critical syscalls so Go runtime does not depend on stubs.

### Scope

- Implement runnable user-thread creation for `sys_thread_spawn` in M3/G2 path.
- Implement `sys_vm_map` and `sys_vm_unmap` with validated user mappings and deterministic errors.
- Keep ABI numbers and calling convention unchanged (`docs/abi/syscall_v0.md`).

### Primary files

- `kernel_rs/src/lib.rs`
- `docs/abi/syscall_v0.md`
- `docs/abi/process_thread_model_v0.md`
- New tests under `tests/user/` and `tests/go/`

### Acceptance checks

- Add tests for:
  - thread spawn success + join/exit marker behavior
  - invalid spawn entry rejection
  - vm_map/vm_unmap success path and bad-arg rejection
- Existing tests continue passing:
  - `tests/user/test_thread_exit.py`
  - `tests/go/test_std_go_binary.py`

### Done criteria for PR-1

- `sys_thread_spawn`, `sys_vm_map`, `sys_vm_unmap` are no longer stub semantics for G2 path.
- ABI/process docs updated and aligned with implementation.

## PR-2: Go Runtime Bridge Hardening (No TinyGo-only Assumptions)

### Objective

Move from minimal spike glue to runtime-oriented bridge behavior suitable for standard Go runtime integration.

### Scope

- Expand G2 bridge layer to include thread spawn + vm map/unmap callouts.
- Refine startup/runtime stubs split (`rt0`, syscall bridge, runtime stubs) into stable interfaces.
- Add deterministic marker flow for runtime primitives (thread + vm + time/yield baseline).

### Primary files

- `services/go_std/rt0.asm`
- `services/go_std/syscalls.asm`
- `services/go_std/runtime_stubs.asm`
- `services/go_std/main.go`
- `docs/abi/go_port_spike_v0.md`
- `tests/go/test_std_go_binary.py` (and any new `tests/go/*`)

### Acceptance checks

- `make image-go-std`
- `python -m pytest tests/go/test_std_go_binary.py -v`
- New go_std tests covering runtime-primitive bridges (thread/vm) pass.

### Done criteria for PR-2

- G2 bridge covers runtime-critical syscall set used by next-stage toolchain/runtime work.
- Docs clearly separate "implemented now" vs "still planned".

## PR-3: Stock-Go Toolchain Path + Final G2 Gate

### Objective

Introduce and verify standard Go toolchain output path for `GOOS=rugo`/`GOARCH=amd64` and close milestone.

### Scope

- Add initial stock-Go target/toolchain integration (build script + artifact contract).
- Replace TinyGo compatibility dependency in G2 acceptance path.
- Ensure `test_std_go_binary` exercises stock-Go-produced binary.
- Update milestone/status docs from prep to done when all criteria are met.

### Primary files

- `tools/build_go_std_spike.sh` (or successor script)
- `Makefile` (new/updated build target for stock-Go G2 path)
- `services/go_std/` (or successor standard-Go sample path)
- `tests/go/test_std_go_binary.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make image-go-std` (stock-Go artifact path)
- `python -m pytest tests/go/test_std_go_binary.py -v`
- Full regression sanity:
  - `python -m pytest tests/go/test_go_user_service.py tests/go/test_std_go_binary.py -v`

### Done criteria for PR-3

- G2 DoD in `MILESTONES.md` is fully satisfied.
- Status docs mark G2 as `done`.

## Sequencing Notes

- PR-1 first: kernel primitives unblock everything else.
- PR-2 second: bridge/runtime hardening before toolchain switch.
- PR-3 last: toolchain integration + final milestone closure.

## Non-goals for this backlog

- Re-architecting all kernel scheduling beyond what runtime integration requires for G2.
- Expanding unrelated milestone scope (M0-M7 are already complete).
