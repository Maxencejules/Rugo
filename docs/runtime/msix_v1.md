# MSI-X capability setup — contract v1

Status: boot-verified via `make test-msix-v1`
Source: `kernel_rs/src/lib.rs` (`pci_find_cap`, `msix_selftest`,
`msix_table_selftest`).
Proof: `tests/runtime/test_msix_v1.py`.

Full-OS guide Part II.7 (driver model + buses), MSI-X: message-signalled
interrupts — discover the capability, read its sizing, and enable it. The
prerequisite for interrupt-driven (vs polled / legacy-IRQ) drivers.

## Behaviour

- **`pci_find_cap(bdf, id)`**: walks a function's PCI capability list (status bit
  4 gates its presence; the list head is at config offset 0x34; capabilities are
  dword-aligned) and returns the offset of capability `id`.
- **`msix_selftest`**: finds the first function with an MSI-X capability (id
  0x11), reads the **table size** from the Message Control register
  (`(ctl & 0x7FF) + 1`), sets the **MSI-X Enable** bit (15), reads it back to
  confirm, then restores the original Message Control so the existing driver's
  state is undisturbed.

## Acceptance

`make test-msix-v1`: the boot transcript shows `MSIX: dev=0x<device>
vectors=0x<n> enable ok` then `MSIX: table ok bir=0x<n>` (the table registers
latched a written message address/data + mask), with no `MSIX: none`, no `enable
fail`, and no `MSIX: table fail`.

## MSI-X table programming

`msix_table_selftest` locates the MSI-X table from the capability's **Table BIR +
offset** (dword at cap+4), reads the named BAR (32- or 64-bit memory BAR), maps
that page (`mmio_map_4k`), and programs **entry 0**: message address
`0xFEE0_0000` (the x86 LAPIC MSI window), message data `0x41` (a sample vector),
and `vector_control` bit 0 (per-vector mask) set. It reads the three back to
confirm the registers latched, then **restores the original entry** so the device
is left exactly as found. MSI-X is disabled during this (the control register was
restored first), so no live interrupt is ever armed.

## v1 boundary / carry-forward

- Capability discovery + enable/sizing + **table programming (entry 0, with
  read-back + restore)**. What remains: per-vector message routing for every
  vector, unmasking, routing the vectors to IDT handlers, and switching a driver
  from legacy IRQ / polling to live MSI-X delivery — carry-forward (they build on
  the DMA pool + the per-CPU interrupt work in `smp_*`).
- The enable bit + entry 0 are both restored after the test (no live MSI-X
  delivery yet).
