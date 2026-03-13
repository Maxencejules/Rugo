# Go Port Spike v0 (G2)

## Purpose

Define the G2 artifact/runtime contract used to qualify the supported
stock-Go lane:
- lock the target contract (`GOOS=rugo`, `GOARCH=amd64`),
- keep deterministic runtime marker behavior for the syscall/thread/vm bridge,
- validate that the acceptance path is driven by a stock-Go toolchain step.

## Target contract

- Contract GOOS: `rugo`
- Contract GOARCH: `amd64`

Artifact contract file:
- `out/gostd-contract.env`
- includes:
  - `GOOS=rugo`
  - `GOARCH=amd64`
  - `STOCK_GO_VERSION=...`
  - `STOCK_GO_HOST_GOOS=...`
  - `STOCK_GO_HOST_GOARCH=...`

Build path:
- `make build-go-std` runs the supported bootstrap path:
  - `bash tools/bootstrap_go_port_v1.sh --rebuild`
- the bootstrap path rebuilds through:
  - `tools/build_go_std_spike.sh`
  - `go run ./tools/gostd_stock_builder/main.go`
- `tools/gostd_stock_builder/main.go` writes:
  - `out/gostd.bin`
  - `out/gostd-contract.env`
- `tools/runtime_toolchain_contract_v1.py` writes:
  - `out/runtime-toolchain-contract.env`
- `tests/go/test_std_go_binary.py` checks both serial markers and contract
  metadata to keep the stock-Go gate explicit.

## Minimal binary target

- Runtime marker source lane: `services/go_std/`
- Build script: `tools/build_go_std_spike.sh`
- Flat binary output: `out/gostd.bin`
- Interface source report: `out/gostd-syscall-interface.json`
- Kernel feature: `go_std_test`
- ISO target: `os-go-std.iso`
- Acceptance path: `tests/go/test_std_go_binary.py` (marker set: `GOSTD: ok`,
  `GOSTD: time ok`, `GOSTD: yield ok`, `GOSTD: vm ok`,
  `GOSTD: spawn child ok`, `GOSTD: spawn main ok`, `THREAD_EXIT: ok`)

## Runtime hooks and glue points

### Implemented in go_std lane

Entry and syscall glue:
- `services/go_std/rt0.asm`: `_start` -> calls `main`, then halts.
- `services/go_std/syscalls.asm`: syscall wrappers.
- `services/go_std/start.asm`: thin aggregator including the split assembly files.
- `main.sysDebugWrite` -> syscall `0` (`sys_debug_write`) via `int 0x80`.
- `main.sysThreadSpawn` -> syscall `1` (`sys_thread_spawn`) via `int 0x80`.
- `main.sysThreadExit` -> syscall `2` (`sys_thread_exit`) via `int 0x80`.
- `main.sysYield` -> syscall `3` (`sys_yield`) via `int 0x80`.
- `main.sysVmMap` -> syscall `4` (`sys_vm_map`) via `int 0x80`.
- `main.sysVmUnmap` -> syscall `5` (`sys_vm_unmap`) via `int 0x80`.
- `main.sysTimeNow` -> syscall `10` (`sys_time_now`) via `int 0x80`.
- `main.sysSpawnEntry` -> helper symbol returning a user entry trampoline
  (`gostd_spawn_entry`) for spawn marker validation.

Runtime/libc stubs in the lane:
`services/go_std/runtime_stubs.asm`
- `runtime.alloc` (simple bump allocator)
- `getrandom`
- `tinygo_register_fatal_signals`
- `memset`, `memcpy`, `memmove`, `write`, `abort`, `exit`, `_exit`

### Still planned beyond v0 gate

Runtime/toolchain integration beyond this G2 closure:
- startup/rt0 glue for a native rugo target (`rt0` + runtime init),
- full Go runtime scheduler/thread model integration,
- runtime memory-management integration beyond marker probes,
- full timer/clock integration for runtime behavior,
- optional service hooks over IPC (`sys_ipc_*`, `sys_svc_*`) for stdlib work.

## Syscall bridge map (v0)

| Go-facing bridge | ASM symbol | Syscall nr | Kernel ABI symbol |
|------------------|------------|------------|-------------------|
| debug write | `main.sysDebugWrite` | `0` | `sys_debug_write` |
| thread spawn | `main.sysThreadSpawn` | `1` | `sys_thread_spawn` |
| thread exit | `main.sysThreadExit` | `2` | `sys_thread_exit` |
| yield | `main.sysYield` | `3` | `sys_yield` |
| vm map | `main.sysVmMap` | `4` | `sys_vm_map` |
| vm unmap | `main.sysVmUnmap` | `5` | `sys_vm_unmap` |
| time now | `main.sysTimeNow` | `10` | `sys_time_now` |
| blk read (planned) | `main.sysBlkRead` | `13` | `sys_blk_read` |
| blk write (planned) | `main.sysBlkWrite` | `14` | `sys_blk_write` |
| net send (planned) | `main.sysNetSend` | `15` | `sys_net_send` |
| net recv (planned) | `main.sysNetRecv` | `16` | `sys_net_recv` |

Notes:
- Register convention remains as defined in `docs/abi/syscall_v0.md`.
- Return `-1` (`u64` all ones) is the uniform error contract.
- `main.sysSpawnEntry` is a local helper for the spawn-marker trampoline and is
  not a syscall bridge.

## Status

- G1 (`go_test`) remains the stable TinyGo bringup lane.
- G2 (`go_std_test`) now has a stock-Go build gate and is marked `done` in
  `MILESTONES.md` and `docs/STATUS.md`.
