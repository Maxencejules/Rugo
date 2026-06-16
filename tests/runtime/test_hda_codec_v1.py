# Full-OS guide Part III acceptance: HD Audio codec communication (CORB/RIRB).
#
# Booted with an HDA controller AND a codec (-device intel-hda -device hda-duplex),
# the kernel resets the controller, sets up the CORB (command) + RIRB (response)
# DMA rings, and round-trips one verb -- GET_PARAMETER(node 0, VENDOR_ID) -- to the
# first present codec, reading its vendor/device id back from the RIRB. This is the
# codec-communication core a real HDA driver (BDL streams + PCM playback) builds
# on. QEMU's HDA codec reports vendor 0x1AF4 (Red Hat).

import os
import socket
import subprocess
import sys
import time
import uuid

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import conftest  # noqa: E402


def _boot_with_hda_codec(timeout=40):
    iso = os.path.join(conftest.REPO_ROOT, "out", "os-go.iso")
    if not os.path.isfile(iso):
        import pytest

        pytest.skip(f"ISO not built: {iso}")
    serial_port = conftest._pick_serial_port()
    disk = os.path.join(conftest.REPO_ROOT, "out", f"hdac-{uuid.uuid4().hex}.img")
    conftest._ensure_app_region(disk)
    cmd = [
        conftest.QEMU_BIN,
        "-machine", "q35", "-cpu", "qemu64", "-smp", "1", "-m", "256",
        "-display", "none", "-no-reboot",
        "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
        "-cdrom", iso, "-boot", "d",
        "-drive", f"file={disk},if=none,id=disk0,format=raw",
        "-device", "virtio-blk-pci,drive=disk0,disable-modern=on",
        "-netdev", "user,id=n0",
        "-device", "virtio-net-pci,netdev=n0,disable-modern=on",
        # The device under test: an HDA controller with a codec attached. A null
        # audio backend (audiodev=none) gives the codec a sink whose timer drives
        # the PCM stream DMA deterministically, regardless of the host's QEMU
        # audio default.
        "-audiodev", "none,id=snd0",
        "-device", "intel-hda", "-device", "hda-duplex,audiodev=snd0",
        "-serial", f"tcp:127.0.0.1:{serial_port},server=on,wait=off",
    ]
    proc = subprocess.Popen(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    transcript = ""
    try:
        serial = conftest._connect_serial(serial_port, proc, 20)
        deadline = time.monotonic() + timeout
        sent = False
        while time.monotonic() < deadline and proc.poll() is None:
            try:
                chunk = serial.recv(4096)
            except socket.timeout:
                chunk = None
            except OSError:
                break
            if chunk:
                transcript += chunk.decode("utf-8", errors="replace")
            if not sent and "GOSH: session ready" in transcript:
                serial.sendall(b"shutdown\n")
                sent = True
        serial.close()
    finally:
        if proc.poll() is None:
            proc.kill()
        try:
            proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            pass
        for _ in range(20):
            try:
                if os.path.isfile(disk):
                    os.remove(disk)
                break
            except PermissionError:
                time.sleep(0.25)
    return transcript


def test_hda_codec_verb_roundtrip(find_in_order):
    out = _boot_with_hda_codec()
    find_in_order(out, [
        "HDA: found gcap=0x",
        # CORB/RIRB round-trip: GET_PARAMETER(VENDOR_ID) returned codec 0's
        # vendor (Red Hat 0x1AF4) -> proof codec communication works.
        "HDA: codec 0000000000000000 vid=0x0000000000001AF4",
        "ok",
        # Codec-tree enumeration: walk the root's SUBORDINATE_NODE_COUNT to the
        # first function group, read its TYPE (1 = audio function group) and its
        # widget count. QEMU's hda-duplex codec reports 1 fg, audio, 4 widgets
        # (DAC, output pin, ADC, input pin) -- the topology a PCM driver walks.
        "HDA: codec enum fgs=0x0000000000000001 afgtype=0x0000000000000001 widgets=0x",
        "ok",
        # PCM playback: the kernel builds a BDL + sample buffer, binds the DAC to
        # output stream SD0, runs it, and watches SDnLPIB advance -- proof the
        # controller is DMAing the buffer to the codec (the real streaming path).
        "HDA: pcm lpib=0x",
        "ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "HDA: no codec" not in out
    assert "HDA: codec no-response" not in out
    assert "HDA: reset fail" not in out
    assert "HDA: dma fail" not in out
    assert "HDA: pcm no-progress" not in out
    assert "HDA: pcm no dac" not in out
    assert "HDA: pcm dma fail" not in out
