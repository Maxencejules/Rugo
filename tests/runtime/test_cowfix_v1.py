# Regression guard for two fork-time memory-isolation bugs:
#
#   P1: granting write to a forked CoW page via mprotect (sys_vm_ctl op 4) must
#       break the share (private copy) instead of writing through the frame the
#       forked sibling still maps. The child mprotect()s a shared page back to RW
#       and stores S2; the parent (after waitpid, so ordering is deterministic)
#       must still read its own S1. A leak proves mprotect aliased the frame.
#   P2: a forked child must inherit the parent's program break -- after the parent
#       grows it with brk(), the child's brk(0) must report the inherited break,
#       not reset to the base (address_space_fork cloned the grown heap pages).
#
# Like test_bigc_v1, this packs its OWN minimal app region (base-shell +
# cowfixprobe) rather than the shared 40-app region, which is already packed up
# to the on-disk VFS boundary at sector 512.

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
    for name in ("base-shell", "cowfixprobe"):
        elf = os.path.join(conftest.REPO_ROOT, "out", f"app-{name}.elf")
        if not os.path.isfile(elf):
            pytest.skip(f"app ELF not built: {elf}")
        cmd += ["--app", f"{name}={elf}"]
    subprocess.run(cmd, check=True, capture_output=True, text=True)


def test_fork_mprotect_cow_isolation_and_brk_inheritance(find_in_order):
    iso = conftest.ISO_GO_PATH
    if not os.path.isfile(iso):
        pytest.skip(f"ISO not built: {iso}")
    if not conftest.QEMU_BIN:
        pytest.skip("qemu-system-x86_64 not found")
    disk = os.path.join(conftest.REPO_ROOT, "out", f"cowfix-{uuid.uuid4().hex}.img")
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
            cmd, conftest.NET_TIMEOUT, input_text="probe cowfixprobe\nshutdown\n"
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

    # The child runs to completion (parent is blocked in waitpid), so its markers
    # precede the parent's verdict.
    find_in_order(out, [
        "EXEC: cowfixprobe ok",
        "COWFIX: child brk-inherited ok",
        "COWFIX: child mprotect-wrote",
        "COWFIX: parent mp-isolated ok",
        "RUGO: halt ok",
    ])
    # P1: the parent must not have observed the child's write through the frame.
    assert "COWFIX: parent mp-leak FAIL" not in out
    # P2: the child must not have reset its break to the base.
    assert "COWFIX: child brk-reset FAIL" not in out
    # No setup/wait failure, no fault.
    assert "COWFIX: FAIL setup" not in out
    assert "COWFIX: FAIL waiterr" not in out
    assert "USERPF" not in out
