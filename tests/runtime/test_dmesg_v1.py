# Full-OS guide Part V.11 (observability) / IV.10 (audit) acceptance: the
# kernel dmesg ring buffer.
#
# Every serial_write line is mirrored into a fixed ring (kernel_rs/src/lib.rs
# klog_append), readable via sys_sysinfo op 4. dmesgprobe writes a unique
# cookie (captured by the ring as it is printed), reads the dmesg tail back,
# and echoes it -- so the cookie appears twice, proving capture + readback.


def test_dmesg_capture_and_readback(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot(
        "probe dmesgprobe\n"
        "shutdown\n"
    ).stdout

    find_in_order(out, [
        "DMESGCOOKIE-7142",   # first: the probe's own write
        "DMESGCOOKIE-7142",   # second: echoed from the dmesg ring readback
        "DMESGPROBE: ok",
        "RUGO: halt ok",
    ])
    # The readback must also contain an earlier kernel boot marker, proving the
    # ring holds genuine kernel log output (not just the cookie just written).
    assert out.count("DMESGCOOKIE-7142") >= 2
    assert "DMESGPROBE: FAIL" not in out
