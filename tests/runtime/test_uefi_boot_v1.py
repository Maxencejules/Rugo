# Full-OS guide Part V.11 acceptance: UEFI boot path.
#
# The same kernel that boots under BIOS (the rest of the suite) must also boot
# under UEFI firmware. We assemble an EFI System Partition (Limine's UEFI loader
# at /EFI/BOOT/BOOTX64.EFI, the limine config, and the kernel) as a host
# directory, expose it to QEMU via VVFAT, and boot it under OVMF (edk2). The
# kernel must reach a clean shutdown, exactly like the BIOS lane — proving the
# Limine UEFI -> kernel handoff and the kernel's Limine-request-based bring-up
# (HHDM, memmap, framebuffer, SMP) are firmware-agnostic.

import os
import shutil
import socket
import subprocess
import sys
import time
import uuid

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import conftest  # noqa: E402


def _find_ovmf():
    """Return (code_fd, vars_template_fd) for x86_64 OVMF/edk2, or (None, None)."""
    qemu_dir = os.path.dirname(conftest.QEMU_BIN)
    candidates = [
        os.path.join(qemu_dir, "share"),
        "/usr/share/edk2/x64",
        "/usr/share/ovmf",
        "/usr/share/OVMF",
    ]
    for d in candidates:
        code = os.path.join(d, "edk2-x86_64-code.fd")
        if os.path.isfile(code):
            for v in ("edk2-i386-vars.fd", "edk2-x86_64-vars.fd"):
                vt = os.path.join(d, v)
                if os.path.isfile(vt):
                    return code, vt
        ovmf_code = os.path.join(d, "OVMF_CODE.fd")
        ovmf_vars = os.path.join(d, "OVMF_VARS.fd")
        if os.path.isfile(ovmf_code) and os.path.isfile(ovmf_vars):
            return ovmf_code, ovmf_vars
    return None, None


def _build_esp(esp_dir):
    """Assemble an EFI System Partition tree from the vendored Limine UEFI files."""
    root = conftest.REPO_ROOT
    bootx64 = os.path.join(root, "limine", "BOOTX64.EFI")
    if not os.path.isfile(bootx64):
        bootx64 = os.path.join(root, "vendor", "limine", "BOOTX64.EFI")
    kernel = os.path.join(root, "out", "kernel-go.elf")
    conf = os.path.join(root, "boot", "limine.conf")
    for f in (bootx64, kernel, conf):
        if not os.path.isfile(f):
            return False
    shutil.rmtree(esp_dir, ignore_errors=True)
    os.makedirs(os.path.join(esp_dir, "EFI", "BOOT"), exist_ok=True)
    os.makedirs(os.path.join(esp_dir, "boot", "limine"), exist_ok=True)
    shutil.copyfile(bootx64, os.path.join(esp_dir, "EFI", "BOOT", "BOOTX64.EFI"))
    shutil.copyfile(kernel, os.path.join(esp_dir, "boot", "kernel.elf"))
    shutil.copyfile(conf, os.path.join(esp_dir, "boot", "limine", "limine.conf"))
    return True


def test_kernel_boots_under_uefi(find_in_order):
    import pytest

    iso_kernel = os.path.join(conftest.REPO_ROOT, "out", "kernel-go.elf")
    if not os.path.isfile(iso_kernel):
        pytest.skip("kernel-go.elf not built")
    code_fd, vars_template = _find_ovmf()
    if not code_fd:
        pytest.skip("OVMF/edk2 x86_64 firmware not found")

    out_dir = os.path.join(conftest.REPO_ROOT, "out")
    esp_dir = os.path.join(out_dir, f"esp-uefi-{uuid.uuid4().hex}")
    if not _build_esp(esp_dir):
        pytest.skip("Limine UEFI files / kernel not available")
    vars_fd = os.path.join(out_dir, f"ovmf-vars-{uuid.uuid4().hex}.fd")
    shutil.copyfile(vars_template, vars_fd)
    disk = os.path.join(out_dir, f"uefi-{uuid.uuid4().hex}.img")
    conftest._ensure_app_region(disk)
    serial_port = conftest._pick_serial_port()

    cmd = [
        conftest.QEMU_BIN,
        "-machine", "q35", "-cpu", "qemu64", "-m", "256",
        "-display", "none", "-no-reboot",
        "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
        "-drive", f"if=pflash,format=raw,unit=0,readonly=on,file={code_fd}",
        "-drive", f"if=pflash,format=raw,unit=1,file={vars_fd}",
        # The ESP via VVFAT on IDE/SATA so OVMF's fallback boot finds
        # \\EFI\\BOOT\\BOOTX64.EFI (the virtio-blk data disk has no ESP).
        "-drive", f"file=fat:rw:{esp_dir},format=raw,if=ide",
        "-drive", f"file={disk},if=none,id=disk0,format=raw",
        "-device", "virtio-blk-pci,drive=disk0,disable-modern=on",
        "-netdev", "user,id=n0",
        "-device", "virtio-net-pci,netdev=n0,disable-modern=on",
        "-serial", f"tcp:127.0.0.1:{serial_port},server=on,wait=off",
    ]
    proc = subprocess.Popen(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    transcript = ""
    try:
        serial = conftest._connect_serial(serial_port, proc, 20)
        deadline = time.monotonic() + 45
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
        shutil.rmtree(esp_dir, ignore_errors=True)
        for f in (vars_fd, disk):
            for _ in range(20):
                try:
                    if os.path.isfile(f):
                        os.remove(f)
                    break
                except PermissionError:
                    time.sleep(0.25)

    find_in_order(transcript, [
        "RUGO: boot ok",        # the kernel started under UEFI/Limine
        "GOSH: session ready",  # full bring-up reached the shell
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "USERPF" not in transcript
    assert "PF: addr" not in transcript
