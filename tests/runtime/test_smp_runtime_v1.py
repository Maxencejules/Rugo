# Phase 10d acceptance: SMP bring-up groundwork. Booted with -smp 4,
# the kernel must report all four CPUs from the Limine SMP response and
# every application processor must check in (run real kernel code on
# its own core) before parking. The default Go lane must also boot and
# shut down cleanly on multicore with the APs parked.

import os
import socket
import subprocess
import sys
import time
import uuid

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import conftest  # noqa: E402


def _find_in_order(serial: str, markers: list[str]) -> None:
    pos = -1
    for marker in markers:
        pos = serial.find(marker, pos + 1)
        assert pos != -1, f"Missing '{marker}' in serial output.\nFull output:\n{serial}"


def _boot_smp(iso, smp, input_text=None, with_devices=False, timeout=30):
    serial_port = conftest._pick_serial_port()
    cmd = [
        conftest.QEMU_BIN,
        "-machine", "q35", "-cpu", "qemu64", "-smp", str(smp), "-m", "256",
        "-display", "none", "-no-reboot",
        "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
        "-cdrom", iso,
        "-serial", f"tcp:127.0.0.1:{serial_port},server=on,wait=off",
    ]
    disk = None
    if with_devices:
        disk = os.path.join(conftest.REPO_ROOT, "out", f"smp-{uuid.uuid4().hex}.img")
        conftest._ensure_app_region(disk)
        cmd += [
            "-drive", f"file={disk},if=none,id=disk0,format=raw",
            "-device", "virtio-blk-pci,drive=disk0,disable-modern=on",
            "-netdev", "user,id=n0",
            "-device", "virtio-net-pci,netdev=n0,disable-modern=on",
        ]
    proc = subprocess.Popen(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    transcript = ""
    try:
        serial = conftest._connect_serial(serial_port, proc, 20)
        deadline = time.monotonic() + timeout
        sent = input_text is None
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
                serial.sendall(input_text.encode())
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
        if disk:
            for _ in range(20):
                try:
                    if os.path.isfile(disk):
                        os.remove(disk)
                    break
                except PermissionError:
                    time.sleep(0.25)
    return transcript


def test_aps_check_in_on_quad_core():
    iso = os.path.join(conftest.REPO_ROOT, "out", "os.iso")
    if not os.path.isfile(iso):
        import pytest

        pytest.skip(f"ISO not built: {iso}")
    out = _boot_smp(iso, 4)
    _find_in_order(out, [
        "RUGO: boot ok",
        "SMP: cpus=0x0000000000000004",
        "SMP: aps online=0x0000000000000003",
        "RUGO: halt ok",
    ])


def test_default_lane_boots_clean_on_multicore():
    iso = os.path.join(conftest.REPO_ROOT, "out", "os-go.iso")
    if not os.path.isfile(iso):
        import pytest

        pytest.skip(f"ISO not built: {iso}")
    out = _boot_smp(iso, 2, input_text="shutdown\n", with_devices=True, timeout=40)
    _find_in_order(out, [
        "SMP: cpus=0x0000000000000002",
        "SMP: aps online=0x0000000000000001",
        "GOSH: session ready",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "USERPF" not in out
    assert "GOINIT: err" not in out
