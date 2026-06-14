# Full-OS guide Part V.11 acceptance: installer provisioning a target disk.
#
# Booted with a blank SECOND virtio-blk disk attached, the kernel's installer
# finds the target (the first disk is the boot/app-region disk), writes a boot
# record (a "RUGOINST" magic + version + the 0x55AA MBR signature) to its
# sector 0, reads it back to verify, and restores the boot disk. We assert both
# the kernel's own marker AND, host-side, that the target disk file actually
# holds the written image (proving the write reached the disk, not just an
# in-memory round-trip). A generic boot has only one disk, so the installer is a
# safe no-op there ("INSTALL: no target").

import os
import socket
import subprocess
import sys
import time
import uuid

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import conftest  # noqa: E402


def _boot_with_target(timeout=40, target_seed=None):
    iso = os.path.join(conftest.REPO_ROOT, "out", "os-go.iso")
    if not os.path.isfile(iso):
        import pytest

        pytest.skip(f"ISO not built: {iso}")
    serial_port = conftest._pick_serial_port()
    boot_disk = os.path.join(conftest.REPO_ROOT, "out", f"inst-boot-{uuid.uuid4().hex}.img")
    target_disk = os.path.join(conftest.REPO_ROOT, "out", f"inst-target-{uuid.uuid4().hex}.img")
    conftest._ensure_app_region(boot_disk)
    # A 1 MiB install target: blank by default, or pre-seeded (non-blank) so the
    # installer's safety check can be exercised.
    with open(target_disk, "wb") as f:
        f.write(b"\x00" * (1024 * 1024))
    if target_seed is not None:
        with open(target_disk, "r+b") as f:
            f.seek(0)
            f.write(target_seed)
    cmd = [
        conftest.QEMU_BIN,
        "-machine", "q35", "-cpu", "qemu64", "-smp", "1", "-m", "256",
        "-display", "none", "-no-reboot",
        "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
        "-cdrom", iso,
        "-boot", "d",
        # disk0 = boot/app-region disk (the FIRST virtio-blk).
        "-drive", f"file={boot_disk},if=none,id=disk0,format=raw",
        "-device", "virtio-blk-pci,drive=disk0,disable-modern=on",
        # disk1 = blank install target (the SECOND virtio-blk).
        "-drive", f"file={target_disk},if=none,id=disk1,format=raw",
        "-device", "virtio-blk-pci,drive=disk1,disable-modern=on",
        "-netdev", "user,id=n0",
        "-device", "virtio-net-pci,netdev=n0,disable-modern=on",
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
    # Read the target disk's sector 0 back on the host before cleanup.
    target_head = b""
    try:
        with open(target_disk, "rb") as f:
            target_head = f.read(512)
    except OSError:
        pass
    for path in (boot_disk, target_disk):
        for _ in range(20):
            try:
                if os.path.isfile(path):
                    os.remove(path)
                break
            except PermissionError:
                time.sleep(0.25)
    return transcript, target_head


def test_installer_provisions_target_disk(find_in_order):
    out, target_head = _boot_with_target()
    find_in_order(out, [
        "INSTALL: image written+verified ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "INSTALL: verify FAIL" not in out
    assert "INSTALL: no target" not in out
    # Host-side: the image really landed on the target disk.
    assert target_head[0:8] == b"RUGOINST", f"magic missing: {target_head[0:16]!r}"
    assert target_head[8] == 0x01
    assert target_head[510] == 0x55 and target_head[511] == 0xAA


def test_installer_refuses_nonblank_target():
    # A target that already holds (unrelated) data: sector 0 is NOT blank and not
    # our magic. The installer must refuse and leave it byte-for-byte untouched —
    # the self-test runs on every boot, so it must never destroy a data disk.
    seed = bytearray(512)
    seed[0:8] = b"USERDATA"
    seed[16:32] = b"important-bytes!"
    seed[510] = 0x55
    seed[511] = 0xAA
    out, target_head = _boot_with_target(target_seed=bytes(seed))

    assert "INSTALL: target not blank, refusing" in out
    assert "INSTALL: image written+verified ok" not in out
    assert "GOINIT: result shutdown-clean" in out  # boot disk restored cleanly
    # Host-side: the target's sector 0 is exactly the seed (NOT overwritten).
    assert target_head[0:8] == b"USERDATA", f"target was clobbered: {target_head[0:16]!r}"
    assert target_head[16:32] == b"important-bytes!"
