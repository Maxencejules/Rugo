// PS/2 keyboard input (gap-analysis item 7): scancode set 1 make codes
// decoded to ASCII and queued for the console. Bytes arrive through
// IRQ1 while user code runs, and through direct i8042 polling while the
// console read loop spins in the kernel (interrupts are masked there).

#![allow(dead_code)]

use crate::arch_x86::inb;

const QUEUE: usize = 64;

struct Kbd {
    buf: [u8; QUEUE],
    head: usize,
    tail: usize,
    shift: bool,
}

static mut KBD: Kbd = Kbd {
    buf: [0; QUEUE],
    head: 0,
    tail: 0,
    shift: false,
};

// Scancode set 1, unshifted / shifted (0 = ignore).
const MAP: [u8; 0x3A] = [
    0, 27, b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'0',
    b'-', b'=', 8, b'\t', b'q', b'w', b'e', b'r', b't', b'y', b'u', b'i',
    b'o', b'p', b'[', b']', b'\n', 0, b'a', b's', b'd', b'f', b'g', b'h',
    b'j', b'k', b'l', b';', b'\'', b'`', 0, b'\\', b'z', b'x', b'c', b'v',
    b'b', b'n', b'm', b',', b'.', b'/', 0, b'*', 0, b' ',
];
const MAP_SHIFT: [u8; 0x3A] = [
    0, 27, b'!', b'@', b'#', b'$', b'%', b'^', b'&', b'*', b'(', b')',
    b'_', b'+', 8, b'\t', b'Q', b'W', b'E', b'R', b'T', b'Y', b'U', b'I',
    b'O', b'P', b'{', b'}', b'\n', 0, b'A', b'S', b'D', b'F', b'G', b'H',
    b'J', b'K', b'L', b':', b'"', b'~', 0, b'|', b'Z', b'X', b'C', b'V',
    b'B', b'N', b'M', b'<', b'>', b'?', 0, b'*', 0, b' ',
];

unsafe fn push(b: u8) {
    let next = (KBD.head + 1) % QUEUE;
    if next != KBD.tail {
        KBD.buf[KBD.head] = b;
        KBD.head = next;
    }
}

unsafe fn decode(scancode: u8) {
    match scancode {
        0x2A | 0x36 => KBD.shift = true,
        0xAA | 0xB6 => KBD.shift = false,
        sc if sc & 0x80 == 0 && (sc as usize) < MAP.len() => {
            let ch = if KBD.shift {
                MAP_SHIFT[sc as usize]
            } else {
                MAP[sc as usize]
            };
            if ch != 0 {
                push(ch);
            }
        }
        _ => {}
    }
}

/// IRQ1 entry: consume pending scancodes. The output-buffer-full check
/// matters: the console wait loop polls the i8042 directly, and a
/// latched IRQ1 for a byte the poll already consumed would otherwise
/// re-read the stale data register and double every keystroke.
pub(crate) unsafe fn kbd_irq() {
    kbd_poll();
}

/// Poll the i8042 directly (the console read loop runs with interrupts
/// masked in the kernel, so IRQ1 cannot fire there).
pub(crate) unsafe fn kbd_poll() {
    while inb(0x64) & 0x01 != 0 {
        if inb(0x64) & 0x20 != 0 {
            // Mouse byte: drain and ignore.
            let _ = inb(0x60);
            continue;
        }
        decode(inb(0x60));
    }
}

/// Pop one decoded byte, or None when the queue is empty.
pub(crate) unsafe fn kbd_pop() -> Option<u8> {
    if KBD.tail == KBD.head {
        return None;
    }
    let b = KBD.buf[KBD.tail];
    KBD.tail = (KBD.tail + 1) % QUEUE;
    Some(b)
}

pub(crate) unsafe fn kbd_has_input() -> bool {
    kbd_poll();
    KBD.tail != KBD.head
}
