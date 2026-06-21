# Full-OS guide Part I acceptance: thread-local storage (sys_vm_ctl op 5 = set_tls).
#
# The kernel gives each task its own %fs base, restored on every resume by
# r4_switch_to, so FS-relative addressing (%fs:offset) reaches a per-task TLS
# block. tlsprobe sets fs.base to a buffer, writes a magic via [fs:0], confirms it
# aliases the buffer, then YIELDS (forcing a context switch to other tasks whose
# fs.base is 0) and re-reads [fs:0] -- which only still holds the magic if the
# kernel restored THIS task's fs.base on resume. Prints "TLS: fs-base tls ok".
#
# Uses its OWN minimal app region (base-shell + tlsprobe): the shared 40-app
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
    disk = os.path.join(conftest.REPO_ROOT, "out", f"tls-{uuid.uuid4().hex}.img")
    with open(disk, "wb") as f:
        f.write(b"\x00" * (1024 * 1024))
    cmd = [sys.executable, conftest.APP_DISK_V1_TOOL, "--disk", disk]
    for name in ("base-shell", "tlsprobe"):
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
            qcmd, conftest.NET_TIMEOUT, input_text="probe tlsprobe\nshutdown\n"
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


def test_thread_local_storage(find_in_order):
    out = _boot()
    find_in_order(out, [
        "EXEC: tlsprobe ok",
        "TLS: fs-base tls ok",
        "RUGO: halt ok",
    ])
    assert "TLS: FAIL" not in out
    assert "USERPF" not in out
