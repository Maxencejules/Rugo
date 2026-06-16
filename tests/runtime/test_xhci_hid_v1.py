# Full-OS guide Part II.7 acceptance: USB device enumeration over xHCI (the
# foundation a HID driver builds on).
#
# Booted with a qemu-xhci controller AND a usb-kbd attached to it, the kernel must
# bring the controller up, find the keyboard on a root port, reset the port,
# allocate a device slot (Enable Slot), set up the device + input contexts and an
# EP0 transfer ring, address the device (Address Device), and issue a
# GET_DESCRIPTOR(device) control transfer that reads the 18-byte USB device
# descriptor -- reporting the keyboard's vendor/product id (QEMU HID: 0x0627/0x0001).

import os
import socket
import subprocess
import sys
import time
import uuid

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import conftest  # noqa: E402


def _boot_with_xhci_kbd(timeout=40):
    iso = os.path.join(conftest.REPO_ROOT, "out", "os-go.iso")
    if not os.path.isfile(iso):
        import pytest

        pytest.skip(f"ISO not built: {iso}")
    serial_port = conftest._pick_serial_port()
    disk = os.path.join(conftest.REPO_ROOT, "out", f"xhcihid-{uuid.uuid4().hex}.img")
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
        # The device under test: an xHCI controller with a USB keyboard attached.
        "-device", "qemu-xhci,id=xhci",
        "-device", "usb-kbd,bus=xhci.0",
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


def test_xhci_enumerates_usb_keyboard(find_in_order):
    out = _boot_with_xhci_kbd()
    find_in_order(out, [
        "XHCI: found ver=0x0000000000000100",
        # The command/event ring works (No-Op completes).
        "XHCI: noop ok",
        # Full enumeration: port reset -> Enable Slot -> Address Device ->
        # GET_DESCRIPTOR returned the 18-byte device descriptor with the QEMU
        # HID keyboard's vendor (0x0627) and product (0x0001) ids.
        "XHCI: hid enumerated port=0x0000000000000005 vid=0x0000000000000627 pid=0x0000000000000001",
        # The device is then put into its configured state (SET_CONFIGURATION),
        # an interrupt-IN endpoint is added (Configure Endpoint command), and the
        # HID boot protocol is selected (SET_PROTOCOL) -- ready to poll reports.
        "XHCI: hid configured ep-in ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "XHCI: enum fail" not in out
    assert "XHCI: hid configure fail" not in out
    assert "XHCI: noop fail" not in out
    assert "XHCI: none" not in out
