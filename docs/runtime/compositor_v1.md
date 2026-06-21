# Compositor / window-server — contract v1

Status: boot-verified via `make test-compositor-v1` (QMP screendump)
Source: `kernel_rs/src/lib.rs` (`sys_ioctl` op 4, `COMPOSE_MAX`), `fb::fb_blit_rect`.
ABI: `sys_ioctl` (id 56) op 4.
Proof: `tests/runtime/test_compositor_v1.py` (counts red + blue pixels in the dump).

Full-OS implementation guide Part III (human interface), the window-server core:
composite multiple independent surfaces onto the framebuffer in **z-order**.
Framebuffer blit (`graphics_v1.md`) draws one rectangle; the compositor draws a
*stack* of surfaces with correct front-to-back ordering in a single frame — the
operation a window-server performs to put windows on a desktop.

## ABI

`sys_ioctl(op=4, a2=ptr, a3=count)`:

- `a3` = number of surface descriptors (1..`COMPOSE_MAX`=16);
- `a2` = pointer to that many 16-byte descriptors in the caller's memory; each is
  two little-endian u64s:
  - word 0: `x<<48 | y<<32 | w<<16 | h` (each a u16, pixels) — the same packing
    as the op-1 blit;
  - word 1: `color` in the low 32 bits (XRGB) `| z<<32` in the high 32 bits.

The kernel copies the descriptors in, **stably sorts the draw order by z
ascending** (painter's algorithm: lowest z = background first, highest z =
foreground last; equal z keeps submission order, so an earlier-submitted surface
is drawn under a later one), and blits each via `fb_blit_rect` (clamped to the
framebuffer). Returns the number of surfaces blitted (or -1 on bad
op/count/pointer).

## v1 boundary / carry-forward

- **Solid-color surfaces, kernel-composited.** A surface is a colored rectangle,
  not yet a per-client pixel buffer. v1 proves z-ordered composition of multiple
  surfaces; **shared-memory pixel surfaces** (a client maps a buffer, draws
  pixels, the compositor blits the buffer), **damage/dirty regions**, alpha
  blending, and a standing **compositor process** that owns the framebuffer are
  carry-forward.
- **No input routing.** Pairing composition with the PS/2 mouse (`mouse_v1.md`)
  + a per-window input event queue (so clicks hit the top window) is the
  window-server's next layer; the mouse device is up but movement reporting needs
  QMP injection to exercise (status doc item 3).

## Acceptance

`make test-compositor-v1`: `compositorprobe` submits a large blue background
(z=0) and a smaller red window (z=1) fully inside it; the kernel composites them
and a QMP screendump shows BOTH > 20000 red pixels (the window, drawn on top) AND
> 20000 blue pixels (the background, still visible around the window) — which only
holds if the two surfaces were blitted in z-order, not as a single rectangle.
