# Phase 7 acceptance: the OS is usable outside a serial pipe.
# - Keyboard: an entire health/shutdown session is typed through the
#   emulated PS/2 keyboard via QMP sendkey; the session can only complete
#   if the kernel's IRQ1/i8042 path delivers the keystrokes.
# - Framebuffer: a QMP screendump after the session must show the boot
#   transcript rendered as pixels (thousands of lit foreground pixels).

import json
import os
import socket
import subprocess
import sys
import time
import uuid

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import conftest  # noqa: E402

KEYMAP = {
    "\n": "ret",
    " ": "spc",
    "-": "minus",
    ".": "dot",
    "/": "slash",
}


def _find_in_order(serial: str, markers: list[str]) -> None:
    pos = -1
    for marker in markers:
        pos = serial.find(marker, pos + 1)
        assert pos != -1, f"Missing '{marker}' in serial output.\nFull output:\n{serial}"


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


def _send_text(f, text):
    for ch in text:
        key = KEYMAP.get(ch, ch)
        _qmp_cmd(f, {"execute": "send-key",
                     "arguments": {"keys": [{"type": "qcode", "data": key}]}})
        time.sleep(0.02)


def _count_lit_pixels(ppm_path):
    with open(ppm_path, "rb") as fh:
        magic = fh.readline().strip()
        assert magic == b"P6", f"unexpected screendump format: {magic!r}"
        dims = fh.readline().split()
        while dims and dims[0].startswith(b"#"):
            dims = fh.readline().split()
        width, height = int(dims[0]), int(dims[1])
        fh.readline()  # maxval
        data = fh.read(width * height * 3)
    lit = 0
    for i in range(0, len(data), 3):
        if data[i] > 96 and data[i + 1] > 96 and data[i + 2] > 96:
            lit += 1
    return lit, width, height


def test_keyboard_session_and_framebuffer_pixels(tmp_path):
    iso = os.path.join(conftest.REPO_ROOT, "out", "os-go.iso")
    if not os.path.isfile(iso):
        import pytest

        pytest.skip(f"ISO not built: {iso}")
    disk = os.path.join(
        conftest.REPO_ROOT, "out", f"console-{uuid.uuid4().hex}.img"
    )
    conftest._ensure_app_region(disk)
    serial_port = conftest._pick_serial_port()
    qmp_port = conftest._pick_serial_port()
    dump = str(tmp_path / "console.ppm")

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
    try:
        deadline = time.monotonic() + 30
        serial = conftest._connect_serial(serial_port, proc, 20)
        qmp_sock, qmp = _qmp_connect(qmp_port, deadline)

        transcript = ""
        typed = False
        dumped = False
        while time.monotonic() < deadline and proc.poll() is None:
            try:
                chunk = serial.recv(4096)
            except socket.timeout:
                chunk = None
            except OSError:
                break
            if chunk:
                transcript += chunk.decode("utf-8", errors="replace")
            if not typed and "GOSH: session ready" in transcript:
                _send_text(qmp, "health\n")
                typed = True
            if typed and not dumped and "SOAKC5: mixed ok" in transcript:
                _qmp_cmd(qmp, {"execute": "screendump",
                               "arguments": {"filename": dump}})
                dumped = True
                _send_text(qmp, "shutdown\n")
        # Drain whatever is left after QEMU exits.
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

    assert dumped, f"never reached the screendump point.\n{transcript}"
    _find_in_order(transcript, [
        "FB: console on",
        "KBD: on",
        "GOSH: session ready",
        "GOSH: lookup ok",
        "SOAKC5: mixed ok",
        "GOSH: shutdown",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])

    lit, width, height = _count_lit_pixels(dump)
    assert width >= 640 and height >= 400
    assert lit > 2000, f"framebuffer nearly empty: {lit} lit pixels"
