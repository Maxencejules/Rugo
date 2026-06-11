# Local Console Contract v1 (Framebuffer + PS/2 Keyboard)

Status: live runtime (boot-verified)
Source: `kernel_rs/src/fb.rs`, `kernel_rs/src/kbd.rs`, console routing in
`kernel_rs/src/lib.rs`
Proof: `make test-console-v1`, `tests/runtime/test_console_runtime_v1.py`

Closes gap-analysis build-list item 7: the OS is usable outside a serial
pipe — output is drawn as pixels, input arrives from the keyboard.

## Framebuffer text console

- A Limine framebuffer request adopts the bootloader-provided linear
  framebuffer (32 bpp). Marker: `FB: console on 0x<w> x 0x<h>`, or
  `FB: none` when the bootloader provides none (serial-only fallback).
- Every `serial_write` is mirrored onto the framebuffer with an embedded
  public-domain 8x8 font (ASCII 0x20–0x7E), with newline handling,
  backspace, wraparound, and scroll. The full boot transcript is on
  screen. Compiled into every lane, like `mm.rs`.
- Acceptance: a QMP `screendump` after a typed session must contain
  thousands of lit foreground pixels.
- Carry-forward: a larger PSF font, color classes per marker prefix,
  cursor rendering.

## PS/2 keyboard

- IRQ1 (vector 33) decodes scancode set 1 make codes (letters, digits,
  punctuation, shift pairs, enter, backspace) into a 64-byte queue;
  marker `KBD: on` when the line is unmasked in the go lane.
- The console read path takes keyboard bytes first and falls back to
  serial, so both input sources work interchangeably. While the read
  loop spins inside the kernel (interrupts masked), it polls the i8042
  directly; the IRQ path covers bytes arriving while user code runs.
  The handler checks output-buffer-full before reading the data port —
  a latched IRQ for an already-polled byte must not re-read stale data
  (that doubles every keystroke).
- Acceptance: an entire `health`/`shutdown` session is typed through QMP
  `send-key` — the session only completes if the keyboard path delivers
  every keystroke.
- Carry-forward: blocking reads should sleep the task instead of
  spinning (needs kernel wait queues); extended scancodes (arrows,
  keypad).
