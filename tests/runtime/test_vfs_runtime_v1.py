# Phase 5 acceptance: a writable on-disk file tree with directories.
# Live runtime evidence - files created through the shell persist across
# a reboot on the same disk.


def _find_in_order(serial: str, markers: list[str]) -> None:
    pos = -1
    for marker in markers:
        pos = serial.find(marker, pos + 1)
        assert pos != -1, f"Missing '{marker}' in serial output.\nFull output:\n{serial}"


def test_vfs_tree_create_list_persist(qemu_go_c4_runtime):
    boot, _disk_path = qemu_go_c4_runtime

    first = boot(
        "fsmk /data/etc\n"
        "fswrite /data/etc/motd hello-rugo\n"
        "fscat /data/etc/motd\n"
        "fsls /data\n"
        "fsls /data/etc\n"
        "shutdown\n"
    ).stdout
    _find_in_order(first, [
        "VFS: format ok",
        "rugo> fsmk /data/etc",
        "FSH: mkdir ok",
        "rugo> fswrite /data/etc/motd hello-rugo",
        "FSH: write ok",
        "rugo> fscat /data/etc/motd",
        "hello-rugo",
        "FSH: cat ok",
        "rugo> fsls /data",
        "etc/",
        "FSH: ls ok",
        "rugo> fsls /data/etc",
        "motd",
        "FSH: ls ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "FSH: err" not in first
    assert "VFS: io err" not in first

    second = boot(
        "fscat /data/etc/motd\n"
        "fsrm /data/etc/motd\n"
        "fscat /data/etc/motd\n"
        "shutdown\n"
    ).stdout
    _find_in_order(second, [
        "VFS: mount ok files=0x0000000000000002",
        "rugo> fscat /data/etc/motd",
        "hello-rugo",
        "FSH: cat ok",
        "rugo> fsrm /data/etc/motd",
        "FSH: rm ok",
        "rugo> fscat /data/etc/motd",
        "FSH: err",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "VFS: format ok" not in second
    assert second.count("FSH: err") == 1
