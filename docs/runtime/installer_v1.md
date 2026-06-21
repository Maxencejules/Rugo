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
- writes an **MBR** to sector 0 — a `RUGOINST` magic, an image version byte, a
  real **primary partition-table entry** (bootable flag `0x80`, type `0x83`, LBA
  start 64, 8 sectors — the same 16-byte layout the kernel's own MBR parser at
  `sys_sysinfo` op 5 reads), and the `0x55AA` boot signature — then reads sector 0
  back into a cleared buffer and verifies the magic, signature, **and** the
  partition entry (type/LBA/sector-count) round-tripped exactly;
- writes the **partition image**: an 8-sector payload into the partition (LBA
  64..71) whose first sector is its own boot record (`RUGOPART` magic + `0x55AA`)
  and whose every sector carries a deterministic fill pattern, then reads all 8
  sectors back and verifies the magic + fill (a missing, duplicated, or
  mis-ordered sector is detected);
- **resets the target** (so it releases the shared single virtqueue) and
  **restores the boot disk** (`virtio_blk_init` on the boot disk's I/O base),
  halting loudly (`INSTALL: boot restore FAIL`) if that restore ever fails rather
  than feeding the shell a half-initialized disk0;
- reports `INSTALL: image written+verified ok`, then
  `INSTALL: partition type=0x…83 lba=0x…40 sectors=0x…08` and
  `INSTALL: bootable install ok` (or `INSTALL: payload FAIL` / `INSTALL: verify
  FAIL`).

Because the write path is only taken when a second, blank disk is present, the
installer cannot disturb the boot disk, any single-disk lane, or an attached data
disk.

## v1 boundary / carry-forward

- **Partitioned install + write/read verification.** v1 lays down a real MBR
  partition table and a multi-sector partition image on the target and verifies
  the whole round-trip (incl. the partition entry the kernel parser reads back).
  What remains for a target that boots *standalone*: copying the actual kernel +
  a full SimpleFS/app-region filesystem image into the partition (vs the
  fixed-pattern verification payload), and installing the bootloader stages
  (Limine BIOS/UEFI) onto the target — which needs the host image-build tooling
  (xorriso/mtools) on the target disk. UEFI boot of the *primary* image already
  works (`uefi_boot_v1.md`); package fetch over TCP already works
  (`pkgfetch_v1.md`); self-hosting remains.
- The block layer is single-device, so the installer time-shares it (switch to
  target, provision, switch back). A persistent second-disk handle is future work.

## Acceptance

`make test-installer-v1`: the go lane boots with a blank second virtio-blk disk;
the transcript shows `INSTALL: image written+verified ok`, the partition line,
and `INSTALL: bootable install ok` (never `INSTALL: no target` / `INSTALL: verify
FAIL` / `INSTALL: payload FAIL`), the lane still reaches `GOINIT: result
shutdown-clean` and `RUGO: halt ok` (the boot disk was restored), and host-side
the target disk file holds, at sector 0, `RUGOINST` + version `0x01` + a bootable
`0x83` partition entry (LBA 64, 8 sectors) + the `0x55AA` signature, and at LBA 64
the `RUGOPART` partition boot record + the fill pattern across all 8 sectors —
proving the partitioned install reached the disk.
