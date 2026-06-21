# Full-OS guide Part V (observability) acceptance: sysinfo metrics
# (sys_sysinfo id 61).
#
# `sysinfoprobe` reads the live task count (>=1), free physical frames
# (>0), and uptime ticks (must advance across a busy interval).


def test_sysinfo_metrics(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("probe sysinfoprobe\nshutdown\n").stdout

    find_in_order(out, [
        "SYSINFOPROBE: ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "SYSINFOPROBE: FAIL" not in out
    assert "GOINIT: err" not in out
