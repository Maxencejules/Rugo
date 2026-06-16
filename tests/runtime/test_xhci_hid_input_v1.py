# Full-OS guide Part II.7 / III.8 acceptance: USB HID INPUT REPORTS over xHCI.
#
# With a usb-kbd on the xHCI bus, the kernel enumerates + configures it
# (SET_CONFIGURATION + Configure Endpoint for the interrupt-IN endpoint +
# SET_PROTOCOL boot), then polls the interrupt-IN endpoint for a HID report. This
# test injects a key on the host via QMP send-key while the kernel is polling; the
# kernel must receive the 8-byte boot-keyboard report and read back the keycode
# (USB HID usage 0x04 == 'a').

import json
import os
import socket
import subprocess
import sys
import time
import uuid

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import conftest  # noqa: E402


def _qmp_connect(port, deadline):
    while time.monotonic() < deadline:
        try:
            sock = socket.create_connection(("127.0.0.1", port), timeout=0.5)
            sock.settimeout(5)
            f = sock.makefile("rwb")
            json.loads(f.readline())  # greeting
            f.write(b'{"execute":"qmp_capabilities"}\n')
            f.flush()
            json.loads(f.readline())
            return sock, f
        except OSError:
            time.sleep(0.1)
    raise AssertionError("QMP socket never became ready")


def _boot_and_inject(timeout=50):
    iso = os.path.join(conftest.REPO_ROOT, "out", "os-go.iso")
    if not os.path.isfile(iso):
        import pytest

        pytest.skip(f"ISO not built: {iso}")
    sp = conftest._pick_serial_port()
    qp = conftest._pick_serial_port()
    disk = os.path.join(conftest.REPO_ROOT, "out", f"hidin-{uuid.uuid4().hex}.img")
    conftest._ensure_app_region(disk)
    cmd = [
        conftest.QEMU_BIN, "-machine", "q35", "-cpu", "qemu64", "-smp", "1", "-m", "256",
        "-display", "none", "-no-reboot",
        "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
        "-cdrom", iso, "-boot", "d",
        "-drive", f"file={disk},if=none,id=disk0,format=raw",
        "-device", "virtio-blk-pci,drive=disk0,disable-modern=on",
        "-netdev", "user,id=n0", "-device", "virtio-net-pci,netdev=n0,disable-modern=on",
        "-device", "qemu-xhci,id=xhci", "-device", "usb-kbd,bus=xhci.0",
        "-serial", f"tcp:127.0.0.1:{sp},server=on,wait=off",
        "-qmp", f"tcp:127.0.0.1:{qp},server=on,wait=off",
    ]
    proc = subprocess.Popen(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    out = ""
    try:
        serial = conftest._connect_serial(sp, proc, 20)
        _qsock, qf = _qmp_connect(qp, time.monotonic() + 20)
        deadline = time.monotonic() + timeout
        injected = False
        sent = False
        while time.monotonic() < deadline and proc.poll() is None:
            try:
                chunk = serial.recv(4096)
            except socket.timeout:
                chunk = None
            except OSError:
                break
            if chunk:
                out += chunk.decode("utf-8", errors="replace")
            # Inject the 'a' key once the kernel starts polling the HID endpoint.
            if not injected and "XHCI: hid polling" in out:
                qf.write(
                    b'{"execute":"send-key","arguments":'
                    b'{"keys":[{"type":"qcode","data":"a"}]}}\n'
                )
                qf.flush()
                injected = True
            if not sent and "GOSH: session ready" in out:
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
    return out


def test_xhci_hid_input_report():
    out = _boot_and_inject()
    # The interrupt-IN endpoint is configured, the kernel polls it, and the
    # host-injected 'a' produces an 8-byte boot-keyboard report whose keycode byte
    # is USB HID usage 0x04 ('a'). (Markers are emitted in mixed order -- "polling"
    # and "report" inline during the poll, "configured" in the end marker block --
    # so presence is asserted rather than strict ordering.)
    assert "XHCI: hid configured ep-in ok" in out, out
    assert "XHCI: hid polling" in out, out
    assert "XHCI: hid report mod=0x0000000000000000 key=0x0000000000000004" in out, out
    assert "XHCI: hid no-report" not in out
    assert "XHCI: hid configure fail" not in out
