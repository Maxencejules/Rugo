# Full-OS guide Part IV.10 acceptance: secure boot (measured chain-of-trust gate).
#
# At boot the kernel measures a trusted boot component into a PCR (pcr = SHA-256(
# pcr || SHA-256(component))) and verifies it against a GOLDEN digest baked into
# the kernel, refusing on mismatch. The self-test also confirms that a one-byte
# tamper of the component changes the measurement so the verify rejects it.


def test_secure_boot_golden_verify(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime
    out = boot("shutdown\n").stdout
    find_in_order(out, [
        "RUGO: boot ok",
        # The trusted component measured to the golden value AND a tampered
        # component was rejected.
        "SECURE_BOOT: golden ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "SECURE_BOOT: FAIL" not in out
