# Framebuffer graphics — contract v1

Status: boot-verified via `make test-graphics-v1`
Source: `kernel_rs/src/fb.rs` (`fb_blit_rect`), `kernel_rs/src/lib.rs`
(`sys_ioctl` op 1), `apps/coreutils/gfxprobe.asm`.
Proof: `tests/runtime/test_graphics_v1.py`.

Full-OS implementation guide Part III (input/graphics/audio), the graphics
slice — direct framebuffer rectangle fill. First pixel-level drawing beyond
the text console.

## ABI

`sys_ioctl` — ABI v3.2 id **56** (the §0.2 generic device-control syscall):

| op | call | args |
|----|------|------|
| 1 | framebuffer blit | `rsi` = rect (`x<<48 | y<<32 | w<<16 | h`, each u16), `rdx` = XRGB color |

Returns 0 on success, -1 if there is no framebuffer, the origin is
off-screen, or the op is unknown. The rectangle is clamped to the screen
bounds. Color is 32-bpp little-endian `0x00RRGGBB` (Limine XRGB).

## Mechanism

`fb::fb_blit_rect` writes the Limine linear framebuffer directly (the same
surface the text console renders into), one `u32` per pixel across
`[x, x+w) x [y, y+h)`. No double-buffering or damage tracking in v1.

## v1 boundary / carry-forward

- Solid-color rectangle fill only. PSF font blitting, image/sprite blits,
  a compositor with damage tracking, a window server, input-event delivery
  for mouse, and audio are all carry-forward (the other Part III items).
- Shares the console's single framebuffer (no per-window surfaces — the
  shared `USER_PML4` constraint the guide notes).

## Acceptance

`make test-graphics-v1`: `gfxprobe` blits a 240x180 red rectangle at
(200,150); a QMP screendump after the blit is parsed and must contain
> 30 000 red pixels (the ~43 200-pixel rect), proving the kernel wrote real
pixels to the framebuffer — not merely a serial marker.
