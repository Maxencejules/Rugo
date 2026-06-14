# Installer — contract v1

Status: boot-verified via `make test-installer-v1` (go lane with a 2nd disk)
Source: `kernel_rs/src/lib.rs` (`installer_selftest`, `install_write_and_verify`),
boot call after the net self-tests.
Proof: `tests/runtime/test_installer_v1.py` (asserts the kernel marker AND,
host-side, that the image landed on the target disk file).

Full-OS implementation guide Part V.11 (operations), the install path: the OS
provisions a target disk with a boot record.

## Behaviour

At boot (go lane, after storage + net are up), `installer_selftest`:

- locates the **boot disk** (the first virtio-blk, already initialized) and then
  scans for a **second** virtio-blk — the install target. A generic boot has only
  the boot disk, so it reports `INSTALL: no target` and does nothing (safe);
- switches the block driver to the target (`virtio_blk_init` on its BAR I/O
  base);
- **refuses to clobber a non-blank disk**: it reads the target's sector 0 and
  only provisions if it is all-zero (a fresh disk) or already carries the
  `RUGOINST` magic (idempotent re-install) — otherwise it reports
  `INSTALL: target not blank, refusing` and writes nothing. This self-test runs
  on every boot, so it must never destroy an unrelated data disk;
- writes a **boot record** to sector 0 — a `RUGOINST` magic, an image version
  byte, and the `0x55AA` MBR boot signature — then reads sector 0 back into a
  cleared buffer and verifies a byte-exact round-trip;
- **resets the target** (so it releases the shared single virtqueue) and
  **restores the boot disk** (`virtio_blk_init` on the boot disk's I/O base),
  halting loudly (`INSTALL: boot restore FAIL`) if that restore ever fails rather
  than feeding the shell a half-initialized disk0;
- reports `INSTALL: image written+verified ok` (or `INSTALL: verify FAIL`).

Because the write path is only taken when a second, blank disk is present, the
installer cannot disturb the boot disk, any single-disk lane, or an attached data
disk.

## v1 boundary / carry-forward

- **Provisioning + write/read verification only.** v1 writes one boot-record
  block and proves the cross-disk write/read path. A full bootable install — a
  partition table sized to the target, copying the kernel + a SimpleFS/app-region
  image onto a target partition, and installing the bootloader so the target
  boots standalone — is carry-forward, as is UEFI (a second Limine boot path),
  package fetch over the TCP client, and self-hosting.
- The block layer is single-device, so the installer time-shares it (switch to
  target, provision, switch back). A persistent second-disk handle is future work.

## Acceptance

`make test-installer-v1`: the go lane boots with a blank second virtio-blk disk;
the transcript shows `INSTALL: image written+verified ok` (never
`INSTALL: no target` / `INSTALL: verify FAIL`), the lane still reaches
`GOINIT: result shutdown-clean` and `RUGO: halt ok` (the boot disk was restored),
and host-side the target disk file's sector 0 holds `RUGOINST`, version `0x01`,
and the `0x55AA` signature — proving the write reached the disk.
