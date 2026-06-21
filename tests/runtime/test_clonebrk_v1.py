# Regression guard for per-address-space brk() across clone (sys_proc_ctl op2)
# threads. POSIX brk is a property of the address space, so all threads sharing a
# pml4 must observe one break. The kernel enforces this with copy-at-clone (a new
# thread starts at the caller's current break) + propagate-on-write (every brk
# change is reflected to all siblings via as_set_heap_brk).
#
# The probe: main grows its break, clones a thread, and futex-waits. The clone
# checks brk(0) returns the INHERITED break (copy-at-clone), grows it further,
# then wakes main. Main checks brk(0) now returns the clone's grown value
# (propagate-on-write). A regression of either mechanism flips a marker to FAIL.
#
# Own minimal app disk (base-shell + clonebrkprobe), like test_bigc/test_cowfix,
# so it does not consume the shared 40-app region (packed to the sector-512 VFS
# boundary).

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
    for name in ("base-shell", "clonebrkprobe"):
        elf = os.path.join(conftest.REPO_ROOT, "out", f"app-{name}.elf")
        if not os.path.isfile(elf):
            pytest.skip(f"app ELF not built: {elf}")
        cmd += ["--app", f"{name}={elf}"]
    subprocess.run(cmd, check=True, capture_output=True, text=True)


def test_brk_is_per_address_space_across_clone(find_in_order):
    iso = conftest.ISO_GO_PATH
    if not os.path.isfile(iso):
        pytest.skip(f"ISO not built: {iso}")
    if not conftest.QEMU_BIN:
        pytest.skip("qemu-system-x86_64 not found")
    disk = os.path.join(conftest.REPO_ROOT, "out", f"clonebrk-{uuid.uuid4().hex}.img")
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
            cmd, conftest.NET_TIMEOUT, input_text="probe clonebrkprobe\nshutdown\n"
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

    # The clone checks inheritance first, then main (after futex wake) checks it
    # saw the clone's growth.
    find_in_order(out, [
        "EXEC: clonebrkprobe ok",
        "CLONEBRK: clone inherited ok",
        "CLONEBRK: main saw-shared ok",
        "RUGO: halt ok",
    ])
    # copy-at-clone regression:
    assert "CLONEBRK: clone inherited FAIL" not in out
    # propagate-on-write regression:
    assert "CLONEBRK: main saw-shared FAIL" not in out
    assert "CLONEBRK: FAIL clone-err" not in out
    assert "USERPF" not in out
