# Audio (PC speaker) — contract v1

Status: boot-verified via `make test-beep-v1`
Source: `kernel_rs/src/lib.rs` (`sys_ioctl` op 3), `apps/coreutils/beepprobe.asm`.
Proof: `tests/runtime/test_beep_v1.py`.

Full-OS implementation guide Part III (human interface), audio slice — the
simplest sound output: a square-wave tone on the legacy PC speaker.

## Behaviour

`sys_ioctl` (id 56) **op 3** = beep: `rsi` = frequency in Hz (20..20000).
It programs i8253/i8254 PIT channel 2:

- `out 0x43, 0xB6` — channel 2, lobyte/hibyte, mode 3 (square wave);
- `out 0x42, divisor` (low then high), `divisor = 1193182 / Hz`;
- `out 0x61, prev | 0x03` — set the timer-2 gate (bit 0) and speaker-data
  enable (bit 1).

It then reads port `0x61` back and returns the gate bits (`3` = enabled), after
which it clears bits 0/1 again (v1 has no timer-driven duration, so it does not
leave the speaker droning — the read-back already proved the enable took). It
logs `BEEP: freq=0x.. div=0x.. gate=0x..`.

## v1 boundary / carry-forward

- **PC speaker only**, one square-wave tone, no sustained duration (no timer to
  auto-silence), no note queue, no volume.
- No PCM / AC'97 / Intel HD Audio / virtio-sound — a real audio stack
  (mixer, DMA ring, sample formats) is carry-forward.
- The tone is not asserted audibly in CI (headless); the test verifies the PIT
  divisor and the port-0x61 read-back instead.

## Acceptance

`make test-beep-v1`: `beepprobe` calls op 3 at 440 Hz; the transcript shows
`BEEP: freq=0x..01B8 div=0x..0A97 gate=0x..03` (divisor 2711, speaker enabled)
and `BEEPPROBE: ok`.
