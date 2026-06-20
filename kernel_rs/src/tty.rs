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
    intr: bool,         // Ctrl-C seen: a reader should deliver SIGINT and discard the line
    eof: bool,          // Ctrl-D on an empty line: a reader sees end-of-input
}

impl LineDiscipline {
    pub const fn new() -> Self {
        Self {
            line: [0; LINE_MAX],
            len: 0,
            ready: false,
            echo: [0; ECHO_MAX],
            echo_len: 0,
            intr: false,
            eof: false,
        }
    }

    /// Erase the last buffered char and echo the standard "\b \b" rub-out.
    fn erase_one(&mut self) {
        if self.len > 0 {
            self.len -= 1;
            self.echo_push(0x08);
            self.echo_push(b' ');
            self.echo_push(0x08);
        }
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
                self.erase_one();
            }
            0x15 => {
                // Ctrl-U (KILL): discard the whole pending line, rubbing each char out.
                while self.len > 0 {
                    self.erase_one();
                }
            }
            0x17 => {
                // Ctrl-W (WERASE): erase the trailing word -- first any trailing
                // spaces, then the run of non-space chars before them.
                while self.len > 0 && self.line[self.len - 1] == b' ' {
                    self.erase_one();
                }
                while self.len > 0 && self.line[self.len - 1] != b' ' {
                    self.erase_one();
                }
            }
            0x03 => {
                // Ctrl-C (INTR): raise the interrupt flag and flush the pending line.
                self.intr = true;
                self.len = 0;
                self.echo_push(b'^');
                self.echo_push(b'C');
                self.echo_push(b'\n');
            }
            0x04 => {
                // Ctrl-D (EOF): mid-line, deliver the partial line as-is (no '\n');
                // on an empty line, signal end-of-input to the reader.
                if self.len > 0 {
                    self.ready = true;
                } else {
                    self.eof = true;
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

    /// True once Ctrl-C was seen (a reader would deliver SIGINT + discard the line).
    pub fn took_intr(&self) -> bool {
        self.intr
    }

    /// True once Ctrl-D was seen on an empty line (a reader sees end-of-input).
    pub fn at_eof(&self) -> bool {
        self.eof
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
    } else {
        serial_write(b"TTY: line discipline fail\n");
    }

    // Control characters (full-os Part V.11): Ctrl-U kill-line, Ctrl-W word-erase,
    // Ctrl-C interrupt, Ctrl-D end-of-input.
    let mut ku = LineDiscipline::new();
    for &b in b"abc\x15x\n" {
        ku.input(b);
    }
    let kill_ok = ku.line_ready() && ku.line() == b"x\n"; // Ctrl-U discarded "abc"

    let mut kw = LineDiscipline::new();
    for &b in b"foo bar\x17\n" {
        kw.input(b);
    }
    let werase_ok = kw.line_ready() && kw.line() == b"foo \n"; // Ctrl-W erased "bar"

    let mut ki = LineDiscipline::new();
    for &b in b"q\x03" {
        ki.input(b);
    }
    let intr_ok = ki.took_intr() && ki.line().is_empty(); // Ctrl-C flushed the line

    let mut ke = LineDiscipline::new();
    ke.input(0x04); // Ctrl-D on an empty line
    let eof_ok = ke.at_eof() && !ke.line_ready();

    let ctrl_ok = kill_ok && werase_ok && intr_ok && eof_ok;
    if ctrl_ok {
        serial_write(b"TTY: ctrl-chars ok\n");
    } else {
        serial_write(b"TTY: ctrl-chars fail\n");
    }

    if line_ok && echo_ok && ctrl_ok {
        1
    } else {
        0
    }
}
