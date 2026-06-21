# Full-OS guide Part I.3 acceptance: clone + futex (sys_proc_ctl clone op 2,
# sys_futex id 52).
#
# `futexprobe` clones a thread (shared address space), blocks in futex_wait
# on a shared word, and the child sets the word and wakes it. Proves clone
# shares memory and futex wait/wake hand off correctly.


def test_clone_futex_wait_wake(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("probe futexprobe\nshutdown\n").stdout

    find_in_order(out, [
        "FUTEX: wait tid=0x",
        "FUTEX: wake n=0x0000000000000001",
        "FUTEXPROBE: woken ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "FUTEXPROBE: FAIL" not in out
    assert "R4: deadlock" not in out
    assert "GOINIT: err" not in out
