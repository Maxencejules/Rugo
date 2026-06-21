# Full-OS guide Part III acceptance: live PS/2 mouse via IRQ12.
#
# The kernel enables the i8042 aux interrupt + mouse data reporting and unmasks
# IRQ12. The test injects relative mouse movement over QMP (input-send-event);
# QEMU delivers it as PS/2 movement packets on IRQ12, the kernel's handler
# assembles + decodes them, accumulates a cursor, and logs the first real
# movement ("MOUSE: irq dx=... dy=...") -- proving live interrupt-driven mouse
# input end to end (beyond the synthetic-packet parser self-test).

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


def _qmp_cmd(f, payload):
    f.write((json.dumps(payload) + "\n").encode())
    f.flush()
    while True:
        line = json.loads(f.readline())
        if "return" in line or "error" in line:
            return line


def _move_mouse(f, dx, dy):
    _qmp_cmd(f, {"execute": "input-send-event", "arguments": {"events": [
        {"type": "rel", "data": {"axis": "x", "value": dx}},
        {"type": "rel", "data": {"axis": "y", "value": dy}},
    ]}})


def test_live_mouse_irq12(tmp_path):
    iso = os.path.join(conftest.REPO_ROOT, "out", "os-go.iso")
    if not os.path.isfile(iso):
        import pytest
        pytest.skip(f"ISO not built: {iso}")
    disk = os.path.join(conftest.REPO_ROOT, "out", f"mouse-{uuid.uuid4().hex}.img")
    conftest._ensure_app_region(disk)
    serial_port = conftest._pick_serial_port()
    qmp_port = conftest._pick_serial_port()
    cmd = [
        conftest.QEMU_BIN,
        "-machine", "q35", "-cpu", "qemu64", "-m", "256",
        "-display", "none", "-no-reboot",
        "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
        "-cdrom", iso,
        "-drive", f"file={disk},if=none,id=disk0,format=raw",
        "-device", "virtio-blk-pci,drive=disk0,disable-modern=on",
        "-netdev", "user,id=n0",
        "-device", "virtio-net-pci,netdev=n0,disable-modern=on",
        "-serial", f"tcp:127.0.0.1:{serial_port},server=on,wait=off",
        "-qmp", f"tcp:127.0.0.1:{qmp_port},server=on,wait=off",
    ]
    proc = subprocess.Popen(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    transcript = ""
    try:
        deadline = time.monotonic() + 35
        serial = conftest._connect_serial(serial_port, proc, 20)
        qmp_sock, qmp = _qmp_connect(qmp_port, deadline)
        injected = False
        shut = False
        while time.monotonic() < deadline and proc.poll() is None:
            try:
                chunk = serial.recv(4096)
            except socket.timeout:
                chunk = None
            except OSError:
                break
            if chunk:
                transcript += chunk.decode("utf-8", errors="replace")
            # Once the mouse IRQ is enabled, inject several movements (QEMU emits a
            # PS/2 packet per event; repeat so at least one full packet lands).
            if not injected and "MOUSE: irq enabled" in transcript:
                for _ in range(5):
                    _move_mouse(qmp, 12, 7)
                    time.sleep(0.05)
                injected = True
            if injected and not shut and "MOUSE: irq dx=" in transcript:
                _qmp_cmd(qmp, {"execute": "system_powerdown"})
                shut = True
                break
        try:
            while True:
                chunk = serial.recv(4096)
                if not chunk:
                    break
                transcript += chunk.decode("utf-8", errors="replace")
        except OSError:
            pass
        serial.close()
        qmp_sock.close()
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

    assert "MOUSE: irq enabled" in transcript, f"mouse IRQ never enabled\n{transcript[-2000:]}"
    assert "MOUSE: irq dx=" in transcript, f"no live mouse packet received\n{transcript[-2000:]}"
