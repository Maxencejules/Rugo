# Driver model — device enumeration — contract v1

Status: boot-verified via `make test-drivers-v1`
Source: `kernel_rs/src/lib.rs` (`pci_enumerate_log`, called from go-lane boot).
Proof: `tests/runtime/test_drivers_v1.py`.

Full-OS implementation guide Part II.7 (driver model + buses), the
device-discovery slice. Makes the boot-time PCI inventory visible — the
registry-discovery step the per-driver probe/attach refactor builds on.

## Behavior

At go-lane boot, `pci_enumerate_log()` scans PCI bus 0 via the legacy
config-space ports (0xCF8/0xCFC): for each device 0..31 it reads vendor/device
(offset 0), and for multi-function devices (header-type bit 7) walks all 8
functions, logging vendor, device, and class/subclass (offset 0x08) for each
present function, then the total count. It is **read-only** — it claims and
initializes nothing; the existing virtio/NVMe probes still own attachment.

## Markers

| Marker | Format |
|--------|--------|
| `PCI: enumerate bus0` | start of the scan |
| `PROBE: dev=0x<d> func=0x<f> vendor=0x<vid> device=0x<did> class=0x<cc>` | one per present function |
| `PCI: devices=0x<n>` | total functions found |

Hex fields are 16 zero-padded digits (`serial_write_hex`). The q35 test
machine yields 7 functions: host bridge (8086:29C0), VGA (1234:1111),
virtio-blk (1AF4:1001), virtio-net (1AF4:1000), and the multi-function ICH9
LPC/SATA/SMBus at dev 0x1F (funcs 0/2/3).

## v1 boundary / carry-forward

- Bus 0 only (no PCI-to-PCI bridge recursion, no ECAM/multi-segment).
- A discovery log, not yet a `DriverProbe` registry with per-driver
  `probe_fn`/claim, a DMA pool, or IRQ/MSI routing — those, plus moving
  virtio/NVMe onto the registry and new USB/XHCI/e1000 drivers, are
  carry-forward.

## Acceptance

`make test-drivers-v1`: a boot transcript shows `PCI: enumerate bus0`, the
virtio-blk and virtio-net `PROBE:` lines, the multi-function 0x1F device
walked past func 0, and `PCI: devices=0x0000000000000007`.
