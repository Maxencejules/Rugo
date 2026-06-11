# Preemptive Scheduling in the Default Lane Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** PIT-driven preemption of R4 user tasks in the default Go lane —
gap-analysis §2.2/§3.3 ("move the PIT/APIC path out of `sched_test`").

**Architecture:** The PIC/PIT helpers in `kernel_rs/src/sched.rs` lose their
`sched_test`-only gating (shared with `go_test`). The default-lane boot path
programs the PIT at 100 Hz, user tasks run with RFLAGS.IF=1 (new
`enter_ring3_preemptible`, and `r4_init_task` seeds RFLAGS=0x202 under
`go_test`), and IDT vector 32 lands in a new `r4_timer_preempt` that EOIs the
PIC and involuntarily switches the running user task to the next Ready task.
Unlike `r4_yield_and_switch`, preemption preserves RAX — the saved frame is
not a syscall return. Kernel code still runs with IF=0 (interrupt gates), so
there is no nesting and no kernel locking needed yet.

**Tech Stack:** Rust no_std kernel, 8259A PIC + PIT, QEMU marker tests.

**Key facts from source recon:**
- `enter_ring3_at` pushes RFLAGS `0x002` hard-coded (`arch_x86.rs:131-143`).
- `r4_init_task` seeds `saved_frame[19] = 0x02` (`lib.rs:2258`).
- `r4_yield_and_switch` clobbers `saved_frame[14]` (RAX) — do NOT reuse for
  preemption (`lib.rs:2390-2405`).
- `pic_init` already masks all lines except IRQ0 (`sched.rs:24-25`).
- trap vector 32 is `sched_test`-gated (`trap.rs:77-80`).
- The M3 lanes never remap the PIC, so user IF must stay 0 there (a pending
  legacy-vector tick would hit the double-fault handler).

---

### Task 1: Failing boot test

Create `tests/sched/test_preempt_default_lane_v1.py`:

```python
# Phase 2 acceptance: the DEFAULT lane preempts user tasks on the PIT timer.
# Live runtime evidence - serial markers from a normal Go-lane boot.


def _find_in_order(serial: str, markers: list[str]) -> None:
    pos = 0
    for marker in markers:
        found = serial.find(marker, pos)
        assert found != -1, f"marker not found in order: {marker}"
        pos = found + len(marker)


def test_default_lane_preempts(qemu_serial_go):
    out = qemu_serial_go.stdout
    _find_in_order(out, [
        "SCHED: preempt on hz=100",
        "GOINIT: start",
        "SCHED: preempt hit",
        "GOINIT: ready",
        "RUGO: halt ok",
    ])
    assert out.count("SCHED: preempt on hz=100") == 1
    assert out.count("SCHED: preempt hit") == 1
    assert "GOINIT: err" not in out
    assert "GOSVCM: err" not in out
```

Run `python -m pytest tests/sched/test_preempt_default_lane_v1.py -v` →
FAIL on `SCHED: preempt on hz=100`.

Note ordering: `SCHED: preempt hit` must come after `GOINIT: start` (first
preemption can only happen once user code runs) and before shutdown. If the
boot window turns out too short for a guaranteed hit before `GOINIT: ready`,
relax the in-order anchor to `RUGO: halt ok` only — but try the strict order
first; the shell session keeps user code running long enough.

### Task 2: Share the PIC/PIT helpers

`kernel_rs/src/sched.rs`: change the cfg on `PIC1_CMD..PIC2_DATA`,
`pic_init`, `pic_send_eoi`, `pit_init` from `#[cfg(feature = "sched_test")]`
to `#[cfg(any(feature = "sched_test", feature = "go_test"))]`, and make
`pic_send_eoi` `pub(crate)`.

### Task 3: Preemptible ring-3 entry

`kernel_rs/src/arch_x86.rs`: add next to `enter_ring3_at`:

```rust
    /// Like enter_ring3_at but with RFLAGS.IF set: the task can be
    /// preempted by the PIT. Only safe once the PIC is remapped+masked.
    pub(crate) unsafe fn enter_ring3_preemptible(code_va: u64, user_sp: u64) -> ! {
        core::arch::asm!(
            "push 0x1B",
            "push {stack}",
            "push 0x202",
            "push 0x23",
            "push {code}",
            "iretq",
            stack = in(reg) user_sp,
            code = in(reg) code_va,
            options(noreturn),
        );
    }
```

