# Full-OS guide Part V.11 acceptance: lseek (sys_fs_ctl op 6), all three whences.
#
# `lseekprobe` writes "ABCDE" to a /data file, reopens it, and exercises every
# whence: SEEK_SET 2 -> reads "CDE"; SEEK_CUR -3 (from offset 5) -> reads "C";
# SEEK_END -2 (file size 5) -> reads "DE". The single "LSEEKPROBE: ok" marker is
# printed only if all three seeks + reads matched (any mismatch prints FAIL).


def test_lseek_all_whences(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("probe lseekprobe\nshutdown\n").stdout

    find_in_order(out, [
        "LSEEKPROBE: ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "LSEEKPROBE: FAIL" not in out
    assert "GOINIT: err" not in out
