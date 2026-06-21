# pty (pseudo-terminal) pair — contract v1

Status: boot-verified via `make test-pty-v1`
Source: `kernel_rs/src/lib.rs` (`M8FdKind::PtyMaster`/`PtySlave`, `PtyObj`,
`PTYS`, `pty_drop_end`, `sys_ioctl` op 2, `sys_read_v1`/`sys_write_v1`/
`sys_poll_v1` pty arms), `apps/coreutils/ptyprobe.asm`.
Proof: `tests/runtime/test_pty_v1.py`.

Full-OS implementation guide Part V.11 (TTY/pty + job control) — the
pseudo-terminal byte channel that a terminal/job-control layer builds on.

## Behaviour

`sys_ioctl` (id 56) **op 2** = openpty: allocates a `PtyObj` and a master/slave
fd pair, returning `(slave_fd << 32) | master_fd`. The object holds two 512-byte
rings:

- bytes written to the **master** are readable from the **slave** (`m2s`);
- bytes written to the **slave** are readable from the **master** (`s2m`).

Both ends carry `READ | WRITE | POLL` rights. `poll` reports `POLLIN` when the
end's read ring is non-empty and `POLLOUT` always (space permitting). On
`close`, `pty_drop_end` releases the end; the `PtyObj` is recycled once both
ends are closed. Up to `PTY_MAX` (2) pairs exist at once.

## v1 boundary / carry-forward

- **No line discipline.** Raw byte channel only — no canonical/cooked mode, no
  echo, no `Ctrl-C`→`SIGINT`, no `Ctrl-D` EOF, no `ERASE`/`KILL` editing.
- **No job control.** No controlling terminal, foreground process group, or
  `SIGTTIN`/`SIGTTOU`. These are the carry-forward that makes this a *terminal*.
- **Non-blocking reads.** An empty read returns 0 immediately (v1) rather than
  blocking until data arrives or all writers close.
- **No `winsize`/termios ioctls** (`TIOCGWINSZ`, `TCGETS`, …).
- Ring capacity is 512 bytes per direction; a write that would overflow fails
  (caller retries). A single write is already capped at 256 bytes by
  `sys_write_v1`.

## Acceptance

`make test-pty-v1`: `ptyprobe` opens a pty, writes `ptyhello` to the master and
reads it back from the slave, then writes `ptyback!` to the slave and reads it
back from the master, echoing each read. The transcript shows `ptyhello` then
`ptyback!` then `PTYPROBE: ok`, proving bidirectional delivery.
