# Full-OS guide Part IV.10 (security) acceptance: sandbox / syscall
# allowlist (sys_sandbox id 59).
#
# `sandboxprobe` narrows its own allowlist to syscall 0, then attempts
# sys_yield (3), which the kernel denies (-1). Proves per-task pledge-style
# restriction with monotonic narrowing.


def test_sandbox_denies_filtered_syscall(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("probe sandboxprobe\nshutdown\n").stdout

    find_in_order(out, [
        "SANDBOX: deny nr=0x0000000000000003",
        "SANDBOXPROBE: denied ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "SANDBOXPROBE: FAIL" not in out
    assert "GOINIT: err" not in out
