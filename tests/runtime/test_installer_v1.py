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
        # disk0 = boot/app-region disk (the FIRST virtio-blk). Pin PCI slots so
        # disk0 always enumerates before disk1 — QEMU does NOT guarantee command
        # -line order == PCI device order, and the installer targets the SECOND
        # virtio-blk, so without pinning the target could non-deterministically be
        # the boot disk (a real source of flakiness).
        "-drive", f"file={boot_disk},if=none,id=disk0,format=raw",
        "-device", "virtio-blk-pci,drive=disk0,disable-modern=on,addr=0x05",
        # disk1 = blank install target (the SECOND virtio-blk).
        "-drive", f"file={target_disk},if=none,id=disk1,format=raw",
        "-device", "virtio-blk-pci,drive=disk1,disable-modern=on,addr=0x06",
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
    # Read the target disk's MBR + the installed partition payload (LBA 64..71)
    # back on the host before cleanup (72 sectors covers both).
    target_head = b""
    try:
        with open(target_disk, "rb") as f:
            target_head = f.read(72 * 512)
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


def _install_fill(sector, off):
    # Mirror kernel install_fill(): ((sector*31) + off + 0x5A) & 0xFF.
    return ((sector * 31) + off + 0x5A) & 0xFF


def test_installer_provisions_target_disk(find_in_order):
    out, target_head = _boot_with_target()
    find_in_order(out, [
        "INSTALL: image written+verified ok",
        # The MBR carries a real bootable primary partition...
        "INSTALL: partition type=0x0000000000000083 lba=0x0000000000000040 "
        "sectors=0x0000000000000008",
        # ...the multi-sector payload wrote+verified into it...
        # ...and the kernel wrote its OWN ELF (real bytes read at runtime via the
        # Limine kernel-file response) to the target and verified the round-trip.
        "INSTALL: self-image elf size=0x",
        "INSTALL: bootable install ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "INSTALL: verify FAIL" not in out
    assert "INSTALL: payload FAIL" not in out
    assert "INSTALL: self-image FAIL" not in out
    assert "INSTALL: no target" not in out
    # Host-side: the boot record really landed on the target disk.
    assert target_head[0:8] == b"RUGOINST", f"magic missing: {target_head[0:16]!r}"
    assert target_head[8] == 0x01
    assert target_head[510] == 0x55 and target_head[511] == 0xAA
    # Host-side: a real MBR partition entry (offset 446): bootable 0x80, type
    # 0x83, LBA start 64, 8 sectors.
    e = 446
    assert target_head[e] == 0x80, f"part not bootable: {target_head[e]:#x}"
    assert target_head[e + 4] == 0x83, f"part type: {target_head[e + 4]:#x}"
    assert int.from_bytes(target_head[e + 8:e + 12], "little") == 64
    assert int.from_bytes(target_head[e + 12:e + 16], "little") == 8
    # Host-side: the installed partition payload (LBA 64..71). Sector 0 of the
    # partition is its own boot record; every sector carries the fill pattern.
    p0 = 64 * 512
    assert target_head[p0:p0 + 8] == b"RUGOPART", f"payload magic: {target_head[p0:p0 + 8]!r}"
    assert target_head[p0 + 510] == 0x55 and target_head[p0 + 511] == 0xAA
    for s in range(8):
        base = (64 + s) * 512
        for off in (16, 100, 503):
            assert target_head[base + off] == _install_fill(s, off), (
                f"payload sector {s} off {off}: {target_head[base + off]:#x} "
                f"!= {_install_fill(s, off):#x}"
            )


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
