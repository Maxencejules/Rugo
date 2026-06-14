# Full-OS guide Part II.5 acceptance: in-memory tmpfs (/tmp).
#
# Writes a /tmp file and reads it back within the same boot (via the shell's
# fswrite/fscat builtins). tmpfs is heap-free and lost on reboot.


def test_tmpfs_write_then_read(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot(
        "fswrite /tmp/note hello-tmpfs\n"
        "fscat /tmp/note\n"
        "shutdown\n"
    ).stdout

    find_in_order(out, [
        "FSH: write ok",
        "hello-tmpfs",
        "FSH: cat ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "FSH: err" not in out
    assert "GOINIT: err" not in out
