# Full-OS guide Part IV.10 acceptance: SHA-256 + measured boot.
#
# At boot the kernel verifies its SHA-256 against three FIPS 180-4 known-answer
# vectors (empty string, "abc", and a 56-byte message that forces a second padded
# block), then performs a TPM-style measured boot: it extends a zeroed PCR with
# two kernel components (the SHA-256 round constants and the AES S-box) and
# records the resulting measurement. SHA-256 is the integrity foundation for
# secure/measured boot; signature verification of the measurement is carry-forward.


def test_sha256_and_measured_boot(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    find_in_order(out, [
        "SHA256: kat ok",
        "MEASURE: pcr=0x",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "SHA256: kat fail" not in out
