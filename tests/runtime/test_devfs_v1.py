# Full-OS guide Part II.5 acceptance: /dev character devices (pseudo-fs).
#
# `devprobe` opens /dev/zero (reads all-zero), /dev/urandom (reads vary
# from zero), and /dev/null (writes accepted and discarded).


def test_dev_character_devices(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("probe devprobe\nshutdown\n").stdout

    find_in_order(out, [
        "DEVPROBE: ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "DEVPROBE: FAIL" not in out
    assert "GOINIT: err" not in out
