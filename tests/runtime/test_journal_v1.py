# Full-OS guide Part II.5 acceptance: filesystem journaling (crash consistency).
#
# sys_sysinfo op 10 write-ahead-logs a target write to the journal (data sector
# + committed header), verifies the target is NOT yet applied (simulating a
# crash before commit), then replays the journal and confirms the target now
# holds the logged data. journalprobe drives it.


def test_fs_journal_replay(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("probe journalprobe\nshutdown\n").stdout

    find_in_order(out, [
        "JOURNAL: replay ok",
        "JOURNALPROBE: ok",
        "RUGO: halt ok",
    ])
    assert "JOURNALPROBE: FAIL" not in out
