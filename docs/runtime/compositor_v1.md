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

## Persistent surface registry (standing window server)

op 4 composites a *throwaway* list supplied per call — fine for one client drawing
its own frame, but not a window server, which must hold **multiple clients'**
windows persistently and own their lifecycle. `sys_ioctl` adds a persistent,
owner-stamped registry (`WM_SURFACES`, 8 slots):

- **op 8 `wm_register(slot=a2, desc_ptr=a3)`** — register/update the caller's
  surface in `slot` (`desc` = `[geom][color|z<<32]`, same encoding as op 4). The
  surface is stamped with the caller's tid and **persists across calls** until
  cleared or the owner exits. A slot owned by another live client cannot be
  hijacked. Returns the slot.
- **op 9 `wm_compose()`** — composite the **whole** registry (every live client's
  window) to the framebuffer in z-order. Returns the surface count.
- **op 10 `wm_clear(slot=a2)`** — remove the caller's surface (owner-checked, so a
  client cannot close another's window).
- **Exit cleanup** — `wm_release_owner` runs from `r4_exit_and_switch`, so a dead
  client's windows disappear automatically (the lifecycle a server enforces).

`test_winsrv_v1.py`: `wmprobe` registers two windows (red z=0, blue z=1),
composes (=2), clears one, composes (=1), and exits leaving the other registered;
then a **different** client `wmcheck` composes and sees **0** — the kernel removed
the exited owner's window. This proves the registry is a server-owned lifecycle,
not a per-call list.

## v1 boundary / carry-forward

- **Solid-color surfaces.** A surface is a colored rectangle, not yet a per-client
  pixel buffer; **shared-memory pixel surfaces**, **damage/dirty regions**, and
  alpha blending are carry-forward.
- **Registry is kernel-mediated; clients submit directly.** The persistent
  registry + per-client lifecycle exist, but a **resident user-space compositor
  process** driving the compose loop on a timer/vsync (vs each client triggering
  `wm_compose`), plus **two concurrently-live clients** coexisting on screen
  (proven here across two *sequential* clients via exit-cleanup), are the next step.
- **No input routing.** Pairing composition with the mouse + a per-window input
  event queue (so clicks hit the top window) is the window-server's next layer.
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
