# Full-OS guide Part IV.10 acceptance: RDRAND hardware seeding for the CSPRNG.
#
# The kernel folds the CPU's RDRAND hardware RNG into the xorshift64* seed when
# CPUID advertises it (CPUID.1:ECX[30]), XOR-mixed so it can only strengthen the
# CMOS/PIT soft seed. Booted on a CPU model WITH rdrand (-cpu qemu64,+rdrand),
# rng_hwseed_selftest reports "RNG: hwseed rdrand ok"; on a CPU without it the
# kernel falls back to the soft seed ("RNG: hwseed soft (no rdrand)"). Either way
# the pool must still produce distinct draws (never "RNG: hwseed FAIL").

import os
import socket
import subprocess
import sys
import time
import uuid

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import conftest  # noqa: E402
import pytest  # noqa: E402


def _boot(cpu, timeout=40):
    iso = conftest.ISO_GO_PATH
    if not os.path.isfile(iso):
        pytest.skip(f"ISO not built: {iso}")
    if not conftest.QEMU_BIN:
        pytest.skip("qemu-system-x86_64 not found")
    sp = conftest._pick_serial_port()
    disk = os.path.join(conftest.REPO_ROOT, "out", f"rng-{uuid.uuid4().hex}.img")
    conftest._ensure_app_region(disk)
    cmd = [
        conftest.QEMU_BIN, "-machine", "q35", "-cpu", cpu, "-smp", "1", "-m", "256",
        "-display", "none", "-no-reboot",
        "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
        "-cdrom", iso, "-boot", "d",
        "-drive", f"file={disk},if=none,id=disk0,format=raw",
        "-device", "virtio-blk-pci,drive=disk0,disable-modern=on",
        "-netdev", "user,id=n0", "-device", "virtio-net-pci,netdev=n0,disable-modern=on",
        "-serial", f"tcp:127.0.0.1:{sp},server=on,wait=off",
    ]
    proc = subprocess.Popen(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    out = ""
    try:
        serial = conftest._connect_serial(sp, proc, 20)
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
                out += chunk.decode("utf-8", errors="replace")
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


def test_rng_hwseed_rdrand():
    # With RDRAND advertised, the CSPRNG must fold it into the seed.
    out = _boot("qemu64,+rdrand")
    assert "RNG: hwseed rdrand ok" in out, out
    assert "RNG: hwseed FAIL" not in out
    assert "RUGO: halt ok" in out


def test_rng_hwseed_soft_fallback():
    # Without RDRAND, the portable soft seed path is used and still healthy.
    out = _boot("qemu64")
    assert "RNG: hwseed soft (no rdrand)" in out, out
    assert "RNG: hwseed FAIL" not in out
    assert "RUGO: halt ok" in out
