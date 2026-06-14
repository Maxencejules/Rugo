# clock_gettime — contract v1

Status: boot-verified via `make test-clock-v1`
Source: `kernel_rs/src/lib.rs` (`sys_time`, `cmos_read`, `cmos_unix_seconds`,
`days_from_civil`), `apps/coreutils/timeprobe.asm`.
Proof: `tests/runtime/test_clock_v1.py`.

Full-OS implementation guide Part IV.9 (time/timekeeping). Standalone — no
prerequisites.

## ABI

`sys_time` — ABI v3.2 id **53**, op-multiplexed:

| op | call | args | returns |
|----|------|------|---------|
| 1 | clock_gettime | `rsi` = clockid | value, or -1 |

clockids:

- **0 = CLOCK_MONOTONIC** — nanoseconds since boot, derived from the PIT
  tick counter (`R4_PREEMPT_TICKS`, 100 Hz → 10 ms/tick). Strictly
  non-decreasing; resolution is 10 ms.
- **1 = CLOCK_REALTIME** — seconds since the Unix epoch, read from the CMOS
  RTC (ports 0x70/0x71). BCD vs binary is taken from status register B;
  24-hour mode is assumed (QEMU default). Civil date → days is computed with
  Howard Hinnant's algorithm.

The value is returned directly in `rax` (not via a `timespec` pointer) for
v1 simplicity.

## Markers

The probe emits `TIMEPROBE: monotonic ok` / `TIMEPROBE: realtime ok`. The
kernel adds no per-call marker (the existing `sys_time_now`, id 10, is
unchanged).

## v1 boundary / carry-forward

- **No `timespec`/`timeval` struct out.** A pointer-based variant is
  carry-forward.
- **`nanosleep` is implemented** (op 2, `rsi` = nanoseconds): the caller
  blocks (10 ms PIT resolution) and OTHER tasks run meanwhile; if nothing
  else is runnable the scheduler parks in a ring0 idle loop
  (`r4_idle_loop`, interrupts enabled) and the PIT (`r4_wake_sleepers`)
  wakes the sleeper at its deadline — no spinning, no false deadlock. This
  idle/wake path is the shared wait-queue prerequisite. Proof:
  `make test-nanosleep-v1`.
- **`timerfd` is implemented** (op 3 = timerfd_create(`rsi` = nanoseconds) →
  fd). A one-shot `TimerFd` becomes readable at its deadline; a non-blocking
  `read` returns the 8-byte expiration count (1) once fired and 0 before, and
  `poll` reports `POLLIN` once expired. Proof: `make test-timerfd-v1`.
  (Periodic/interval timers and blocking timerfd reads are carry-forward.)
- **RTC assumptions:** 24-hour mode, century = 2000+. A dead CMOS battery or
  12-hour mode would skew REALTIME; MONOTONIC is unaffected.
- `sys_time_now` (id 10) keeps its existing per-call counter semantics; this
  adds a real clock alongside it rather than changing it.

## Acceptance

`make test-clock-v1`: `probe timeprobe` reads MONOTONIC across a busy
interval and asserts it advanced (the PIT preempts the loop and ticks the
clock), then reads REALTIME and asserts it is after 2023-11
(`> 1_700_000_000`), proving the CMOS path decodes a sane wall clock.
