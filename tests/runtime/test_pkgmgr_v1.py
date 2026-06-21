# Full-OS guide Part V.11 acceptance: a signed package-repository manager.
#
# Beyond the single-blob package fetch (test_pkgfetch_v1) and the sig-verify +
# install of one payload (test_pkg_install_v1), this exercises the package-manager
# CORE: an on-disk repo with an HMAC-signed index of multiple packages. At boot
# pkg_manager_selftest verifies the index signature, selects a package by name
# ("calc"), verifies that package's SHA-256, rejects a tampered payload AND a
# forged index, then installs the verified payload (write + read-back).
#
# The repo (tools/pkg_repo_v1.py) is seeded into the boot disk's scratch gap
# (index @LBA 24, payloads @LBA 25+), separate from the app region (@LBA 64+).

import os
import subprocess
import sys
import uuid

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import conftest  # noqa: E402
import pytest  # noqa: E402

PKG_REPO_TOOL = os.path.join(conftest.REPO_ROOT, "tools", "pkg_repo_v1.py")


def _boot_with_repo():
    iso = conftest.ISO_GO_PATH
    if not os.path.isfile(iso):
        pytest.skip(f"ISO not built: {iso}")
    if not conftest.QEMU_BIN:
        pytest.skip("qemu-system-x86_64 not found")
    disk = os.path.join(conftest.REPO_ROOT, "out", f"pkgmgr-{uuid.uuid4().hex}.img")
    conftest._ensure_app_region(disk)  # app region @64+ so the go lane boots normally
    # Seed the signed repo into the scratch gap (index @LBA 24, payloads @25+).
    subprocess.run(
        [sys.executable, PKG_REPO_TOOL, "--disk", disk,
         "--pkg", "calc:700", "--pkg", "edit:300", "--pkg", "term:512"],
        check=True, capture_output=True, text=True,
    )
    try:
        cmd = [
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
            cmd, conftest.NET_TIMEOUT, input_text="shutdown\n"
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


def test_pkg_manager_signed_repo(find_in_order):
    out = _boot_with_repo()
    find_in_order(out, [
        # The HMAC-signed index verified (3 packages) and a forged index is
        # rejected (checked inside the selftest before this marker prints).
        "PKGMGR: index count=0x0000000000000003 sig ok",
        # "calc" selected by name, its SHA-256 verified.
        "PKGMGR: select calc lba=0x0000000000000019 len=0x00000000000002BC hash ok",
        # A tampered payload is rejected, then the verified payload installs.
        "PKGMGR: tamper rejected",
        "PKGMGR: install ok",
        "PKGMGR: ok",
        "RUGO: halt ok",
    ])
    # No pkg-manager failure of any kind (index sig, forge, hash, tamper, install).
    for line in out.splitlines():
        if line.startswith("PKGMGR:"):
            assert "FAIL" not in line, line
    assert "PKGMGR: no repo" not in out