`kernel_rs/src/lib.rs` `r4_init_task` (line 2258): seed IF under go_test:

```rust
        #[cfg(feature = "go_test")]
        { R4_TASKS[tid].saved_frame[19] = 0x202; } // RFLAGS, preemptible
        #[cfg(not(feature = "go_test"))]
        { R4_TASKS[tid].saved_frame[19] = 0x02; }  // RFLAGS
```

### Task 4: Timer preemption path

`kernel_rs/src/lib.rs`, inside the cfg_r4 block next to
`r4_yield_and_switch` (preserves RAX, counts the event):

```rust
    #[cfg(feature = "go_test")]
    static mut R4_PREEMPT_TICKS: u64 = 0;
    #[cfg(feature = "go_test")]
    static mut R4_PREEMPT_COUNT: u64 = 0;

    /// PIT tick entry: EOI first, then preempt if we interrupted ring 3
    /// and another task is Ready. RAX is preserved - this is not a syscall.
    #[cfg(feature = "go_test")]
    pub(crate) unsafe fn r4_timer_preempt(frame: *mut u64) {
        R4_PREEMPT_TICKS += 1;
        sched::pic_send_eoi(0);
        if *frame.add(18) & 3 != 3 {
            return; // interrupted kernel init path - nothing to switch
        }
        if R4_NUM_TASKS == 0 {
            return;
        }
        let cur = R4_CURRENT;
        if let Some(tid) = r4_find_ready(cur) {
            r4_save_frame(frame, cur);
            R4_TASKS[cur].state = R4State::Ready;
            R4_PREEMPT_COUNT += 1;
            if R4_PREEMPT_COUNT == 1 {
                serial_write(b"SCHED: preempt hit\n");
            }
            r4_switch_to(frame, tid);
        }
    }
```

`kernel_rs/src/trap.rs`: replace the vector-32 arm with:

```rust
            #[cfg(feature = "sched_test")]
            32 => {
                crate::sched::handle_timer_irq();
            }
            #[cfg(all(feature = "go_test", not(feature = "sched_test")))]
            32 => {
                crate::r4_timer_preempt(frame);
            }
```

### Task 5: Default-lane boot wiring

`kernel_rs/src/lib.rs` go_test kmain branch (lines ~5510-5555): after
`r4_init_task(0, ...)` / before entering ring 3:

```rust
        sched::pic_init();
        sched::pit_init(100);
        serial_write(b"SCHED: preempt on hz=100\n");
```

and change the final `enter_ring3_at(USER_CODE_VA, USER_STACK_TOP)` in this
branch to `enter_ring3_preemptible(USER_CODE_VA, USER_STACK_TOP)`.

### Task 6: Build, test, regress, commit

1. `mingw32-make image-go` (also rebuild `image` to confirm kernel-only lane
   unaffected).
2. `python -m pytest tests/sched/test_preempt_default_lane_v1.py -v` → PASS.
3. Regression: `python -m pytest tests/mm tests/go tests/runtime/test_service_boot_runtime_v2.py tests/runtime/test_process_scheduler_runtime_v2.py tests/pkg/test_default_shell_app_runtime_v1.py tests/runtime/test_service_control_runtime_v1.py -v`
   — service lifecycle must tolerate involuntary interleaving (it already
   tolerates arbitrary yields; sys_debug_write lines stay atomic because
   interrupt gates keep IF=0 in kernel).
4. `mingw32-make image-sched && python -m pytest tests/sched -v` (old
   sched_test lane still green).
5. Docs: `docs/runtime/scheduler_v1.md` marker contract + SOURCE_MAP row +
   README proof line; Makefile `test-sched-preempt-v1` target.
6. Commit: `feat: preempt default-lane user tasks on the PIT timer`.

## Self-Review Notes
- RAX preservation on preempt is the correctness-critical difference from
  the yield path.
- MSI-X vectors 64/65 can now fire from ring 3; `runtime::native::handle_irq`
  already covers them in the go lane.
- `sys_time_now` stays a per-call monotonic counter for now; real tick-based
  time is a later phase (gap §2.10).
