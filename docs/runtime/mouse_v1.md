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

## v1 boundary / carry-forward

- **Bring-up only.** v1 does not enable continuous data reporting (`0xF4`) or
  parse 3-byte movement packets, and it does not raise an event to userspace.
  After reset the mouse defaults to reporting **disabled**, so it stays quiet and
  cannot flood the i8042; the existing keyboard poll would drain any aux bytes
  anyway (so the console is unaffected).
- **No movement test.** Exercising movement/button packets needs QMP
  `input-send-event` injection (the boot fixture only feeds a fixed keyboard
  string). A movement-reporting driver + an input event queue feeding a
  compositor/window-server is the carry-forward (status doc item 3).

## Acceptance

`make test-mouse-v1`: the go lane boots, the transcript shows
`MOUSE: reset bat=0x...AA id=0x...00 ok`, and the keyboard-driven shell still
takes the `shutdown` keystrokes and reaches `GOINIT: result shutdown-clean` and
`RUGO: halt ok` — confirming the enabled mouse does not disturb console input.
