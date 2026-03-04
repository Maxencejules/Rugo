# Package/Repository Format v1

## Lane

Rugo (Rust kernel + Go user space), M8 PR-3 bootstrap contract.

## Status

Active as of 2026-03-04 for external app packaging and install bootstrap.

## Scope

This document defines:

- package blob format (`rugo.pkg.v1`),
- signed repository metadata format (`rugo.repo.v1`),
- install semantics from package/repository data to runtime execution path.

This contract is intentionally bootstrap-focused and deterministic for CI.

## Package format (`rugo.pkg.v1`)

All multi-byte fields are little-endian.

### Blob layout

- Header (16 bytes):
  - `magic` (`8 bytes`) = `RPKGV1\0\0`
  - `manifest_len` (`u32`)
  - `payload_len` (`u32`)
- Manifest (`manifest_len` bytes): UTF-8 JSON, canonicalized with sorted keys
  and compact separators.
- Payload (`payload_len` bytes): raw executable payload.

### Manifest fields

- `schema`: must be `rugo.pkg.v1`.
- `name`: package/application name.
- `version`: package version string.
- `abi_profile`: compatibility profile target (v1 baseline: `compat_profile_v1`).
- `payload_size`: byte size of payload.
- `payload_sha256`: hex SHA-256 digest of payload.

Installers must reject blobs with size/hash/schema mismatches.

## Repository metadata (`rugo.repo.v1`)

Repository metadata is a signed document with:

- `metadata` object:
  - `schema`: `rugo.repo.v1`
  - `generated_at`: RFC3339 UTC timestamp
  - `packages`: list of package records:
    - `name`, `version`, `abi_profile`
    - `pkg_file` (artifact filename)
    - `pkg_size`, `pkg_sha256`
- `signature` object:
  - `alg`: `HMAC-SHA256`
  - `key_id`: signer key identifier
  - `sig_hex`: HMAC over canonical JSON(metadata)

## Signature and trust model

For M8 bootstrap, metadata signatures use deterministic HMAC-SHA256 so CI can
generate and verify repository metadata without external key infrastructure.

This is sufficient for bootstrap integrity checks. A future hardening revision
may upgrade signature material and key distribution without changing package
payload layout.

## Install semantics

Install path for M8 PR-3:

1. Verify repository metadata signature.
2. Verify package artifact hash/size against repository metadata.
3. Parse package v1 manifest and verify payload hash/size.
4. Bridge payload into runtime package form (`hello.pkg`) inside SimpleFS.
5. Boot kernel `pkg_hash_test`/`fs_test` path; runtime verifies payload hash
   and executes user payload.

The runtime bridge exists because kernel-side package execution currently
consumes `hello.pkg` in the established M6 format path.

## Tooling and tests

- Tooling:
  - `tools/pkg_bootstrap_v1.py`
- Tests:
  - `tests/pkg/test_pkg_external_apps.py`
  - existing runtime checks: `tests/pkg/test_pkg_install_run.py`,
    `tests/pkg/test_pkg_hash.py`
