# Rugo — Status

## What this repo contains

| Track | Language | Location | Role |
|-------|----------|----------|------|
| **Rugo** | Rust (no_std) | repo root | Default build. Production rewrite target. |
| **Legacy** | C + Go (gccgo) | `legacy/` | Completed reference baseline (read-only). |

Full milestone definitions and acceptance criteria live in
[MILESTONES.md](../MILESTONES.md).

## How to run

```bash
# Rugo (build + full QEMU smoke-test suite)
make test-qemu

# Legacy (build + QEMU smoke tests, 16 tests)
make -C legacy build && make -C legacy image && make -C legacy test-qemu

# Cross-platform via Docker
make docker-all          # Rugo only
make docker-legacy       # Legacy only (requires gccgo in Docker image)
```

## Status matrix

| Milestone | Legacy | Rugo | Evidence |
|-----------|--------|------|----------|
| **M0** Boot + serial | ✅ | ✅ | `tests/boot/test_boot_banner.py` (`RUGO: boot ok`, `RUGO: halt ok`), `tests/boot/test_panic_path.py` (`RUGO: panic code=`). Legacy: `legacy/tests/boot/test_boot_banner.py` |
| **M1** Paging + traps | ✅ | ✅ | Rugo: `tests/boot/test_paging_enabled.py` (`MM: paging=on`), `tests/trap/test_page_fault_report.py` (`PF: addr=0x...`), `tests/trap/test_idt_smoke.py` (`TRAP: ok`). Legacy: `legacy/tests/boot/test_paging_enabled.py`, `legacy/tests/trap/test_page_fault_report.py` |
| **M2** Scheduler + threads | ✅ | ✅ | Rugo: `tests/sched/test_timer_ticks.py` (`TICK: 100`), `tests/sched/test_two_threads.py` (interleaved `A`/`B`). Legacy: `legacy/tests/sched/test_timer_ticks.py`, `legacy/tests/sched/test_two_threads.py` |
| **M3** User mode + syscalls | ✅ | ✅ | Rugo: `tests/user/test_enter_user_mode.py` (`USER: hello`), `tests/user/test_syscall_roundtrip.py` (`SYSCALL: ok`), `tests/user/test_thread_exit.py` (`THREAD_EXIT: ok`), `tests/user/test_user_fault.py` (`USER: killed`). Legacy: `legacy/tests/user/` |
| **M4** IPC + shared memory | ✅ | ✅ | Rugo: `tests/ipc/test_ping_pong.py` (`PING: ok`, `PONG: ok`), `tests/ipc/test_shm_bulk.py` (`SHM: checksum ok`). Legacy: `legacy/tests/ipc/` |
| **M5** VirtIO block | ✅ | ✅ | Rugo: `tests/drivers/test_virtio_blk_identify.py` (`BLK: found virtio-blk`), `tests/drivers/test_virtio_blk_rw.py` (`BLK: rw ok`), `tests/drivers/test_virtio_blk_init_invariants.py` (`BLK: invariants ok`), `tests/drivers/test_virtio_blk_init_failure.py` (`BLK: init failed`). Legacy: `legacy/tests/drivers/` |
| **M6** Filesystem + pkg + shell | ✅ | ✅ | Rugo: `tests/fs/test_fsd_smoke.py` (`FSD: mount ok`), `tests/pkg/test_pkg_install_run.py` (`APP: hello world`). Legacy: `legacy/tests/fs/test_fsd_smoke.py`, `legacy/tests/pkg/test_pkg_install_run.py` |
| **M7** VirtIO net + UDP | ✅ | ✅ | Rugo: `tests/net/test_udp_echo.py` (`NET: udp echo`). Legacy: `legacy/tests/net/test_udp_echo.py` |
| **G0** Go kernel entry | ✅ | n/a | `legacy/tests/boot/test_go_entry.py` (`GO: kmain ok`). Legacy-only. Re-verified 2026-02-18 via Docker. |
| **G1** Go services (TinyGo) | n/a | ✅ | Rugo: `tests/go/test_go_user_service.py` (`GOUSR: ok`). TinyGo bare-metal x86_64. |
| **G2** Full Go port | n/a | ✅ | Rugo-only. Done: `tests/go/test_std_go_binary.py` on `go_std_test` stock-Go artifact path (`os-go-std.iso`, `GOSTD: ok`). |

