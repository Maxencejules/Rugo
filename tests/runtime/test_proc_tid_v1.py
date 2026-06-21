# Full-OS guide Part II.5 acceptance: /proc/<tid>/stat for an ARBITRARY task.
#
# Beyond /proc/self/stat (the caller's own line), the kernel now serves
# /proc/<tid>/stat for any live task -- the per-task introspection ps-style
# tooling needs. `fscat /proc/0/stat` (the shell, reading the init task tid 0,
# not itself) must return that task's generated stat line (tid/uid/state).


def test_proc_tid_stat(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("fscat /proc/0/stat\nshutdown\n").stdout

    find_in_order(out, [
        # The requested tid (0) round-trips -- proof it looked up an arbitrary
        # task, not the caller (the shell is not tid 0).
        "tid=0x0000000000000000",
        "FSH: cat ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "uid=0x" in out
    assert "state=" in out
    assert "FSH: err" not in out
    assert "GOINIT: err" not in out
