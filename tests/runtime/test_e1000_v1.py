# Full-OS guide Part II.7 acceptance: Intel e1000 NIC detection.
#
# Booted with an e1000 attached (-device e1000, the QEMU 82540EM), the kernel must
# discover it on the PCI bus (vendor 0x8086 device 0x100E), map its BAR0, read the
# STATUS register, and read the MAC out of the EEPROM via EERD, reporting what it
# found. The driver registry also emits ATTACH: e1000. The default lane (no e1000)
# reports "E1000: none".

import os
import socket
import subprocess
import sys
import time
import uuid

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import conftest  # noqa: E402

# The e1000 MAC is pinned so the EEPROM read is deterministic. mac bytes
# 52:54:00:12:34:56 -> EEPROM words 0x5452,0x1200,0x5634 -> packed 0x563412005452.
E1000_MAC = "52:54:00:12:34:56"
E1000_MAC_PACKED = "mac=0x0000563412005452"


def _boot_with_e1000(timeout=40):
    iso = os.path.join(conftest.REPO_ROOT, "out", "os-go.iso")
    if not os.path.isfile(iso):
        import pytest

        pytest.skip(f"ISO not built: {iso}")
    serial_port = conftest._pick_serial_port()
    disk = os.path.join(conftest.REPO_ROOT, "out", f"e1000-{uuid.uuid4().hex}.img")
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
        # The device under test: an Intel e1000 NIC on a second netdev.
        "-netdev", "user,id=n1",
        "-device", f"e1000,netdev=n1,mac={E1000_MAC}",
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


def test_e1000_detected(find_in_order):
    out = _boot_with_e1000()
    find_in_order(out, [
        "PCI: enumerate bus0",
        "ATTACH: e1000",
        "E1000: found status=0x",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "E1000: none" not in out
    assert "E1000: bar" not in out  # neither "bar not mmio" nor "bar unassigned"
    assert "E1000: eeprom timeout" not in out
    assert "E1000: mmio map fail" not in out
    # The EEPROM returned the pinned MAC.
    assert E1000_MAC_PACKED in out
