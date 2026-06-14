# Full-OS guide Part IV.10 (security) acceptance: getrandom (sys id 54).
#
# `rngprobe` draws two 16-byte buffers and proves the pool produced output
# (not all zero) and advances (the two draws differ).


def test_getrandom_produces_varying_bytes(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("probe rngprobe\nshutdown\n").stdout

    find_in_order(out, [
        "RNGPROBE: ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "RNGPROBE: FAIL" not in out
    assert "GOINIT: err" not in out
