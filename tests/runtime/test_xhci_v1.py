# Full-OS guide Part II.7 acceptance: USB xHCI host-controller detection.
#
# Booted with a qemu-xhci controller attached, the kernel must discover it on the
# PCI bus (class 0x0C / subclass 0x03 / prog-if 0x30), read its memory-mapped
# capability registers (HCIVERSION, CAPLENGTH, HCSPARAMS1) through the HHDM, and
# report the controller it found ("XHCI: found ver=... ports=... slots=..."). The
# same scan reports "XHCI: none" when no controller is present (the default lane).

import os
import socket
import subprocess
import sys
import time
import uuid

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import conftest  # noqa: E402


def _boot_with_xhci(timeout=40):
    iso = os.path.join(conftest.REPO_ROOT, "out", "os-go.iso")
    if not os.path.isfile(iso):
        import pytest

        pytest.skip(f"ISO not built: {iso}")
    serial_port = conftest._pick_serial_port()
    disk = os.path.join(conftest.REPO_ROOT, "out", f"xhci-{uuid.uuid4().hex}.img")
    conftest._ensure_app_region(disk)
    cmd = [
        conftest.QEMU_BIN,
        "-machine", "q35", "-cpu", "qemu64", "-smp", "1", "-m", "256",
        "-display", "none", "-no-reboot",
        "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
        "-cdrom", iso,
        "-boot", "d",
        "-drive", f"file={disk},if=none,id=disk0,format=raw",
        "-device", "virtio-blk-pci,drive=disk0,disable-modern=on",
        "-netdev", "user,id=n0",
        "-device", "virtio-net-pci,netdev=n0,disable-modern=on",
        # The device under test: a USB 3 xHCI host controller.
        "-device", "qemu-xhci",
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
        try:
            while True:
                chunk = serial.recv(4096)
                if not chunk:
                    break
                transcript += chunk.decode("utf-8", errors="replace")
        except OSError:
            pass
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


def test_xhci_controller_detected(find_in_order):
    out = _boot_with_xhci()
    find_in_order(out, [
        "PCI: enumerate bus0",
        # The xHCI controller is discovered and its capability registers read.
        "XHCI: found ver=0x0000000000000100",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "XHCI: none" not in out
    assert "XHCI: bar" not in out  # neither "bar not mmio" nor "bar unassigned"
