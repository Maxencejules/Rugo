# Full-OS guide Part III acceptance: framebuffer graphics (sys_ioctl op 1).
#
# `gfxprobe` draws a 240x180 red rectangle via the framebuffer blit syscall.
# A QMP screendump after the blit must show thousands of red pixels - proof
# the kernel actually wrote the linear framebuffer, not just a serial marker.

import json
import os
import socket
import subprocess
import sys
import time
import uuid

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import conftest  # noqa: E402

KEYMAP = {"\n": "ret", " ": "spc", "-": "minus", ".": "dot", "/": "slash"}


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


def _count_red_pixels(ppm_path):
    with open(ppm_path, "rb") as fh:
        assert fh.readline().strip() == b"P6"
        dims = fh.readline().split()
        while dims and dims[0].startswith(b"#"):
            dims = fh.readline().split()
        width, height = int(dims[0]), int(dims[1])
        fh.readline()  # maxval
        data = fh.read(width * height * 3)
    red = 0
    for i in range(0, len(data), 3):
        if data[i] > 200 and data[i + 1] < 60 and data[i + 2] < 60:
            red += 1
    return red, width, height


def test_framebuffer_blit_paints_red_rect(tmp_path, find_in_order):
    iso = os.path.join(conftest.REPO_ROOT, "out", "os-go.iso")
    if not os.path.isfile(iso):
        import pytest
        pytest.skip(f"ISO not built: {iso}")
    disk = os.path.join(conftest.REPO_ROOT, "out", f"gfx-{uuid.uuid4().hex}.img")
    conftest._ensure_app_region(disk)
    serial_port = conftest._pick_serial_port()
    qmp_port = conftest._pick_serial_port()
    dump = str(tmp_path / "gfx.ppm")

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
    dumped = False
    try:
        deadline = time.monotonic() + 30
        serial = conftest._connect_serial(serial_port, proc, 20)
        qmp_sock, qmp = _qmp_connect(qmp_port, deadline)
        typed = False
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
                _send_text(qmp, "probe gfxprobe\n")
                typed = True
            if typed and not dumped and "GFXPROBE: blit ok" in transcript:
                _qmp_cmd(qmp, {"execute": "screendump",
                               "arguments": {"filename": dump}})
                dumped = True
                _send_text(qmp, "shutdown\n")
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
    assert "GFXPROBE: FAIL" not in transcript
    red, width, height = _count_red_pixels(dump)
    assert width >= 640 and height >= 400
    # The rect is 240x180 = 43200 px; allow generous slack for clamping.
    assert red > 30000, f"red rectangle not painted: {red} red pixels"
