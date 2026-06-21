# Full-OS guide Part I acceptance: PID namespace isolation (sys_nsctl, id 57).
#
# A PID namespace isolates a task's process view: after unsharing, the task sees
# only its own namespace (just itself, until it clones) and reads a
# namespace-LOCAL pid starting at 1 -- it becomes its namespace's "init".
#
# nsprobe (single client): ns_task_count BEFORE unshare returns the whole system
# (>1, services running); unshare_pid moves it into a fresh namespace; ns_task_count
# AFTER returns 1 (only itself); ns_getpid returns 1 (local pid != its global tid).
# Prints "NS: pid-namespace isolated ok".
#
# Uses its OWN minimal app region (base-shell + nsprobe): the shared 40-app region
# already fills the 1 MiB disk to the on-disk VFS boundary at sector 512.

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
    disk = os.path.join(conftest.REPO_ROOT, "out", f"pidns-{uuid.uuid4().hex}.img")
    with open(disk, "wb") as f:
        f.write(b"\x00" * (1024 * 1024))
    cmd = [sys.executable, conftest.APP_DISK_V1_TOOL, "--disk", disk]
    for name in ("base-shell", "nsprobe"):
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
            qcmd, conftest.NET_TIMEOUT, input_text="probe nsprobe\nshutdown\n"
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


def test_pid_namespace_isolation(find_in_order):
    out = _boot()
    find_in_order(out, [
        "EXEC: nsprobe ok",
        "NS: pid-namespace isolated ok",
        # UTS namespace: a fresh namespace inherits the global "rugo" hostname,
        # then sets its own ("ctr") -- the hostname view is namespace-scoped.
        "NS: uts-namespace hostname ok",
        "RUGO: halt ok",
    ])
    assert "NS: FAIL" not in out
    assert "USERPF" not in out
