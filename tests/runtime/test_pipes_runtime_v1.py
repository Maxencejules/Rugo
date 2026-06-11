# Phase 8b acceptance: pipe IPC. `cat file | wc` joins two real external
# programs through a kernel pipe: cat writes the file into the pipe's
# write end (handed over by sys_spawn), wc reads the pipe until EOF and
# reports the byte count. The count only comes out right if the bytes
# actually crossed the kernel ring.


def test_pipe_joins_two_external_programs(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    # payload: 16 bytes -> WC: 0x10
    out = boot(
        "fsmk /data/etc\n"
        "fswrite /data/etc/motd pipe-payload-123\n"
        "cat /data/etc/motd | wc\n"
        "shutdown\n"
    ).stdout

    find_in_order(out, [
        "FSH: write ok",
        "EXEC: cat ok",
        "EXEC: wc ok",
        "WC: 0x10 bytes",
        "GOSH: pipe ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "APP: run err" not in out
    assert "wc: error" not in out
    assert "cat: error" not in out
    # The file content must NOT appear on the console: it went into the
    # pipe, not to debug output. The only occurrence is the echoed
    # fswrite command itself.
    assert out.count("pipe-payload-123") == 1
    assert "GOINIT: err" not in out
