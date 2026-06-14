# Intel HD Audio controller detection — contract v1

Status: boot-verified via `make test-hda-v1`
Source: `kernel_rs/src/lib.rs` (`hda_detect`, `hda_report`).
Proof: `tests/runtime/test_hda_v1.py`.

Full-OS guide Part III (human interface), audio: discover a real audio
controller and read its identity — the foundation a PCM-playback driver builds
on (the PC-speaker beep in [`audio_v1.md`](audio_v1.md) is the prior, codec-less
audio slice).

## Behaviour

`hda_detect` scans PCI bus 0 for a Multimedia / HD-Audio function (class 0x04,
subclass 0x03 — the QEMU `-device intel-hda`). On a match `hda_report` enables
memory space + bus-master, maps BAR0 into the kernel MMIO window (`mmio_map_4k`),
and reads the controller's global capabilities + version: dword 0 packs **GCAP**
(bits [15:0], stream/SDO counts), **VMIN** (bits [23:16]) and **VMAJ**
(bits [31:24]). When no controller is present (the default lane) it reports
`HDA: none`.

## Acceptance

`make test-hda-v1`: booting the go lane with `-device intel-hda`, the transcript
shows `PCI: enumerate bus0` then `HDA: found gcap=0x<gcap> ver=0x<maj><min>`, then
a clean shutdown — with no `HDA: none`, no `HDA: bar …`, and no
`HDA: mmio map fail`.

## v1 boundary / carry-forward

- **Detection + capability read only.** The CORB/RIRB command rings, codec
  enumeration, stream descriptors / BDL, and PCM playback are carry-forward (they
  build on the DMA pool, [`dma_v1.md`](dma_v1.md)).
- HD Audio only (class 0x04/0x03); AC'97 (subclass 0x01, an I/O-BAR device) is
  not matched.
- First BAR page only; single controller (first match wins).
