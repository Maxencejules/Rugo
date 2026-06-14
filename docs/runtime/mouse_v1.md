# PS/2 mouse bring-up — contract v1

Status: boot-verified via `make test-mouse-v1` (go C4 runtime lane)
Source: `kernel_rs/src/kbd.rs` (`mouse_selftest`, `mouse_cmd`, `mouse_read`,
`ctrl_wait_write`), boot call after `xhci_detect`.
Proof: `tests/runtime/test_mouse_v1.py`.

Full-OS implementation guide Part III (Human interface), pointing-device input:
the i8042 hosts a second PS/2 port for the mouse alongside the keyboard. This
brings the mouse device up so the OS can talk to it.

## Behaviour

At boot (go lane, interrupts still masked), `mouse_selftest`:

- enables the **auxiliary (mouse) port** on the i8042 (controller command 0xA8);
- **resets** the mouse (`0xD4` routes the next byte to the aux port, then `0xFF`),
  consuming its `0xFA` ACK, its Basic Assurance Test result `0xAA`, and its
  device ID `0x00` (a standard, non-Intellimouse PS/2 mouse);
- reports `MOUSE: reset bat=0xAA id=0x00 ok`.

`mouse_read` distinguishes mouse bytes from keyboard bytes by the i8042 status
**aux bit (0x20)**, draining any interleaved keyboard byte — and the reset runs
with interrupts masked, so the existing keyboard IRQ/poll cannot race the reply.

## Movement-packet parsing (`mouse_packet_selftest`)

`mouse_decode` parses a standard 3-byte PS/2 movement packet into signed
`(dx, dy)` + a button bitmap: byte 0 carries the buttons (bit0 left, bit1 right,
bit2 middle), the always-1 **sync** bit (bit3), and the X/Y **sign** bits
(bit4/bit5); bytes 1/2 are the 9-bit two's-complement movement whose high bit is
the sign bit in byte 0. A packet whose sync bit is clear is rejected
(out-of-sync). `mouse_packet_selftest` decodes a sequence (`+5,+3` left-down, then
`-2,-1` no-buttons), accumulates a cursor, and confirms it reaches `(3, 2)` and
that an out-of-sync packet is dropped — `MOUSE: packet ok x=0x3 y=0x2`.

## Live IRQ12 delivery (`mouse_enable_irq` / `mouse_irq`)

`mouse_enable_irq` (called when the go runtime brings up interrupts) sets the
i8042 command-byte aux-IRQ bit (clearing the aux clock-disable), turns on the
mouse's data reporting (`0xF4`), and unmasks the cascade (IRQ2) + IRQ12 (PIC2
line 4). The IRQ12 service routine (`mouse_irq`, trap vector 44, EOI to both
PICs) reads each aux byte, assembles a 3-byte packet (resyncing on the sync bit),
decodes it via `mouse_decode`, and accumulates the cursor — logging the first real
movement as `MOUSE: irq dx=… dy=… btn=…`. Proven with QMP-injected movement
(`make test-mouse-irq-v1`): the keyboard + PIT keep working alongside it.

## v1 boundary / carry-forward

- **Bring-up + packet parser + live IRQ12 delivery** are all done. What remains:
  an input **event queue** delivering movement/clicks to a compositor/window
  server and routing clicks to the top window (status doc item 3); scroll-wheel /
  Intellimouse (4-byte packets); absolute pointing (USB tablet).

## Acceptance

`make test-mouse-v1`: the go lane boots, the transcript shows
`MOUSE: reset bat=0x...AA id=0x...00 ok`, and the keyboard-driven shell still
takes the `shutdown` keystrokes and reaches `GOINIT: result shutdown-clean` and
`RUGO: halt ok` — confirming the enabled mouse does not disturb console input.
