# Full-OS guide Part IV.10 acceptance: AES-128 block cipher.
#
# At boot the kernel runs an AES self-test: it encrypts the FIPS-197 Appendix C.1
# known-answer vector (key 000102..0f, plaintext 00112233..ff -> ciphertext
# 69c4e0d8..0a) and checks the result byte-for-byte, then runs an AES-128-CTR
# round-trip over a non-block-aligned buffer (encrypt differs from plaintext;
# decrypt restores it). AES-CTR also backs at-rest disk encryption (disk_crypt),
# replacing the demo xorshift keystream.


def test_aes_known_answer(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    find_in_order(out, [
        "AES: kat ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "AES: kat fail" not in out
