# AES-128 block cipher — contract v1

Status: boot-verified via `make test-aes-v1`
Source: `kernel_rs/src/aes.rs` (`encrypt_block`, `key_expand`, `ctr_xor`,
`aes_selftest`); used by `disk_crypt` in `kernel_rs/src/lib.rs`.
Proof: `tests/runtime/test_aes_v1.py` (and `test_crypt_v1.py`, now AES-backed).

Full-OS guide Part IV.10 (security), crypto: a real, standard cipher replacing
the demo xorshift keystream behind at-rest disk encryption
([`disk_crypt_v1.md`](disk_crypt_v1.md)).

## Behaviour

A from-scratch `no_std` AES-128 (FIPS-197): table S-box, 11-round key schedule,
and the encrypt round (`AddRoundKey` → 9× `SubBytes`/`ShiftRows`/`MixColumns`/
`AddRoundKey` → final `SubBytes`/`ShiftRows`/`AddRoundKey`) over a column-major
state. GF(2⁸) arithmetic via `xtime`/`mul3`.

- `encrypt_block(key, in) -> out`: one 16-byte block.
- `ctr_xor(key, lba, buf)`: AES-128 **CTR** — XOR `buf` with the keystream from
  encrypting per-block counters (`lba` in the low 8 bytes, the 16-byte block
  index in the high 8); the same call encrypts and decrypts.

`disk_crypt` now calls `ctr_xor` with a fixed 16-byte key, so encrypt-on-write /
decrypt-on-read is genuine AES.

## Acceptance

`make test-aes-v1`: the boot transcript shows `AES: kat ok` — the FIPS-197
Appendix C.1 vector encrypts to the exact published ciphertext, and a CTR
round-trip over a 35-byte (non-block-aligned) buffer differs from the plaintext
after encryption and is restored after decryption — with no `AES: kat fail`. The
existing `test_crypt_v1.py` disk round-trip stays green on the AES-CTR backing.

## v1 boundary / carry-forward

- **Encrypt-only core + CTR**; no standalone decrypt (CTR does not need it). ECB
  decrypt (inverse rounds) is carry-forward if a non-CTR mode is ever needed.
- **No authentication.** CTR without a MAC provides confidentiality, not
  integrity; an AEAD (AES-GCM / ChaCha20-Poly1305) is carry-forward.
- **Fixed key, no KDF.** A passphrase-derived key (PBKDF2/argon2) + per-volume
  salt is carry-forward.
- **Table-based S-box** — not constant-time; cache-timing hardening is
  carry-forward (acceptable for a single-tenant demo).
