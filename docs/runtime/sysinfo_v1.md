# sysinfo metrics — contract v1

Status: boot-verified via `make test-sysinfo-v1`
Source: `kernel_rs/src/lib.rs` (`sys_sysinfo`), `kernel_rs/src/mm.rs`
(`free_frames`), `apps/coreutils/sysinfoprobe.asm`.
Proof: `tests/runtime/test_sysinfo_v1.py`.

Full-OS implementation guide Part V (observability / `/proc`-style
metrics), the syscall-based slice. A file-backed `/proc/<tid>/stat` is a
later slice; this exposes the same core numbers without the pseudo-fs
formatting machinery.

## ABI

`sys_sysinfo` — ABI v3.2 id **61**: `rdi` = op.

| op | returns |
|----|---------|
| 1 | live (non-Dead) task count |
| 2 | free physical frames (PMM) |
| 3 | uptime in PIT ticks (100 Hz) |

Other ops return -1.

## v1 boundary / carry-forward

- No per-task `/proc/<tid>/stat` file yet (needs the pseudo-fs factory and
  decimal formatting).
- Frame count is whole-system; per-task RSS accounting is carry-forward.

## Acceptance

`make test-sysinfo-v1`: `probe sysinfoprobe` reads a non-zero task count,
non-zero free frames, and an uptime that strictly advances across a busy
interval, printing `SYSINFOPROBE: ok`.
