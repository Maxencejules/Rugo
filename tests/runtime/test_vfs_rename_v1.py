# Full-OS guide Part II.5 acceptance: SimpleFS rename.
#
# At boot the kernel runs a net-neutral rename self-test against the live SimpleFS:
# it creates a scratch file, writes a marker, renames it within /data, and confirms
# the NEW name resolves to the SAME node with the content intact while the OLD name
# no longer resolves -- then removes it, leaving the on-disk FS exactly as found (so
# the persisted-VFS tests that reuse the disk are unaffected). The node keeps its
# kind / mode / owner / data blocks / size; only the name changes, and the node-table
# write is journaled atomically with the superblock. "VFS: rename ok".


def test_vfs_rename(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    find_in_order(out, [
        "VFS: rename ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "VFS: rename fail" not in out
