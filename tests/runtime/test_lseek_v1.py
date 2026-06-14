# Full-OS guide Part V.11 acceptance: lseek (sys_fs_ctl op 6).
#
# `lseekprobe` writes "ABCDE" to a /data file, reopens it, seeks to offset
# 2, reads 3 bytes, and verifies it got "CDE".


def test_lseek_seek_set(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("probe lseekprobe\nshutdown\n").stdout

    find_in_order(out, [
        "LSEEKPROBE: ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "LSEEKPROBE: FAIL" not in out
    assert "GOINIT: err" not in out
