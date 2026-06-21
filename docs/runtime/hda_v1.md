# Intel HD Audio controller detection — contract v1

Status: boot-verified via `make test-hda-v1` + `make test-hda-codec-v1`
Source: `kernel_rs/src/lib.rs` (`hda_detect`, `hda_report`, `hda_codec_selftest`).
Proof: `tests/runtime/test_hda_v1.py`, `tests/runtime/test_hda_codec_v1.py`.

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

## Codec communication (CORB/RIRB)

With a codec attached (`-device hda-duplex`), `hda_codec_selftest` brings the
controller out of reset, sets up the **CORB** (command) + **RIRB** (response) DMA
rings (256 entries each, on the DMA pool), runs both DMA engines, and round-trips
one verb — **GET_PARAMETER(node 0, VENDOR_ID)** — to the first codec named by
`STATESTS`, reading its vendor/device id back from `RIRB[1]`:
`HDA: codec 0 vid=0x1AF4 did=0x22 ok` (QEMU's HDA codec is Red Hat 0x1AF4). A
controller with no codec reports `HDA: no codec` (a bounded, harmless no-op). This
is the codec-communication core a PCM driver builds on. `make test-hda-codec-v1`.

**Codec-tree enumeration.** After the identity round-trip the selftest issues
three more GET_PARAMETER verbs to walk the codec topology: the root node's
**SUBORDINATE_NODE_COUNT** (how many function groups, and the first one's node id),
then that function group's **FUNCTION_GROUP_TYPE** (1 = audio) and its own
**SUBORDINATE_NODE_COUNT** (the widget count under the AFG):
`HDA: codec enum fgs=0x1 afgtype=0x1 widgets=0x4 ok`. QEMU's `hda-duplex` reports
one audio function group with four widgets (DAC, output pin, ADC, input pin) — the
node tree a real driver walks to locate the DAC and output-pin widgets before
configuring a stream. Issuing more than one verb requires **RINTCNT** set above the
verb count: QEMU's CORB engine stops servicing the ring once `rirb_count` reaches
the response-interrupt threshold, so a threshold of 1 (sufficient for the single
identity verb) would stall enumeration after the first response.

## PCM playback (BDL + output stream)

After enumeration `hda_pcm_selftest` runs the actual streaming path. It walks the
AFG's widgets reading **AUDIO_WIDGET_CAPABILITIES** (param 0x09) to find the **DAC**
(the first widget whose type field, bits [23:20], is 0 = Audio Output), then:

1. fills a DMA page with a 16-bit square-wave PCM buffer and builds a two-entry
   **Buffer Descriptor List** (spec minimum) covering it (IOC set per entry);
2. resets output **stream descriptor SD0** (`SRST`) — located at
   `0x80 + ISS*0x20` where ISS is `GCAP[11:8]` (QEMU reports ISS=4, so SD0 is at
   `0x100`) — and programs its BDL pointer, cyclic buffer length, last-valid-index,
   format (`SDnFMT` = 48 kHz / 16-bit / 2-channel = `0x0011`), and stream number;
3. configures the codec **DAC** with four verbs — `SET_POWER_STATE` D0,
   `SET_CONVERTER_FORMAT` (matching `SDnFMT`), `SET_CONVERTER_STREAM_CHANNEL`
   (binding the DAC to the same stream tag SD0 uses, so the codec pulls from it),
   and `SET_AMP_GAIN_MUTE` (unmute);
4. sets `SDnCTL.RUN` and watches **SDnLPIB** (link position in buffer) advance.

A moving LPIB proves the controller is DMAing the BDL buffer to the codec — the end
to end PCM path. `HDA: pcm lpib=0x<pos> ok` (a stalled stream reports
`HDA: pcm no-progress`; both are wall-clock bounded via PIT-calibrated TSC so boot
never wedges). The test attaches a null audio backend (`-audiodev none`,
`hda-duplex,audiodev=...`) whose timer drives the stream deterministically.
`make test-hda-codec-v1`.

## Userspace PCM (`sys_ioctl` op 7 / `hda_audio_play`)

On success the self-test **keeps** the buffer + BDL + SD0 programming + codec binding
as a persistent audio context (`AUDIO_READY`) instead of tearing it down, so ring-3
apps can play sound without re-initialising the codec. `sys_ioctl` (id 56) **op 7 =
audio_write** (`a2` = PCM pointer, `a3` = length) `copyin_user`s up to one buffer of
16-bit PCM and calls `hda_audio_play`, which copies the samples into the stream
buffer (zero-padding to silence), reprograms + runs SD0, and waits (wall-clock
bounded) for SDnLPIB to advance — returning the byte count accepted (0 if no audio
device or the stream stalled). `hda_audio_selftest` drives that exact core with a
kernel-built block at boot: `AUDIO: play n=0x200 ok`. This is the userspace PCM
submission path (the headline Part III audio deliverable), on top of the full HD
Audio driver rather than the guide's simpler AC'97/virtio-snd suggestion.

## v1 boundary / carry-forward

- **Detection + capability read + CORB/RIRB codec verb round-trip + codec-tree
  enumeration + PCM stream playback + a userspace audio_write path** (sys_ioctl
  op 7). What
  remains: a `sys_audio_write` syscall exposing playback to userspace, input/capture
  (ADC streams), interrupt-driven BDL completion (BCIS) instead of LPIB polling, and
  per-widget routing (CONNECTION_LIST / pin config for real output paths).
- HD Audio only (class 0x04/0x03); AC'97 (subclass 0x01, an I/O-BAR device) is
  not matched.
- First BAR page only; single controller (first match wins). First Audio Output
  widget is used as the DAC; first output stream descriptor (SD0) is used.
