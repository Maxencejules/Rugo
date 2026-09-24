# Full-OS guide Part IV.10 acceptance: multi-user authenticated login.
#
# sys_proc_ctl (id 51) op 5 = login(name, pw) verifies a credential against the
# kernel password database and, on success, assumes that account's uid. Unlike
# op 4 (setuid, root-only), login is an AUTHENTICATED privilege change: a regular
# user (uid 100) who knows the root password elevates to uid 0, while a wrong
# password is denied and audited (uid unchanged).
#
# Like test_cowfix_v1, this packs its OWN minimal app region (base-shell +
# loginprobe) rather than the shared app region, which is packed up to the
# on-disk VFS boundary at sector 512: as that region's last app, loginprobe ran
# past the boundary and was overwritten by the boot-time VFS format
# (EXEC: loginprobe badhash).

import os
import sys
import uuid

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import conftest  # noqa: E402
import pytest  # noqa: E402


def test_login_authenticated_privilege_change(find_in_order):
    iso = conftest.ISO_GO_PATH
    if not os.path.isfile(iso):
        pytest.skip(f"ISO not built: {iso}")
    disk = os.path.join(conftest.REPO_ROOT, "out", f"login-{uuid.uuid4().hex}.img")
    try:
        conftest._ensure_app_region(disk, apps=("base-shell", "loginprobe"))
        out = conftest._boot_iso_with_disk_and_net(
            iso, disk, input_text="probe loginprobe\nshutdown\n"
        ).stdout
    finally:
        if os.path.isfile(disk):
            os.remove(disk)

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
