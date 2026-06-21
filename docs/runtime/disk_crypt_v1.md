# At-rest disk encryption — contract v1

Status: boot-verified via `make test-crypt-v1`
Source: `kernel_rs/src/lib.rs` (`disk_crypt`, `sys_sysinfo` op 9),
`apps/coreutils/cryptprobe.asm`.
Proof: `tests/runtime/test_crypt_v1.py`.

Full-OS implementation guide Part IV.10 (security), disk-encryption slice — the
transparent encrypt-on-write / decrypt-on-read path for data at rest.

## Behaviour

`disk_crypt(lba, buf)` XORs `buf` with a per-LBA xorshift64 keystream seeded
from a fixed key (`DISK_KEY ^ lba*prime`). It is symmetric — encrypt and decrypt
are the same call — and key+LBA reproducible, so a sector written encrypted
reads back to the original plaintext after a second `disk_crypt`.

`sys_sysinfo` (id 61) **op 9** = round-trip self-test: encrypt a known 32-byte
plaintext, write the ciphertext to a scratch sector (LBA 1600, above the
SimpleFS region), read it back raw, and verify (a) the on-disk bytes are
ciphertext (≠ plaintext) and (b) decrypting them reproduces the plaintext —
emitting `CRYPT: disk roundtrip ok` and returning 1.

## v1 boundary / carry-forward

- **Not cryptographically secure.** An xorshift keystream is a demonstration
  cipher with no confidentiality guarantee and **no integrity tag**. A real
  authenticated cipher (AES-XTS for block storage, or AES-GCM /
  ChaCha20-Poly1305) is carry-forward.
- The key is a fixed embedded constant; a passphrase/TPM-derived key via a KDF
  is carry-forward.
- Encryption is exercised through an explicit self-test, not yet wired as a
  transparent layer under the SimpleFS/FAT block path — a full encrypted block
  device (crypt-on-every-`block_io_dispatch`) is carry-forward.

## Acceptance

`make test-crypt-v1`: `cryptprobe` calls op 9; the transcript shows
`CRYPT: disk roundtrip ok` and `CRYPTPROBE: ok`, proving plaintext →
ciphertext-on-disk → plaintext.
