# Full-OS guide Part II.5 acceptance: SimpleFS truncate (shrink).
#
# At boot the kernel runs a net-neutral truncate self-test against the live SimpleFS:
# it creates a 3-block scratch file with a per-byte pattern, shrinks it to one block,
# and confirms (a) the new size, (b) the surviving first block's bytes are intact,
# (c) reads past the new size are empty, and (d) exactly the two trailing data blocks
# were returned to the free bitmap -- then removes it, leaving the on-disk FS exactly
# as found (so the persisted-VFS tests that reuse the disk are unaffected). The freed
# blocks + node entry are journaled atomically. "VFS: truncate ok".


def test_vfs_truncate(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    find_in_order(out, [
        "VFS: truncate ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "VFS: truncate fail" not in out