✅ done &ensp; ◐ in progress (prep) &ensp; ⬜ not started &ensp; n/a not applicable

## Legacy verification

### G0 (Go kernel entry) — verified 2026-02-18

**Status: PASS.** All 16 legacy tests pass, including G0 (`test_go_entry`).
Verified via `make docker-legacy` (Docker, Ubuntu 24.04, gccgo).

**Test file:** `legacy/tests/boot/test_go_entry.py`

**Expected serial markers:**
- `GO: kmain ok` — Go kernel entry executed
- `KERNEL: boot ok` — M0 boot marker
- `KERNEL: halt ok` — M0 clean halt marker

### How to run Legacy G0

```bash
# Via Docker (recommended — no host toolchain needed)
make docker-legacy

# Native (requires nasm, gcc, gccgo, ld, objcopy, xorriso, qemu, pytest)
make -C legacy build && make -C legacy image && make -C legacy test-qemu

# G0 only (native)
make -C legacy build && make -C legacy image \
  && python3 -m pytest legacy/tests/boot/test_go_entry.py -v
```

### Fixes applied (2026-02-18)

1. **conftest REPO_ROOT path:** `legacy/tests/conftest.py` and
   `legacy/tests/net/test_udp_echo.py` computed `REPO_ROOT` as `legacy/`
   instead of the repo root. Since the legacy `Makefile` sets `OUT = ../out`,
   all build artifacts (ISO, disk images) live in the repo-root `out/`
   directory. Fixed by adding one extra `os.path.dirname()` call in each file.

2. **`docker-legacy` Makefile target:** Added to the root `Makefile` so legacy
   tests can run via Docker on any platform (Windows, macOS, CI) without
   installing gccgo, QEMU, or xorriso on the host.

## Current focus

M0–M7 and G1 are complete: boot, paging, traps, scheduler, user mode, syscalls,
IPC, shared memory, service registry, VirtIO block, filesystem, package manager,
shell, VirtIO net, UDP echo, and TinyGo user-space services are all functional
with passing QEMU integration tests.

G2 is complete as of 2026-03-04:
- `tools/build_go_std_spike.sh` now executes a stock-Go path
  (`go run ./tools/gostd_stock_builder/main.go`) and writes
  `out/gostd.bin` plus `out/gostd-contract.env`.
- `tests/go/test_std_go_binary.py` validates serial marker flow and stock-Go
  contract metadata (`GOOS=rugo`, `GOARCH=amd64`, `STOCK_GO_*` keys).
- `make image-go-std` remains the G2 artifact gate for `os-go-std.iso`.
- Execution and sequencing history remains in `docs/G2_EXECUTION_BACKLOG.md`.

M8 execution update (2026-03-04):
- PR-1 complete (contract-first deliverables):
  - `docs/abi/syscall_v1.md`
  - `docs/abi/compat_profile_v1.md`
  - `tests/compat/` initial suite scaffolding
- PR-2 complete (loader/process/fd core compatibility primitives):
  - `docs/abi/process_thread_model_v1.md`
  - `kernel_rs/src/lib.rs` v1 syscall additions (`sys_open/sys_read/sys_write/sys_close/sys_wait/sys_poll`)
  - executable compat checks:
    - `tests/compat/test_loader_contract.py`
    - `tests/compat/test_process_lifecycle.py`
    - `tests/compat/test_process_wait.py`
    - `tests/compat/test_fd_table.py`
- PR-3 complete (subset closure + package/bootstrap lane + CI gate):
  - executable time/signal/socket closure:
    - `tests/compat/test_time_signal_subset.py`
    - `tests/compat/test_socket_api_subset.py`
    - `tests/compat/test_posix_subset.py`
  - package/repository v1 bootstrap:
    - `docs/pkg/package_format_v1.md`
    - `tools/pkg_bootstrap_v1.py`
    - `tests/pkg/test_pkg_external_apps.py`
  - release-gating CI lane:
    - `.github/workflows/ci.yml` compatibility profile v1 gate step
- M8 is done.

Post-G2 planning and execution:
- Extended roadmap (M8-M14): `docs/POST_G2_EXTENDED_MILESTONES.md`
- Next execution backlog (M8): `docs/M8_EXECUTION_BACKLOG.md`


