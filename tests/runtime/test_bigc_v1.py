# Toolchain + exec-loader acceptance: a C program that spans MORE THAN TWO pages.
#
# The host gcc/ld only target PE-COFF, so C apps compile -mabi=sysv, link in PE,
# then get rewrapped to ET_EXEC ELF by tools/pe_to_elf_v1.py. page3probe proves
# the *kernel* loader maps a 3-page image; this proves the *C toolchain* path end
# to end: bigcprobe carries a ~8 KiB initialized const table (laid into .rdata so
# the image spans ~6 pages) plus an 8 KiB array. At runtime it checksums the
# whole table (every initialized page must have loaded), reads a high-index
# element that lives on the 4th+ page, verifies the array reads back zero then
# writable, and calls a function only reached by address (mingw .refptr
# indirection surviving the rewrap). A single wrong/missing page corrupts the
# checksum -> "BIGC: FAIL".
#
# This test packs its OWN app region (base-shell + bigcprobe only): bigcprobe is
# ~41 sectors, and the shared 40-app region (tests/conftest.py APP_REGION_APPS)
# is already filled up to the on-disk VFS boundary at sector 512, so a 41-sector
# app appended there would overlap the VFS region (clobbered at boot). A real
# install sizes the store well above EXEC_APP_MAX_BYTES (64 KiB); the 1 MiB test
# disk is the only thing that is tight, so this test uses a minimal store.

import os
import subprocess
import sys
import uuid

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import conftest  # noqa: E402
import pytest  # noqa: E402


def _build_region(disk_path):
    with open(disk_path, "wb") as f:
        f.write(b"\x00" * (1024 * 1024))
    cmd = [sys.executable, conftest.APP_DISK_V1_TOOL, "--disk", disk_path]
    for name in ("base-shell", "bigcprobe"):
        elf = os.path.join(conftest.REPO_ROOT, "out", f"app-{name}.elf")
        if not os.path.isfile(elf):
            pytest.skip(f"app ELF not built: {elf}")
        cmd += ["--app", f"{name}={elf}"]
    subprocess.run(cmd, check=True, capture_output=True, text=True)


def test_exec_multipage_c_app(find_in_order):
    iso = conftest.ISO_GO_PATH
    if not os.path.isfile(iso):
        pytest.skip(f"ISO not built: {iso}")
    if not conftest.QEMU_BIN:
        pytest.skip("qemu-system-x86_64 not found")
    disk = os.path.join(conftest.REPO_ROOT, "out", f"bigc-{uuid.uuid4().hex}.img")
    _build_region(disk)
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
        out = conftest._run_qemu_capture(
            cmd, conftest.NET_TIMEOUT, input_text="probe bigcprobe\nshutdown\n"
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

    find_in_order(out, [
        "EXEC: bigcprobe ok",
        "BIGC: ok sum=0x14070400 high=0xe1f2a765 pages>2",
        "BIGC: done",
        "RUGO: halt ok",
    ])
    assert "BIGC: FAIL" not in out
    assert "USERPF" not in out
