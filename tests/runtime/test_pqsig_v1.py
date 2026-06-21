# Full-OS guide Part IV.10 acceptance: PUBLIC-KEY (Lamport) signature verify.
#
# The old package-integrity scheme used a SYMMETRIC keyed hash (HMAC-SHA256): the
# kernel held the key and could therefore forge a signature. This replaces it
# with a genuine asymmetric Lamport one-time signature -- the kernel embeds ONLY
# the public key (256 pairs of SHA-256 hashes, tools/lamport_keygen_v1.py) and a
# reference signature, so it can verify but never forge.
#
# Two proofs in one boot:
#  - boot self-test: "PQSIG: lamport verify ok, forgery rejected"
#  - ring-3 (sys_sigverify id 63): pqsigprobe accepts the genuine signature and
#    rejects BOTH a tampered message and a tampered signature ("PQSIGAPP: verify
#    ok forge rejected").
#
# Uses its OWN minimal app region (base-shell + pqsigprobe): the shared 40-app
# region already fills the 1 MiB disk to the on-disk VFS boundary at sector 512.

import os
import subprocess
import sys
import uuid

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import conftest  # noqa: E402
import pytest  # noqa: E402


def _boot():
    iso = conftest.ISO_GO_PATH
    if not os.path.isfile(iso):
        pytest.skip(f"ISO not built: {iso}")
    if not conftest.QEMU_BIN:
        pytest.skip("qemu-system-x86_64 not found")
    disk = os.path.join(conftest.REPO_ROOT, "out", f"pqsig-{uuid.uuid4().hex}.img")
    with open(disk, "wb") as f:
        f.write(b"\x00" * (1024 * 1024))
    cmd = [sys.executable, conftest.APP_DISK_V1_TOOL, "--disk", disk]
    for name in ("base-shell", "pqsigprobe"):
        elf = os.path.join(conftest.REPO_ROOT, "out", f"app-{name}.elf")
        if not os.path.isfile(elf):
            pytest.skip(f"app ELF not built: {elf}")
        cmd += ["--app", f"{name}={elf}"]
    subprocess.run(cmd, check=True, capture_output=True, text=True)
    try:
        qcmd = [
            conftest.QEMU_BIN,
            "-machine", "q35", "-cpu", "qemu64", "-m", "128",
            "-serial", "stdio", "-display", "none", "-no-reboot",
            "-boot", "d",
            "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
            "-cdrom", iso,
            "-drive", f"file={disk},format=raw,if=none,id=disk0",
            "-device", "virtio-blk-pci,drive=disk0,disable-modern=on",
            "-netdev", "user,id=n0",
            "-device", "virtio-net-pci,netdev=n0,disable-modern=on",
        ]
        return conftest._run_qemu_capture(
            qcmd, conftest.NET_TIMEOUT, input_text="probe pqsigprobe\nshutdown\n"
        ).stdout
    finally:
        for _ in range(20):
            try:
                if os.path.isfile(disk):
                    os.remove(disk)
                break
            except PermissionError:
                import time

                time.sleep(0.25)


def test_public_key_signature_verify(find_in_order):
    out = _boot()
    find_in_order(out, [
        "PQSIG: lamport verify ok, forgery rejected",
        "EXEC: pqsigprobe ok",
        "PQSIGAPP: verify ok forge rejected",
        "RUGO: halt ok",
    ])
    assert "PQSIG: lamport FAIL" not in out
    assert "PQSIGAPP: FAIL" not in out
    assert "USERPF" not in out
