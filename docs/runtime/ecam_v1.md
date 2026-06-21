# PCIe ECAM (memory-mapped config space) — contract v1

Status: boot-verified via `make test-ecam-v1`
Source: `kernel_rs/src/lib.rs` (`ecam_selftest`, using `pci_read32` +
`mmio_map_4k`).
Proof: `tests/runtime/test_ecam_v1.py`.

Full-OS guide Part II.7 (driver model + buses), PCIe ECAM: access PCI
configuration space through the Enhanced Configuration Access Mechanism (a
memory-mapped window) instead of the legacy `0xCF8`/`0xCFC` I/O port pair.

## Behaviour

`ecam_selftest` reads the q35 MCH **PCIEXBAR** (host-bridge `0:0:0` config
offset `0x60`): bit 0 is the enable, and the high bits give the ECAM base (the
base is read from hardware, not hardcoded). A function's config space is at
`base + (bus<<20) + (dev<<15) + (func<<12) + offset`. It maps the relevant 4 KiB
ECAM page (`mmio_map_4k`, one at a time) and reads the vendor/device dword for two
q35 functions — the MCH host bridge (`0:0:0`) and the LPC bridge (`0:0x1F:0`) —
confirming each **matches** the legacy I/O-port read.

## Acceptance

`make test-ecam-v1`: the boot transcript shows `ECAM: base=0x<phys>` then
`ECAM: selftest ok` (both functions' IDs read identically via ECAM and the legacy
path), with no `ECAM: selftest fail` and no `ECAM: disabled`.

## v1 boundary / carry-forward

- Read-only verification against the legacy path for two functions; routing all
  PCI config access through ECAM (and the full 256-bus reach it enables) is
  carry-forward.
- q35 ECAM only (PCIEXBAR); reading the MCFG ACPI table to find the base on
  arbitrary firmware is carry-forward.
