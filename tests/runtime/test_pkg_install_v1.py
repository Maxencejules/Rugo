# Full-OS guide Part V.11 acceptance: package signature-verify + install.
#
# Beyond fetching a package, the manager verifies an HMAC-SHA256 signature over
# the payload (rejecting any tamper) and installs the verified payload to
# persistent storage, reading it back to confirm. The boot self-test exercises a
# valid package (accepted), a tampered payload (rejected against the same
# signature), and the disk install round-trip.


def test_package_signature_verify_and_install(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime
    out = boot("shutdown\n").stdout
    find_in_order(out, [
        "RUGO: boot ok",
        "PKG: sigverify+install ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "PKG: sigverify+install FAIL" not in out
