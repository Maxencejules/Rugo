# TTY line discipline (canonical mode) — contract v1

Status: boot-verified via `make test-tty-v1`
Source: `kernel_rs/src/tty.rs` (`LineDiscipline`, `tty_selftest`).
Proof: `tests/runtime/test_tty_v1.py`.

Full-OS guide Part V.11 (userspace), TTY: cook a raw input byte stream into lines
— the canonical-mode discipline a terminal/pty puts between the wire and an
application's line-oriented read.

## Behaviour

`LineDiscipline::input(byte)` processes one raw input byte:

- **printable** (`0x20..0x7E`): appended to the line buffer and echoed;
- **backspace / DEL** (`0x08` / `0x7F`): erases the last buffered character and
  echoes the standard `"\b \b"` (back up, overwrite with space, back up — the
  terminal visually rubs the character out);
- **newline / CR** (`\n` / `\r`): appends `\n`, marks the line ready, echoes a
  newline.

`line_ready()` / `line()` expose the cooked line to a reader; `echo()` is the
byte stream to send back to the terminal.

## Acceptance

`make test-tty-v1`: the boot self-test feeds `"ab\x08c\n"` and confirms the cooked
line is `"ac\n"` (the backspace erased the `b`) and the echo stream is
`"ab\b \bc\n"` — `TTY: line discipline ok`, with no `TTY: line discipline fail`.

## v1 boundary / carry-forward

- The cooking core + self-test. Wiring it onto the pty slave's read path (a
  raw-vs-canonical mode flag toggled via `termios`/ioctl, blocking reads that
  return on a completed line, the echo routed back to the master) is
  carry-forward — the same mechanism-before-wiring staging the DMA pool / block
  cache slices use ([`pty_v1.md`](pty_v1.md) is the underlying pty).
- No special characters beyond backspace/newline (no `^C`/`^D`/`^U`/`^W`, no tab
  expansion); no job control.
