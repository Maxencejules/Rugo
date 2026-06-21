# Regression guard for cross-address-space waitpid status delivery (a bug an
# adversarial review found in the per-process-address-space work: r4_wake_waiter
# wrote the exit status against the wrong CR3).
#
# `waitprobe` is a spawned app (private address space) that forks a child and
# waitpid()s for it with a status pointer. The kernel must deliver the status
# into the PARENT's address space, not the exiting child's / SHARED table.


def test_waitpid_status_delivered_to_caller_address_space(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("probe waitprobe\nshutdown\n").stdout

    find_in_order(out, [
        "WAITPROBE: status ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "WAITPROBE: FAIL" not in out
    assert "GOINIT: err" not in out
