# Full-OS guide Part II.5 acceptance: mount table (prefix -> filesystem registry).
#
# At boot the kernel runs a mount-table self-test: it registers overlapping mounts
# (/, /data, /mnt, /data/special) and confirms longest-prefix matching on
# component boundaries -- "/data/file"->SimpleFS, "/mnt/HELLO.TXT"->FAT,
# "/data/special/x"->the nested mount (longest wins), "/other"->root fallback, and
# crucially "/database/x"->root NOT /data (the prefix must end on a '/' boundary).


def test_mount_table(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    find_in_order(out, [
        "MOUNT: table ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "MOUNT: table fail" not in out
