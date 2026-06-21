# Full-OS guide Part II.5 acceptance: SimpleFS write-ahead metadata journal.
#
# The live SimpleFS now journals its metadata writes (node table, bitmap,
# superblock) for crash-consistency: a mutation (vfs_write/vfs_mkdir/vfs_unlink)
# logs its metadata sectors to a journal region with a committed header (the atomic
# commit point), applies them, then clears the header; vfs_mount replays a
# committed-but-unapplied journal. vfs_journal_selftest synthesizes exactly the
# on-disk state a crash between commit and clear leaves -- a committed header + a
# data slot targeting a scratch sector -- runs the replay a post-crash mount would,
# and confirms the payload landed and the header cleared ("VFS: journal ok"). The
# live journaled write path itself is exercised by test_vfs_runtime_v1 (every
# write/mkdir/unlink there now goes through a transaction).


def test_vfs_metadata_journal_replay(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    find_in_order(out, [
        # The FS mounts (format on a fresh disk, or mount + replay on a reused one).
        "VFS:",
        # The journal replay (crash-recovery) path is proven on the live journal region.
        "VFS: journal ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "VFS: journal FAIL" not in out
    assert "GOINIT: err" not in out
