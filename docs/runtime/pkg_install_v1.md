# Package signature-verify + install — contract v1

Status: boot-verified via `make test-pkg-install-v1`
Source: `kernel_rs/src/net.rs` (`pkg_install_selftest`), `kernel_rs/src/sha256.rs`
(`hmac_sha256`); boot call after `secure_boot_selftest`.
Proof: `tests/runtime/test_pkg_install_v1.py`.

Full-OS guide Part V.11 (package manager). Beyond fetching a package over TCP
([`pkgfetch_v1.md`](pkgfetch_v1.md)), the manager **authenticates** a package and
**installs** it: it verifies an HMAC-SHA256 signature over the payload (rejecting
any tamper) and writes the verified payload to persistent storage.

## Behaviour

`pkg_install_selftest` builds a mock signed package (a payload + an
**HMAC-SHA256(key, payload)** signature field), then:

- **verifies** it — recomputes the MAC over the received payload and compares to
  the signature field (accepts a valid package);
- **rejects a tamper** — a one-byte change to the payload no longer matches the
  signature field (the installer would refuse it);
- **installs** the verified payload — writes it to a scratch LBA via the block
  layer and reads it back, confirming an exact round-trip.

`hmac_sha256` (RFC 2104) is implemented over the existing SHA-256. All three must
hold → `PKG: sigverify+install ok`; otherwise `PKG: sigverify+install FAIL`.

## Acceptance

`make test-pkg-install-v1`: the go lane boots and prints `PKG: sigverify+install
ok`, reaching a clean shutdown with no `PKG: sigverify+install FAIL`.

## v1 boundary / carry-forward

- A **symmetric** signature (HMAC with a baked key) + install to a scratch sector
  with read-back. Asymmetric signatures (Ed25519/RSA), wiring the install to a
  real filesystem path / the app region (vs a scratch LBA), installing the
  *fetched* package (vs a mock), and a repo index + dependency resolution are
  carry-forward.
