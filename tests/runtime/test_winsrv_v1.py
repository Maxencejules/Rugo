# Full-OS guide Part III acceptance: standing window-server PERSISTENT surface
# registry with per-client ownership + lifecycle.
#
# op 4 composites a single client's throwaway list. A standing window server holds
# a PERSISTENT registry of multiple clients' windows and owns their lifecycle.
# sys_ioctl gains op 8 wm_register (owner-stamped, persistent), op 9 wm_compose
# (z-order the whole registry), op 10 wm_clear (owner-checked).
#
#  - wmprobe: registers two windows (red z=0, blue z=1), composes (=2), clears one,
#    composes (=1), then EXITS leaving the other registered.
#  - wmcheck (a DIFFERENT client): composes and must see 0 -- the kernel removed the
#    exited owner's window (per-client exit-cleanup), proving the registry is a
#    server-owned lifecycle, not a per-call list.
#
# Uses its OWN minimal app region (base-shell + the two probes): the shared 40-app
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
    disk = os.path.join(conftest.REPO_ROOT, "out", f"wm-{uuid.uuid4().hex}.img")
    with open(disk, "wb") as f:
        f.write(b"\x00" * (1024 * 1024))
    cmd = [sys.executable, conftest.APP_DISK_V1_TOOL, "--disk", disk]
    for name in ("base-shell", "wmprobe", "wmcheck"):
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
            qcmd, conftest.NET_TIMEOUT,
            input_text="probe wmprobe\nprobe wmcheck\nshutdown\n",
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


def test_window_server_registry_lifecycle(find_in_order):
    out = _boot()
    find_in_order(out, [
        "EXEC: wmprobe ok",
        "WM: registry compose=2 afterclear=1 ok",
        "EXEC: wmcheck ok",
        "WMCHECK: after-owner-exit=0 ok",
        "RUGO: halt ok",
    ])
    assert "WM: FAIL" not in out
    assert "WMCHECK: FAIL" not in out
    assert "USERPF" not in out
