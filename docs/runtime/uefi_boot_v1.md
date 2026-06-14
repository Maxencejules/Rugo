# UEFI boot — contract v1

Status: boot-verified via `make test-uefi-boot-v1` (boots under OVMF/edk2)
Source: no kernel changes — the kernel's Limine-request bring-up is already
firmware-agnostic; this adds the UEFI boot medium + a test.
Proof: `tests/runtime/test_uefi_boot_v1.py` (skips if OVMF firmware is absent).

Full-OS implementation guide Part V.11 (operations), the second boot path: the
same kernel that boots under legacy BIOS must also boot under UEFI firmware.

## Behaviour

Limine is a hybrid (BIOS + UEFI) bootloader. The BIOS path ships in the ISO
(`mkimage.sh`, `limine-bios.sys`); the UEFI path uses Limine's `BOOTX64.EFI`.
The test assembles an **EFI System Partition** as a host directory:

- `/EFI/BOOT/BOOTX64.EFI` — Limine's UEFI loader (vendored);
- `/boot/limine/limine.conf` — the same config the BIOS ISO uses;
- `/boot/kernel.elf` — the go-lane kernel (`kernel-go.elf`).

It exposes that directory to QEMU via **VVFAT** (`-drive file=fat:rw:DIR`, on
IDE/SATA so OVMF's fallback boot finds `\EFI\BOOT\BOOTX64.EFI`) and boots it
under **OVMF/edk2** (`-drive if=pflash` code + a writable vars store), with the
usual virtio-blk (app region) and virtio-net devices attached.

OVMF (BdsDxe) loads `BOOTX64.EFI` → Limine loads `boot():/boot/kernel.elf` → the
kernel runs. Its hardware bring-up is **firmware-agnostic** because it consumes
Limine requests (HHDM, memory map, kernel-address, framebuffer/GOP, SMP) rather
than BIOS services, so the full sequence — `RUGO: boot ok`, MM/PMM/heap, SMP
spinlock, PCI enumeration, the net responder self-tests, services, the shell, and
a clean `shutdown` — runs identically to the BIOS lane.

## v1 boundary / carry-forward

- **Boot path verified, not yet shipped in the ISO.** The test builds the ESP on
  the fly (VVFAT); folding a UEFI El-Torito entry (`limine-uefi-cd.bin` +
  `BOOTX64.EFI`) into `mkimage.sh` so `os-go.iso` is itself hybrid-bootable needs
  `xorriso` (currently absent; the build uses a pycdlib BIOS-only fallback). A
  hand-rolled or `mtools`-built FAT ESP image is the alternative.
- **Secure Boot** (signed BOOTX64.EFI + the secure OVMF variant) is out of scope.
- A no-NIC minimal UEFI config would fault in the TCP listener self-test (it
  `wire_send`s with no initialized NIC); the test attaches a NIC like the BIOS
  lane. Hardening `wire_send` to no-op without a ready NIC is a separate robustness
  item.

## Acceptance

`make test-uefi-boot-v1`: under OVMF the transcript shows `RUGO: boot ok`,
`GOSH: session ready`, `GOINIT: result shutdown-clean`, and `RUGO: halt ok`, with
no `USERPF`/`PF: addr` — i.e. the kernel boots and shuts down cleanly under UEFI,
matching the BIOS lane. The test skips gracefully where the edk2 firmware is not
installed.
