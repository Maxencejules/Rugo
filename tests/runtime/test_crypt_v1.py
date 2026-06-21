# Full-OS guide Part IV.10 acceptance: at-rest disk encryption.
#
# sys_sysinfo op 9 encrypts a known plaintext, writes the ciphertext to a
# scratch sector, reads it back raw (must differ from the plaintext), decrypts,
# and verifies the round trip. cryptprobe drives it; the test asserts the
# kernel's success marker.


def test_disk_encryption_roundtrip(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("probe cryptprobe\nshutdown\n").stdout

    find_in_order(out, [
        "CRYPT: disk roundtrip ok",
        "CRYPTPROBE: ok",
        "RUGO: halt ok",
    ])
    assert "CRYPTPROBE: FAIL" not in out
