# Next-Wave Native Driver Backlog Closure

As of `2026-03-19`, `M53` and `M54` are no longer only contract or
qualification milestones. The repo now has a real native NVMe lane that boots
in QEMU, binds a PCI device, maps BAR MMIO, routes interrupts, uses DMA-safe
buffers, emits live diagnostics, and drives the existing block plus C4
durability surface end to end.

## Runtime-Backed Outcome

| Backlog | Runtime class now | Implemented lane | Live markers | Live evidence |
|---|---|---|---|---|
| `M53 Native Driver Contract Expansion v1` | `Runtime-backed` | `kernel_rs/src/runtime/native.rs` plus shared block-driver dispatch in `kernel_rs/src/lib.rs` now provide native probe or bind, BAR mapping, MSI or MSI-X routing, DMA-safe memory, firmware-policy hooks, and live diagnostics. | `IRQ: vector bound`, `BAR: map ok`, `DRV: bind driver=nvme`, `FW: allow signed driver=nvme` | `tools/run_native_driver_live_v1.py`, `tests/hw/test_native_driver_live_v1.py`, `make test-native-driver-live-v1` |
| `M54 Native Storage Drivers v1` | `Runtime-backed (NVMe-first)` | The kernel discovers NVMe, performs controller enable, identify, I/O queue setup, block read or write, flush, replay, and `fsync` propagation through the existing block and C4 runtime surface. | `NVME: ready`, `NVME: identify ok`, `NVME: io queue ok`, `BLK: rw ok`, `RECOV: replay ok`, `BLK: fua ok`, `BLK: flush ordered` | `tools/run_native_storage_live_v1.py`, `tests/runtime/test_connected_runtime_c4_native.py`, `tests/hw/test_native_storage_live_v1.py`, `make test-native-storage-live-v1` |

## Implemented Architecture

- Driver core:
  - `block_driver_probe`, `block_io_dispatch`, and `block_flush_dispatch` now
    route storage requests through either legacy VirtIO or native NVMe.
  - `runtime::native::probe_nvme` performs probe, bind, queue bring-up, and
    error reporting.
- PCI or MMIO support:
  - native PCI probing, BAR discovery, and a kernel-installed uncached MMIO
    window now back live controller access.
  - MSI or MSI-X routing is exercised on the native lane with an explicit
    `x2apic` QEMU CPU profile.
- DMA or firmware policy:
  - NVMe queues, identify buffers, and data pages use explicit DMA-safe address
    translation.
  - firmware policy is surfaced as a live runtime hook through
    `FW: allow signed driver=nvme`.
- Storage durability:
  - the C4 storage path now runs on NVMe, including journal replay, FUA-backed
    state commit, flush ordering, and runtime file persistence.

## Live Evidence Commands

1. Build the native images:
   - `make image-blk-native image-go-native`
2. Collect live driver evidence:
   - `python tools/run_native_driver_live_v1.py --out out/native-driver-live-v1.json`
3. Collect live storage or durability evidence:
   - `python tools/run_native_storage_live_v1.py --out out/native-storage-live-v1.json`
4. Run the live tests:
   - `python -m pytest tests/hw/test_native_driver_live_v1.py tests/hw/test_native_storage_live_v1.py tests/runtime/test_connected_runtime_c4_native.py -v`

## Current Native Profile

- Machine: QEMU `q35`
- CPU: `qemu64,+x2apic`
- Storage device: `nvme,drive=disk0,serial=nvme0,logical_block_size=512`
- Network device for the C4 lane: legacy `virtio-net-pci`

## Remaining Breadth Gap

- `M54` is now literal for the NVMe-first runtime lane, but AHCI is still a
  remaining breadth task rather than an in-kernel implementation.
- The new live lane proves native-driver or native-storage substance; the older
  synthetic contract tools remain useful as contract regressions, not as the
  only source of truth.
