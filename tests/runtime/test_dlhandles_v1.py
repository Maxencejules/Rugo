# Full-OS guide Part V.11 acceptance: POSIX dlopen HANDLE TABLE (sys_dlctl op4).
#
# The single-slot loader could keep only ONE shared object live at a time. This
# proves the handle table from ring 3: multidlprobe dlopen("libdl") TWICE -> two
# concurrently-live objects at two different randomized bases; op4
# dlsym_h(handle,"getval") resolves the SAME symbol against each SPECIFIC handle
# (two DISTINCT, independently-relocated, separately-callable VAs); dlclose(h1)
# frees ONLY that object, so h2 still resolves+runs while h1's handle returns -1.
# Prints "MULTIDL: handle table ok".
#
# Uses its OWN minimal app region (base-shell + multidlprobe): the shared 40-app
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
    disk = os.path.join(conftest.REPO_ROOT, "out", f"mdl-{uuid.uuid4().hex}.img")
    with open(disk, "wb") as f:
        f.write(b"\x00" * (1024 * 1024))
    cmd = [sys.executable, conftest.APP_DISK_V1_TOOL, "--disk", disk]
    for name in ("base-shell", "multidlprobe"):
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
            qcmd, conftest.NET_TIMEOUT, input_text="probe multidlprobe\nshutdown\n"
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


def test_dlopen_handle_table(find_in_order):
    out = _boot()
    find_in_order(out, [
        "EXEC: multidlprobe ok",
        "MULTIDL: handle table ok",
        "RUGO: halt ok",
    ])
    assert "MULTIDL: FAIL" not in out
    assert "USERPF" not in out
