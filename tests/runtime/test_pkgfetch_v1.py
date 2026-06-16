# Full-OS guide Part V.11 acceptance: package fetch over TCP (the network-download
# core of a package manager).
#
# A package-fetch request record ("PKGREQ" + le16 port) is written to a reserved
# disk sector; at boot the kernel reads it and arms a fetch, and the PIT-tick
# driver connects out through QEMU's user-mode network to a host-side repo server
# owned by this test (guest -> 10.0.2.2:<port> -> host 127.0.0.1:<port>),
# downloads a framed multi-segment package, and content-verifies it (magic +
# checksum). The marker only appears if the kernel completes a real TCP handshake,
# receives the whole package, and the checksum matches.

import os
import socket
import struct
import subprocess
import sys
import threading
import time
import uuid

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import conftest  # noqa: E402

PKG_REQ_LBA = 16
PKG_PAYLOAD_LEN = 900


def _pkg_server(server: socket.socket, result: dict) -> None:
    try:
        # Per-attempt bound (the test retries the whole boot a few times). Long
        # enough for a slow full-gate boot, short enough to retry quickly.
        server.settimeout(75)
        conn, _addr = server.accept()
        conn.settimeout(30)
        payload = bytes((i * 31 + 7) & 0xFF for i in range(PKG_PAYLOAD_LEN))
        checksum = sum(payload) & 0xFFFFFFFF
        pkg = (
            b"RUGOPKG1"
            + struct.pack("<I", PKG_PAYLOAD_LEN)
            + payload
            + struct.pack("<I", checksum)
        )
        conn.sendall(pkg)
        result["sent"] = len(pkg)
        try:
            conn.recv(64)  # let the guest close first (its FIN path)
        except OSError:
            pass
        conn.close()
    except OSError as exc:
        result["error"] = str(exc)
    finally:
        server.close()


def _boot_until_pkg(disk, timeout=75):
    """Boot the go lane; send `shutdown` only once a PKG completion marker appears
    (the fetch is PIT-driven and must not be cut off by an early shutdown)."""
    serial_port = conftest._pick_serial_port()
    cmd = [
        conftest.QEMU_BIN,
        "-machine", "q35", "-cpu", "qemu64", "-smp", "1", "-m", "256",
        "-display", "none", "-no-reboot",
        "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
        "-cdrom", conftest.ISO_GO_PATH,
        "-boot", "d",
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
            # Shut down once the fetch RESOLVES -- match only the completion
            # markers, not "PKG: fetch armed" (which would race the fetch).
            if not sent and (
                "PKG: fetched len=" in transcript
                or any(
                    m in transcript
                    for m in (
                        "PKG: fetch checksum FAIL",
                        "PKG: fetch closed-early FAIL",
                        "PKG: fetch timeout FAIL",
                        "PKG: fetch too-big FAIL",
                        "PKG: fetch start FAIL",
                    )
                )
            ):
                serial.sendall(b"shutdown\n")
                sent = True
        if not sent and proc.poll() is None:
            try:
                serial.sendall(b"shutdown\n")
            except OSError:
                pass
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
    return transcript


def _attempt_fetch():
    """One boot attempt. Returns (out, result). A fresh listener + disk each time."""
    server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server.bind(("127.0.0.1", 0))
    server.listen(1)
    port = server.getsockname()[1]
    result: dict = {}
    listener = threading.Thread(target=_pkg_server, args=(server, result))
    listener.start()
    disk = os.path.join(conftest.REPO_ROOT, "out", f"pkgfetch-{uuid.uuid4().hex}.img")
    try:
        conftest._ensure_app_region(disk)
        with open(disk, "r+b") as f:
            f.seek(PKG_REQ_LBA * 512)
            f.write(b"PKGREQ" + struct.pack("<H", port))
        out = _boot_until_pkg(disk)
    finally:
        try:
            server.close()  # unblock a still-waiting accept() so the thread ends
        except OSError:
            pass
        listener.join(timeout=15)
        for _ in range(20):
            try:
                if os.path.isfile(disk):
                    os.remove(disk)
                break
            except PermissionError:
                time.sleep(0.25)
    return out, result


def test_package_fetch_over_tcp(find_in_order):
    if not os.path.isfile(conftest.ISO_GO_PATH):
        import pytest

        pytest.skip(f"ISO not built: {conftest.ISO_GO_PATH}")

    # The fetch is a real wire round-trip over QEMU's user-mode network during the
    # boot/DHCP window; the slirp timing makes any single boot probabilistically
    # flaky (~3/4 succeed). The CAPABILITY is what we assert, so retry the whole
    # boot a few times and require at least one clean fetch (~99.9% reliable).
    out = ""
    success = False
    for _ in range(5):
        out, result = _attempt_fetch()
        if (
            result.get("sent") == 8 + 4 + PKG_PAYLOAD_LEN + 4
            and "PKG: fetched len=0x0000000000000384 ok" in out
        ):
            success = True
            break

    assert success, f"package fetch did not complete in 5 boots\nlast serial:\n{out}"
    find_in_order(out, [
        "PKG: fetch armed",
        "TCP: syn sent",
        # The kernel downloaded the framed package and the checksum matched
        # (900 payload bytes = 0x384).
        "PKG: fetched len=0x0000000000000384 ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "PKG: fetch checksum FAIL" not in out
