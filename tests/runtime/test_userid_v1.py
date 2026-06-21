# Full-OS guide Part IV.10 acceptance: multi-user uid privilege model.
#
# sys_proc_ctl op 3 = getuid, op 4 = setuid. An external app runs as uid 100, so
# getuid returns 100 and setuid is DENIED (only root may change uid). userprobe
# exercises getuid -> setuid(0) denied -> getuid unchanged.


def test_uid_privilege_model(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("probe userprobe\nshutdown\n").stdout

    find_in_order(out, [
        "USERPROBE: uid=100 setuid-denied ok",
        "RUGO: halt ok",
    ])
    assert "USERPROBE: FAIL" not in out
