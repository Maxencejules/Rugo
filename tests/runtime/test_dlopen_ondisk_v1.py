# Full-OS guide Part V.11 acceptance: dlopen a shared object FROM THE FILESYSTEM.
#
# Beyond the kernel-embedded libdl, the dynamic linker can now load a .so read off
# the VFS. At boot the kernel seeds /data/dltest.so (a copy of the embedded
# module). ondlprobe dlopen("/data/dltest.so") -> reads it via vfs_lookup/vfs_read
# into a kernel buffer -> links it (RELATIVE + GLOB_DAT + JUMP_SLOT) -> dlsym +
# calls addtwo(40)==42 and getval()==42 (the latter only returns 42 if the on-disk
# image's relocation was applied), printing "ONDISKDL: ok".
#
# Uses its OWN minimal app region (base-shell + ondlprobe): the shared 40-app
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
    disk = os.path.join(conftest.REPO_ROOT, "out", f"ondl-{uuid.uuid4().hex}.img")
    with open(disk, "wb") as f:
        f.write(b"\x00" * (1024 * 1024))
    cmd = [sys.executable, conftest.APP_DISK_V1_TOOL, "--disk", disk]
    for name in ("base-shell", "ondlprobe"):
        elf = os.path.join(conftest.REPO_ROOT, "out", f"app-{name}.elf")
        if not os.path.isfile(elf):
            pytest.skip(f"app ELF not built: {elf}")
        cmd += ["--app", f"{name}={elf}"]
    subprocess.run(cmd, check=True, capture_output=True, text=True)
    # Arm the on-disk dlopen seed: the kernel writes /data/dltest.so only when this
    # "DLSEED" marker is present (LBA 17), so ordinary go-lane boots never touch the
    # VFS. Written after the app-region tool, which only patches sector 64+.
    with open(disk, "r+b") as f:
        f.seek(17 * 512)
        f.write(b"DLSEED\x00\x00")
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
            qcmd, conftest.NET_TIMEOUT, input_text="probe ondlprobe\nshutdown\n"
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


def test_dlopen_from_filesystem(find_in_order):
    out = _boot()
    find_in_order(out, [
        "VFS: format ok",
        "EXEC: ondlprobe ok",
        "ONDISKDL: ok",
        "RUGO: halt ok",
    ])
    assert "ONDISKDL: FAIL" not in out
    assert "USERPF" not in out
