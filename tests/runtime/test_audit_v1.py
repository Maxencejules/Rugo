# Full-OS guide Part IV.10 acceptance: security audit log.
#
# The kernel records denied/privileged security events in a ring distinct from
# dmesg, readable via sys_sysinfo op 7. auditprobe (an external app with only
# the STORAGE capability) calls sys_net_query (needs NETWORK) -> capability
# denial -> audit record; it then reads the audit ring back and echoes it, so
# the denial event for its own task appears.


def test_security_audit_log(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("probe auditprobe\nshutdown\n").stdout

    find_in_order(out, [
        # net_query is syscall 49 = 0x31; recorded with the caller's tid.
        "AUDIT: cap-deny nr=0x0000000000000031 tid=0x",
        "AUDITPROBE: ok",
        "RUGO: halt ok",
    ])
    assert "AUDITPROBE: FAIL" not in out


def test_audit_checkpoints(qemu_go_c4_runtime):
    # Full-OS guide Part IV.10: the audit ring now records additional security
    # checkpoints -- the sandbox-deny gate (a sandboxed task probing a filtered
    # syscall) and the power path (privileged shutdown/reboot). The boot self-test
    # drives the same audit_event calls those sites make and confirms each lands in
    # the ring with the expected tag + syscall nr, read back the way op 7 exposes it.
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    assert "AUDIT: checkpoints ok" in out
    assert "AUDIT: checkpoints fail" not in out
