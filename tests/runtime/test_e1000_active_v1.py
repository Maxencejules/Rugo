# Full-OS guide Part II.7 acceptance: the Intel e1000 as the ACTIVE NIC.
#
# Booted in a lane with an e1000 NIC and NO virtio-net, the kernel binds the e1000
# as the live network device (net::NIC_KIND = e1000) and the real network stack
# sends/receives through it. The proof is a full DHCP DORA completing over the
# e1000: the kernel starts a DISCOVER (transmitted via the e1000 TX ring) and pumps
# the e1000 RX ring until the lease is ACKed. A "DHCP: ack" in a lane with no virtio
# NIC can only have travelled over the e1000 driver -- proof it is a functioning,
# active NIC, not just a self-test.
#
# Generic lanes always have virtio-net (found first), so the e1000-active path binds
# only here; the rest of the network suite is unaffected.

import os
import socket
import subprocess
import sys
import time
import uuid

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import conftest  # noqa: E402


def _boot_e1000_only(timeout=50):
    iso = os.path.join(conftest.REPO_ROOT, "out", "os-go.iso")
    if not os.path.isfile(iso):
        import pytest

        pytest.skip(f"ISO not built: {iso}")
    serial_port = conftest._pick_serial_port()
    disk = os.path.join(conftest.REPO_ROOT, "out", f"e1ka-{uuid.uuid4().hex}.img")
    conftest._ensure_app_region(disk)
    cmd = [
        conftest.QEMU_BIN,
        "-machine", "q35", "-cpu", "qemu64", "-smp", "1", "-m", "256",
        "-display", "none", "-no-reboot",
        "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
        "-cdrom", iso, "-boot", "d",
        "-drive", f"file={disk},if=none,id=disk0,format=raw",
        "-device", "virtio-blk-pci,drive=disk0,disable-modern=on",
        # The NIC under test: an e1000 with NO virtio-net present, so the kernel must
        # use the e1000 as the active NIC for the real stack (DHCP).
        "-netdev", "user,id=n0",
        "-device", "e1000,netdev=n0",
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
    return transcript


def test_e1000_is_the_active_nic(find_in_order):
    out = _boot_e1000_only()
    find_in_order(out, [
        # Discovery + TX/RX self-tests on the e1000.
        "E1000: found status=0x",
        "E1000: tx ok",
        "E1000: rx ok len=0x",
        # Bound as the live NIC (no virtio present).
        "NETC4: e1000 active",
        "E1000: active link up",
        # The real DHCP client completes a full DORA over the e1000.
        "DHCP: offer ip=0x",
        "DHCP: request sent",
        "DHCP: ack ip=0x",
        "E1000: active dhcp ok",
        # Part II.7 stats counters: GPRC/GPTC tracked the DORA's real TX/RX.
        "E1000: stats gprc=0x",
        # And the system still boots + shuts down cleanly.
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "E1000: active dhcp timeout" not in out
    assert "E1000: none" not in out
    # The stats line must report success (both counters non-zero), not "fail".
    assert any(
        "E1000: stats" in line and line.rstrip().endswith("ok")
        for line in out.splitlines()
    )
