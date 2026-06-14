// PS/2 keyboard input (gap-analysis item 7): scancode set 1 make codes
// decoded to ASCII and queued for the console. Bytes arrive through
// IRQ1 while user code runs, and through direct i8042 polling while the
// console read loop spins in the kernel (interrupts are masked there).

#![allow(dead_code)]

use crate::arch_x86::{inb, outb};

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

// ---------------- PS/2 mouse (full-os guide Part III input) ----------------
//
// The i8042 hosts a second PS/2 port (the mouse) alongside the keyboard. v1
// brings the mouse up: enable the aux port, reset it, and read its BAT result +
// device ID, proving the OS can talk to the pointing device. Continuous movement
// reporting (and a compositor consuming it) is carry-forward — movement packets
// need QMP input injection to exercise, and the keyboard poll already drains any
// stray aux bytes (status bit 5) so an enabled mouse never disturbs the console.

/// Wait until the i8042 input buffer is empty (ready to accept a write).
unsafe fn ctrl_wait_write() -> bool {
    let mut spins = 0u32;
    while inb(0x64) & 0x02 != 0 {
        spins += 1;
        if spins > 200_000 {
            return false;
        }
    }
    true
}

/// Read the next byte the mouse (aux) sent, draining any interleaved keyboard
/// bytes (distinguished by status bit 5). None on timeout.
unsafe fn mouse_read() -> Option<u8> {
    let mut spins = 0u32;
    loop {
        let st = inb(0x64);
        if st & 0x01 != 0 {
            let b = inb(0x60);
            if st & 0x20 != 0 {
                return Some(b); // an aux (mouse) byte
            }
            // a keyboard byte: ignore and keep waiting for the mouse reply
        }
        spins += 1;
        if spins > 4_000_000 {
            return None;
        }
    }
}

/// Send one command byte to the mouse (0xD4 routes the next data byte to the aux
/// port) and consume its ACK (0xFA). Returns true on ACK.
unsafe fn mouse_cmd(cmd: u8) -> bool {
    if !ctrl_wait_write() {
        return false;
    }
    outb(0x64, 0xD4); // next 0x60 write goes to the aux device
    if !ctrl_wait_write() {
        return false;
    }
    outb(0x60, cmd);
    matches!(mouse_read(), Some(0xFA))
}

/// Mouse init self-test (full-os guide Part III): enable the aux port, reset the
/// mouse, and read its Basic Assurance Test result (0xAA) + device ID (0x00 for a
/// standard PS/2 mouse). Returns 1 on success. Runs at boot with interrupts off,
/// so the keyboard poll cannot race the reply bytes.
pub(crate) unsafe fn mouse_selftest() -> u64 {
    // Enable the auxiliary (mouse) device on the i8042.
    if !ctrl_wait_write() {
        return 0;
    }
    outb(0x64, 0xA8);
    // Reset: 0xFF -> ACK(0xFA), then BAT(0xAA), then device ID(0x00).
    if !mouse_cmd(0xFF) {
        return 0;
    }
    let bat = match mouse_read() {
        Some(b) => b,
        None => return 0,
    };
    let id = match mouse_read() {
        Some(b) => b,
        None => return 0,
    };
    if bat != 0xAA {
        return 0;
    }
    crate::serial_write(b"MOUSE: reset bat=0x");
    crate::serial_write_hex(bat as u64);
    crate::serial_write(b" id=0x");
    crate::serial_write_hex(id as u64);
    crate::serial_write(b" ok\n");
    1
}

/// Decode a standard 3-byte PS/2 mouse movement packet into signed (dx, dy) and
/// the button bitmap (bit0 left, bit1 right, bit2 middle). byte0 holds the button
/// + sign + overflow flags; byte1/byte2 are 9-bit two's-complement movement whose
/// high bit lives in byte0 (X sign = bit4, Y sign = bit5). Returns None if the
/// sync bit (byte0 bit3) is clear (an out-of-sync / invalid packet).
fn mouse_decode(p: [u8; 3]) -> Option<(i32, i32, u8)> {
    let b0 = p[0];
    if b0 & 0x08 == 0 {
        return None; // sync bit must be set on a valid first packet byte
    }
    let mut dx = p[1] as i32;
    let mut dy = p[2] as i32;
    if b0 & 0x10 != 0 {
        dx -= 256; // X sign -> negative
    }
    if b0 & 0x20 != 0 {
        dy -= 256; // Y sign -> negative
    }
    Some((dx, dy, b0 & 0x07))
}

/// Mouse movement-packet self-test (full-os guide Part III input): decode a
/// sequence of synthetic PS/2 packets, accumulate a cursor position + button
/// state, and verify the signed movement (including negative via the sign bits),
/// the button bitmap, and that an out-of-sync packet (sync bit clear) is
/// rejected. v1: the packet parser + cursor accumulation; live IRQ12 delivery and
/// QMP-injected movement are carry-forward. Emits `MOUSE: packet ok` / `fail`.
pub(crate) unsafe fn mouse_packet_selftest() -> u64 {
    let mut x = 0i32;
    let mut y = 0i32;
    let mut buttons = 0u8;

    // 1) +5, +3 with the left button held. byte0 = sync(0x08) | left(0x01) = 0x09.
    match mouse_decode([0x09, 5, 3]) {
        Some((dx, dy, b)) if dx == 5 && dy == 3 && b == 0x01 => {
            x += dx;
            y += dy;
            buttons = b;
        }
        _ => {
            crate::serial_write(b"MOUSE: packet fail\n");
            return 0;
        }
    }
    // 2) -2, -1 with no buttons. byte0 = sync | X sign(0x10) | Y sign(0x20) = 0x38;
    // byte1 = 254 (-2), byte2 = 255 (-1).
    match mouse_decode([0x38, 254, 255]) {
        Some((dx, dy, b)) if dx == -2 && dy == -1 && b == 0x00 => {
            x += dx;
            y += dy;
            buttons = b;
        }
        _ => {
            crate::serial_write(b"MOUSE: packet fail\n");
            return 0;
        }
    }
    // 3) An out-of-sync packet (sync bit clear) must be rejected.
    if mouse_decode([0x00, 9, 9]).is_some() {
        crate::serial_write(b"MOUSE: packet fail\n");
        return 0;
    }
    // The cursor accumulated to (5-2, 3-1) = (3, 2); buttons cleared in packet 2.
    if x != 3 || y != 2 || buttons != 0 {
        crate::serial_write(b"MOUSE: packet fail\n");
        return 0;
    }
    crate::serial_write(b"MOUSE: packet ok x=0x");
    crate::serial_write_hex(x as u64);
    crate::serial_write(b" y=0x");
    crate::serial_write_hex(y as u64);
    crate::serial_write(b"\n");
    1
}
