# Full-OS guide Part II.5 acceptance: /proc/self/stat pseudo-file.
#
# `fscat /proc/self/stat` reads the caller's generated stat line. The reader
# is the shell (uid 0), so the line carries its tid and uid=0.


def test_proc_self_stat(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("fscat /proc/self/stat\nshutdown\n").stdout

    find_in_order(out, [
        "tid=0x",
        "FSH: cat ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "uid=0x0000000000000000" in out  # shell runs as root
    assert "state=run" in out
    assert "FSH: err" not in out
    assert "GOINIT: err" not in out
