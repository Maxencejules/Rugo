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

## Input event queue (`sys_ioctl` op 5)

The IRQ12 handler no longer just logs movement — it **enqueues** each decoded
packet into a kernel input-event ring (`input_enqueue`), which userspace drains
via `sys_ioctl` op 5 (`input_poll(buf, max_events)`). Each event is 16 bytes:
`kind` u8 (1 = mouse: `data` = button bitmap, `x`/`y` = accumulated cursor;
2 = key: `data` = scancode), `data` u32, `x` i32, `y` i32. The ring (64 entries)
overwrites the oldest when full; producer (IRQ, IF=0) and consumer (syscall,
IF=0) never run concurrently on the single core, so no lock is needed.
`input_event_selftest` enqueues a synthetic mouse-move + key and drains them back,
verifying the ring + encoding round-trip (`INPUT: event queue ok`).

## v1 boundary / carry-forward

- **Bring-up + packet parser + live IRQ12 delivery + an input event queue
  (`sys_ioctl` op 5)** are done. What remains: a keyboard-event producer feeding
  the same ring; routing clicks to the top window in a compositor/window server;
  an end-to-end QMP-injected poll test from ring 3; scroll-wheel / Intellimouse
  (4-byte packets); absolute pointing (USB tablet).

## Acceptance

`make test-mouse-v1`: the go lane boots, the transcript shows
`MOUSE: reset bat=0x...AA id=0x...00 ok`, and the keyboard-driven shell still
takes the `shutdown` keystrokes and reaches `GOINIT: result shutdown-clean` and
`RUGO: halt ok` — confirming the enabled mouse does not disturb console input.
