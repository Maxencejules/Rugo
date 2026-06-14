// TTY line discipline — canonical (cooked) mode (full-os guide Part V.11).
// Cooks a raw input byte stream into lines: printable characters are buffered and
// echoed; backspace/DEL erases the last buffered character (echoing the standard
// "\b \b" so a terminal visually rubs it out); a newline terminates the line and
// makes the whole line available to a reader. This is the discipline a TTY/pty
// layers between the wire and an application's line-oriented read.
//
// v1 boundary: the cooking core + a self-test. Wiring it onto the pty slave's
// read path (raw vs canonical via an ioctl) is carry-forward — the same
// mechanism-before-wiring staging the DMA pool / block cache slices use.

#![allow(dead_code)]

use crate::serial_write;

const LINE_MAX: usize = 256;
const ECHO_MAX: usize = 512;

pub struct LineDiscipline {
    line: [u8; LINE_MAX], // the cooked line being assembled
    len: usize,
    ready: bool,        // a complete (newline-terminated) line is available
    echo: [u8; ECHO_MAX], // bytes to echo back to the terminal
    echo_len: usize,
}

impl LineDiscipline {
    pub const fn new() -> Self {
        Self { line: [0; LINE_MAX], len: 0, ready: false, echo: [0; ECHO_MAX], echo_len: 0 }
    }

    fn echo_push(&mut self, b: u8) {
        if self.echo_len < ECHO_MAX {
            self.echo[self.echo_len] = b;
            self.echo_len += 1;
        }
    }

    /// Feed one raw input byte through the canonical-mode discipline.
    pub fn input(&mut self, byte: u8) {
        match byte {
            0x08 | 0x7F => {
                // Backspace / DEL: erase the last buffered char, echo "\b \b".
                if self.len > 0 {
                    self.len -= 1;
                    self.echo_push(0x08);
                    self.echo_push(b' ');
                    self.echo_push(0x08);
                }
            }
            b'\n' | b'\r' => {
                // Line terminator: append '\n', mark ready, echo a newline.
                if self.len < LINE_MAX {
                    self.line[self.len] = b'\n';
                    self.len += 1;
                }
                self.ready = true;
                self.echo_push(b'\n');
            }
            c if (0x20..0x7F).contains(&c) => {
                // Printable: buffer + echo (leave room for the terminating '\n').
                if self.len < LINE_MAX - 1 {
                    self.line[self.len] = c;
                    self.len += 1;
                    self.echo_push(c);
                }
            }
            _ => {} // other control bytes are ignored in this v1
        }
    }

    pub fn line_ready(&self) -> bool {
        self.ready
    }

    /// The cooked line (valid once `line_ready()`); includes the terminating '\n'.
    pub fn line(&self) -> &[u8] {
        &self.line[..self.len]
    }

    pub fn echo(&self) -> &[u8] {
        &self.echo[..self.echo_len]
    }
}

/// Boot self-test (full-os guide Part V.11): feed "ab\x08c\n" and confirm the
/// cooked line is "ac\n" (backspace erased the 'b') and the echo stream is
/// "ab\b \bc\n" (each printable echoed, backspace rubbed out with "\b \b").
/// Emits `TTY: line discipline ok` / `fail`.
pub fn tty_selftest() -> u64 {
    let mut ld = LineDiscipline::new();
    for &b in b"ab\x08c\n" {
        ld.input(b);
    }
    let line_ok = ld.line_ready() && ld.line() == b"ac\n";
    let echo_ok = ld.echo() == [b'a', b'b', 0x08, b' ', 0x08, b'c', b'\n'];
    if line_ok && echo_ok {
        serial_write(b"TTY: line discipline ok\n");
        1
    } else {
        serial_write(b"TTY: line discipline fail\n");
        0
    }
}
