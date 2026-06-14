# USB xHCI host-controller detection — contract v1

Status: boot-verified via `make test-xhci-v1` (go lane with `-device qemu-xhci`)
Source: `kernel_rs/src/lib.rs` (`xhci_detect`, `xhci_report`, `mmio_map_4k`),
boot call after `pci_enumerate_log`.
Proof: `tests/runtime/test_xhci_v1.py`.

Full-OS implementation guide Part II.7 (Drivers), the USB-stack foundation: the
OS discovers a USB host controller on the PCI bus and reads its capabilities.
The kernel previously had no USB support at all.

## Behaviour

At boot (go lane), after PCI enumeration, `xhci_detect` scans bus 0 for a
function whose class code is **Serial Bus (0x0C) / USB (0x03) / xHCI (0x30)**
(handling multi-function devices). On a match (`xhci_report`):

- enables the controller's PCI **memory space + bus master** (command register);
- reads **BAR0** (the xHCI MMIO base; 32- or 64-bit memory BAR);
- maps the BAR page into the kernel via `mmio_map_4k` — the PCI MMIO hole is
  **not** covered by the Limine HHDM, so this walks the active CR3 through the
  HHDM, allocates any missing page-table levels from the PMM, and installs an
  **uncacheable, non-executable** (PWT|PCD|NX) leaf at a dedicated kernel
  MMIO-window VA. The walk is all-or-nothing (it frees and leaves the tree
  untouched if a frame allocation fails mid-walk);
- reads the capability registers with **32-bit MMIO accesses** (xHCI requires
  dword access): CAPLENGTH + HCIVERSION from dword 0, MaxSlots + MaxPorts from
  HCSPARAMS1 (dword 4);
- reports `XHCI: found ver=<HCIVERSION> caplen=<n> ports=<n> slots=<n>`.

When no xHCI controller is present (the default lane), it reports `XHCI: none`.

## Controller bring-up + command/event ring (`xhci_ring_selftest`)

On detection the kernel also brings the controller up and exercises the
command/event-ring DMA handshake — the core of an xHCI driver:

- maps 32 KiB of the BAR (`mmio_map_region`) covering the operational registers
  (@CAPLENGTH), the runtime registers (@RTSOFF) and the doorbells (@DBOFF);
- **stops + resets** the controller (USBCMD.RUN→0 then HCRST), waiting for
  HCHalted, the reset to clear, and the Controller-Not-Ready bit to clear;
- programs **MaxSlotsEn** (CONFIG), the **DCBAA** (DCBAAP), the **command ring**
  (CRCR | RCS), and the **event ring** + ERST + interrupter 0
  (ERSTSZ/ERSTBA/ERDP) — all in DMA-pool pages ([`dma_v1.md`](dma_v1.md));
- **runs** the controller (USBCMD.RUN), enqueues a **No-Op command** TRB (type
  23) with the producer cycle bit, and rings the command doorbell;
- polls the event ring for a **Command Completion Event** (type 33) with the
  cycle bit and a **Success** completion code — `XHCI: noop ok`.

## Device enumeration (`make test-xhci-hid-v1`)

When a device is attached to a root port (`-device usb-kbd,bus=xhci.0`), the same
bring-up then **enumerates** it — the full USB enumeration path a HID driver
builds on:

- scans the root ports for a Current Connect Status (PORTSC.CCS) and **resets**
  the connected port (PORTSC.PR), preserving the RW1C change bits, then reads its
  speed;
- **Enable Slot** command → the Command Completion Event yields a device slot id;
- builds the **input context** (Add flags for the slot + EP0 contexts), the
  **slot context** (speed, root-hub port, one context entry) and the **EP0
  context** (Control type, max-packet-size by speed, a freshly allocated EP0
  transfer ring as the TR dequeue pointer), and points `DCBAA[slot]` at the
  device (output) context — honoring the controller's context size (HCCPARAMS1.CSZ);
- **Address Device** command (input context + slot id) → Command Completion Event;
- a **GET_DESCRIPTOR(device)** control transfer on the EP0 ring (Setup / Data-IN /
  Status-OUT TRBs, ringing the device's EP0 doorbell) → Transfer Event, then reads
  the **18-byte device descriptor** and verifies `bLength`/`bDescriptorType`.

Events are consumed sequentially (`xhci_wait_event`: dequeue index + consumer
cycle state, skipping unrelated events like Port Status Change, advancing ERDP).
Reports `XHCI: hid enumerated port=<n> vid=<0x..> pid=<0x..>` — for the QEMU
keyboard, vendor `0x0627`, product `0x0001`.

## v1 boundary / carry-forward

- **Detection + capability read + command/event-ring handshake + full device
  enumeration to the device descriptor.** What remains for a complete HID driver:
  the configuration/interface/HID descriptors, SET_CONFIGURATION + SET_PROTOCOL
  (boot), and an **interrupt IN** endpoint transfer ring reading actual key/mouse
  reports (which on QEMU needs QMP input injection to generate events).
- The event ring uses a single ERST segment + polled completion (no MSI-X
  interrupt-driven event delivery yet); one device is enumerated (first connected
  port).

## Acceptance

`make test-xhci-v1`: the go lane boots with `-device qemu-xhci`; the transcript
shows the controller in the PCI enumeration (class `0x0C03`), then
`XHCI: found ver=0x0000000000000100 caplen=0x0000000000000040 ports=0x...08
slots=0x...40` (xHCI 1.0, CAPLENGTH 0x40, 8 root ports, 64 device slots — the
qemu-xhci model), then `XHCI: noop ok` (the controller round-tripped a No-Op
command through the command/event rings), reaching `GOINIT: result shutdown-clean`
and `RUGO: halt ok`,
with no page fault and no `XHCI: none` / `XHCI: bar ...` error.
