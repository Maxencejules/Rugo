# Phase 8 acceptance: ls/cat/echo/ps run as REAL external programs from
# the package store, with arguments delivered by sys_spawn. Every visible
# output line below is produced by the spawned program's own ELF.


def _find_in_order(serial: str, markers: list[str]) -> None:
    pos = -1
    for marker in markers:
        pos = serial.find(marker, pos + 1)
        assert pos != -1, f"Missing '{marker}' in serial output.\nFull output:\n{serial}"


def test_coreutils_run_as_external_programs(qemu_go_c4_runtime):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot(
        "fsmk /data/etc\n"
        "fswrite /data/etc/motd args-and-files-work\n"
        "echo hello-from-exec\n"
        "cat /data/etc/motd\n"
        "ls /data\n"
        "ps\n"
        "shutdown\n"
    ).stdout

    # Anchors are single-write markers from the kernel and the spawned
    # programs themselves.
    _find_in_order(out, [
        "FSH: write ok",
        "EXEC: echo ok",
        "hello-from-exec",
        "EXEC: cat ok",
        "args-and-files-work",
        "EXEC: ls ok",
        "etc/",
        "EXEC: ps ok",
        "PS: tid 0x00",
        "PS: tid 0x01",
        "PS: tid 0x04",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert out.count("EXEC: echo ok") == 1
    assert out.count("EXEC: cat ok") == 1
    assert out.count("EXEC: ls ok") == 1
    assert out.count("EXEC: ps ok") == 1
    assert "APP: run err" not in out
    assert "EXEC: echo badhash" not in out
    assert "cat: error" not in out
    assert "ls: error" not in out
    assert "GOINIT: err" not in out
