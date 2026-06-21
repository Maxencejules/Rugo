# Full-OS guide Part IV.10 acceptance: multi-user authenticated login.
#
# sys_proc_ctl (id 51) op 5 = login(name, pw) verifies a credential against the
# kernel password database and, on success, assumes that account's uid. Unlike
# op 4 (setuid, root-only), login is an AUTHENTICATED privilege change: a regular
# user (uid 100) who knows the root password elevates to uid 0, while a wrong
# password is denied and audited (uid unchanged).


def test_login_authenticated_privilege_change(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("probe loginprobe\nshutdown\n").stdout

    # LOGINPROBE: ok proves the full flow inside one ring-3 app: getuid==100, a
    # wrong-password login is denied with uid unchanged, the correct root password
    # elevates to uid 0, and getuid then reports 0. (The deny/ok events are also
    # recorded in the audit ring, covered by test_audit_v1.)
    find_in_order(out, [
        "SPAWN: loginprobe",
        # Credentials live in a root-owned, owner-only /data/shadow store
        # (provisioned at boot); an unprivileged (uid 100) app is denied reading it.
        "LOGINPROBE: shadow protected ok",
        "LOGINPROBE: ok",
        # After LOGIN_LOCKOUT consecutive wrong root logins the account locks and
        # even the correct password is refused (online brute-force throttle).
        "LOGINPROBE: lockout ok",
        "RUGO: halt ok",
    ])
    assert "LOGINPROBE: FAIL" not in out
