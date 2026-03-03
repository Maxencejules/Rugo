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
| **M3** User mode + syscalls | ✅ | ✅ | Rugo: `tests/user/test_enter_user_mode.py` (`USER: hello`), `tests/user/test_syscall_roundtrip.py` (`SYSCALL: ok`), `tests/user/test_user_fault.py` (`USER: killed`). Legacy: `legacy/tests/user/` |
| **M4** IPC + shared memory | ✅ | ✅ | Rugo: `tests/ipc/test_ping_pong.py` (`PING: ok`, `PONG: ok`), `tests/ipc/test_shm_bulk.py` (`SHM: checksum ok`). Legacy: `legacy/tests/ipc/` |
| **M5** VirtIO block | ✅ | ✅ | Rugo: `tests/drivers/test_virtio_blk_identify.py` (`BLK: found virtio-blk`), `tests/drivers/test_virtio_blk_rw.py` (`BLK: rw ok`), `tests/drivers/test_virtio_blk_init_invariants.py` (`BLK: invariants ok`), `tests/drivers/test_virtio_blk_init_failure.py` (`BLK: init failed`). Legacy: `legacy/tests/drivers/` |
| **M6** Filesystem + pkg + shell | ✅ | ✅ | Rugo: `tests/fs/test_fsd_smoke.py` (`FSD: mount ok`), `tests/pkg/test_pkg_install_run.py` (`APP: hello world`). Legacy: `legacy/tests/fs/test_fsd_smoke.py`, `legacy/tests/pkg/test_pkg_install_run.py` |
| **M7** VirtIO net + UDP | ✅ | ✅ | Rugo: `tests/net/test_udp_echo.py` (`NET: udp echo`). Legacy: `legacy/tests/net/test_udp_echo.py` |
| **G0** Go kernel entry | ✅ | n/a | `legacy/tests/boot/test_go_entry.py` (`GO: kmain ok`). Legacy-only. Re-verified 2026-02-18 via Docker. |
| **G1** Go services (TinyGo) | n/a | ✅ | Rugo: `tests/go/test_go_user_service.py` (`GOUSR: ok`). TinyGo bare-metal x86_64. |
| **G2** Full Go port | n/a | ◐ | Rugo-only. In progress (prep): `tests/go/test_std_go_binary.py` scaffold for `GOSTD: ok`. Long-term. |

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


