# Full-OS guide Part III acceptance: per-client pixel surfaces (sys_ioctl op 6).
#
# Beyond op 4's solid-color rectangles, op 6 composites a real per-pixel client
# bitmap to the framebuffer. surfprobe builds a 32x32 surface -- top half GREEN,
# bottom half BLUE -- and blits it at (300,200). A QMP screendump must show BOTH
# colors in that region: a two-color bitmap a solid-color rect could not produce,
# proving real per-pixel surfaces. Uses its OWN minimal app region (base-shell +
# surfprobe) since the shared 40-app region is full to the on-disk VFS boundary.

import json
import os
import socket
import subprocess
import sys
import time
import uuid

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import conftest  # noqa: E402
import pytest  # noqa: E402

KEYMAP = {"\n": "ret", " ": "spc", "-": "minus", ".": "dot", "/": "slash"}


def _qmp_connect(port, deadline):
    while time.monotonic() < deadline:
        try:
            sock = socket.create_connection(("127.0.0.1", port), timeout=0.5)
            sock.settimeout(5)
            f = sock.makefile("rwb")
            json.loads(f.readline())
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


def _count_colors(ppm_path):
    with open(ppm_path, "rb") as fh:
        assert fh.readline().strip() == b"P6"
        dims = fh.readline().split()
        while dims and dims[0].startswith(b"#"):
            dims = fh.readline().split()
        width, height = int(dims[0]), int(dims[1])
        fh.readline()
        data = fh.read(width * height * 3)
    green = blue = 0
    for i in range(0, len(data), 3):
        r, g, b = data[i], data[i + 1], data[i + 2]
        if g > 200 and r < 60 and b < 60:
            green += 1
        elif b > 200 and r < 60 and g < 60:
            blue += 1
    return green, blue, width, height


def test_surface_compose_pixel_bitmap(tmp_path):
    iso = conftest.ISO_GO_PATH
    if not os.path.isfile(iso):
        pytest.skip(f"ISO not built: {iso}")
    if not conftest.QEMU_BIN:
        pytest.skip("qemu-system-x86_64 not found")
    disk = os.path.join(conftest.REPO_ROOT, "out", f"surf-{uuid.uuid4().hex}.img")
    with open(disk, "wb") as f:
        f.write(b"\x00" * (1024 * 1024))
    cmd0 = [sys.executable, conftest.APP_DISK_V1_TOOL, "--disk", disk]
    for name in ("base-shell", "surfprobe"):
        elf = os.path.join(conftest.REPO_ROOT, "out", f"app-{name}.elf")
        if not os.path.isfile(elf):
            pytest.skip(f"app ELF not built: {elf}")
        cmd0 += ["--app", f"{name}={elf}"]
    subprocess.run(cmd0, check=True, capture_output=True, text=True)
    serial_port = conftest._pick_serial_port()
    qmp_port = conftest._pick_serial_port()
    dump = str(tmp_path / "surf.ppm")
    cmd = [
        conftest.QEMU_BIN, "-machine", "q35", "-cpu", "qemu64", "-m", "256",
        "-display", "none", "-no-reboot",
        "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
        "-cdrom", iso,
        "-drive", f"file={disk},if=none,id=disk0,format=raw",
        "-device", "virtio-blk-pci,drive=disk0,disable-modern=on",
        "-netdev", "user,id=n0", "-device", "virtio-net-pci,netdev=n0,disable-modern=on",
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
                _send_text(qmp, "probe surfprobe\n")
                typed = True
            if typed and not dumped and "SURFACE: compose ok" in transcript:
                _qmp_cmd(qmp, {"execute": "screendump", "arguments": {"filename": dump}})
                dumped = True
                _send_text(qmp, "shutdown\n")
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
    assert "SURFACE: compose FAIL" not in transcript
    green, blue, width, height = _count_colors(dump)
    # 32x32 = 1024 px, ~512 green + ~512 blue; require both clearly present (a
    # solid-color rect could never show two colors in one composed surface).
    assert green > 200, f"green half missing: {green} green px"
    assert blue > 200, f"blue half missing: {blue} blue px"
