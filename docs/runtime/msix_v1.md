# MSI-X capability setup — contract v1

Status: boot-verified via `make test-msix-v1`
Source: `kernel_rs/src/lib.rs` (`pci_find_cap`, `msix_selftest`).
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
vectors=0x<n> enable ok` (a virtio device in the go-lane fixture exposes MSI-X;
the enable bit was set and read back), with no `MSIX: none` and no `enable fail`.

## v1 boundary / carry-forward

- Capability discovery + enable/sizing only. Programming the MSI-X table
  (message address/data per vector, in the BAR named by the cap's Table BIR),
  unmasking vectors, routing them to IDT handlers, and switching a driver from
  legacy IRQ / polling to MSI-X delivery are carry-forward (they build on the DMA
  pool + the per-CPU interrupt work in `smp_*`).
- The enable bit is restored after the test (no live MSI-X delivery yet).
