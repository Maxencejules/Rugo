# Thread-local storage (%fs base) — contract v1

Status: boot-verified via `make test-tls-v1` (go C4 runtime lane)
Source: `kernel_rs/src/lib.rs` (`sys_vm_ctl` op 5, `R4Task.fs_base`,
`r4_switch_to` restore, `r4_init_task` reset), dispatch in
`kernel_rs/src/syscall.rs` (id 50); `apps/coreutils/tlsprobe.asm`.
ABI: `sys_vm_ctl` (id 50) op 5.
Proof: `tests/runtime/test_tls_v1.py`.

Full-OS implementation guide Part I (process model), thread-local storage: each
thread reaches its own private storage through the `%fs` segment base, the
mechanism a C runtime uses for `__thread` / `errno`-per-thread.

## Behaviour

Each task carries an `fs_base` (the `%fs` segment base, MSR `IA32_FS_BASE`
`0xC000_0100`). `sys_vm_ctl op 5 = set_tls(base)`:

- rejects a non-canonical / kernel-half base (`>= 0x0000_8000_0000_0000`) so the
  restore `wrmsr` can never `#GP`;
- stores `base` in the task and applies it immediately (`wrmsr`), so `%fs:offset`
  in ring 3 reaches `base + offset`.

`r4_switch_to` writes `IA32_FS_BASE = task.fs_base` on **every** resume (0 for a
non-TLS task), so:

- each task's `%fs` base is **isolated** — a clone thread sets up its own TLS and
  is never affected by another task's base (which is restored when that task runs);
- `fs_base` is **not inherited** (`r4_init_task` resets it to 0): POSIX TLS is
  per-thread, so a new thread starts with no TLS and installs its own.

`fs_base` is independent of the GP register file (it is an MSR, not part of the
22-qword `saved_frame`), and of `pid_ns` / `isolation_domain`.

## Acceptance

`make test-tls-v1`: `tlsprobe` calls `set_tls(&tls_buf)`, writes a magic word via
`[fs:0]`, and reads `tls_buf[0]` **directly** — they match, proving `%fs:0` aliases
the base. It then `yield`s (running other tasks whose `fs.base` is 0) and re-reads
`[fs:0]`: the magic is still there, which only holds if the kernel **restored this
task's `fs.base` on resume**. Prints `TLS: fs-base tls ok` with no `TLS: FAIL` /
`USERPF`.

## v1 boundary / carry-forward

- **Kernel mechanism only.** The `%fs`-base primitive + per-task restore are done;
  wiring it into rlibc (a `__thread errno`, a per-thread TLS block set up at thread
  start, the `__tls_get_addr` / initial-exec model) is the libc-side follow-up —
  and the PE→ELF C toolchain's `.tdata`/`.tbss` handling is the same blocker that
  gates C-side dynamic features (see `libc_v1.md`).
- **No `%gs` TLS / TCB self-pointer.** Only `%fs` is provided; a `%gs`-based TCB
  and a self-pointer at `%fs:0` (the glibc layout) are carry-forward.
