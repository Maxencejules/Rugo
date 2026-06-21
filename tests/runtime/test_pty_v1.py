# Full-OS guide Part V.11 acceptance: pty (pseudo-terminal) pair.
#
# openpty (sys_ioctl op 2) returns a master/slave fd pair backed by one PtyObj
# (two rings). Bytes written to the master are readable from the slave and
# vice versa. ptyprobe writes "ptyhello" to the master and reads it from the
# slave, then writes "ptyback!" to the slave and reads it from the master,
# echoing each read so the transcript proves bidirectional delivery.


def test_pty_bidirectional(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot(
        "probe ptyprobe\n"
        "shutdown\n"
    ).stdout

    find_in_order(out, [
        "ptyhello",          # master -> slave
        "ptyback!",          # slave -> master
        "PTYPROBE: ok",
        "RUGO: halt ok",
    ])
    assert "PTYPROBE: FAIL" not in out


def test_pty_pool_recycled_on_exit(qemu_go_c4_runtime, find_in_order):
    """ptyprobe opens a pty and exits WITHOUT closing it; the PtyObj pool
    (PTY_MAX=2) must be recycled on task exit, so three sequential runs all
    succeed. Without the exit-time pty_drop_end fix the third run fails."""
    boot, _disk_path = qemu_go_c4_runtime

    out = boot(
        "probe ptyprobe\n"
        "probe ptyprobe\n"
        "probe ptyprobe\n"
        "shutdown\n"
    ).stdout

    assert out.count("PTYPROBE: ok") == 3
    assert "PTYPROBE: FAIL" not in out
