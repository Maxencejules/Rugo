# ABI v3.x id 55 acceptance: sys_epoll readiness. The epollprobe app creates an
# epoll instance and a pipe, registers the pipe read end for EPOLLIN, and proves
# LEVEL-TRIGGERED readiness inside one ring-3 program: while the pipe is empty
# epoll_wait reports nothing ready; after one byte is written it reports the read
# end ready with EPOLLIN set. This is a real runtime syscall path (not a report),
# replacing the prior epoll deferral in the go lane.
#
# Own minimal app disk (base-shell + epollprobe), like test_bigc/test_clonebrk,
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
    for name in ("base-shell", "epollprobe"):
        elf = os.path.join(conftest.REPO_ROOT, "out", f"app-{name}.elf")
        if not os.path.isfile(elf):
            pytest.skip(f"app ELF not built: {elf}")
        cmd += ["--app", f"{name}={elf}"]
    subprocess.run(cmd, check=True, capture_output=True, text=True)


def _boot(input_text):
    iso = conftest.ISO_GO_PATH
    if not os.path.isfile(iso):
        pytest.skip(f"ISO not built: {iso}")
    if not conftest.QEMU_BIN:
        pytest.skip("qemu-system-x86_64 not found")
    disk = os.path.join(conftest.REPO_ROOT, "out", f"epoll-{uuid.uuid4().hex}.img")
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
        return conftest._run_qemu_capture(
            cmd, conftest.NET_TIMEOUT, input_text=input_text
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


def test_epoll_level_triggered_readiness(find_in_order):
    out = _boot("probe epollprobe\nshutdown\n")

    find_in_order(out, [
        "SPAWN: epollprobe",
        "EPOLLPROBE: ready ok",
        "RUGO: halt ok",
    ])
    assert "EPOLLPROBE: FAIL" not in out
    assert "EXEC: epollprobe badpkg" not in out


def test_epoll_instances_reclaimed_on_task_exit(find_in_order):
    # Resource-lifecycle: each epollprobe run creates an epoll instance and
    # exits WITHOUT closing it (op4). EPOLLS is a process-global table of
    # EPOLL_MAX=4 slots, so unless task-exit cleanup reclaims the creator's
    # slot, 4 such early-exits permanently exhaust epoll_create for the rest
    # of the boot and the 5th run fails on create. Running epollprobe 5 times
    # (> EPOLL_MAX) must therefore succeed every time iff exiting tasks release
    # their epoll instances.
    runs = 5
    out = _boot("probe epollprobe\n" * runs + "shutdown\n")

    assert "EPOLLPROBE: FAIL" not in out
    assert out.count("EPOLLPROBE: ready ok") == runs, (
        f"expected {runs} successful epollprobe runs, "
        f"got {out.count('EPOLLPROBE: ready ok')} "
        f"(epoll slots leaked on exit -> create exhausted)"
    )
