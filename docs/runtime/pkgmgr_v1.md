# Package manager — signed repository, contract v1

Status: boot-verified via `make test-pkgmgr-v1` (go lane, repo seeded on the boot
disk's scratch gap).
Source: `kernel_rs/src/net.rs` (`pkg_manager_selftest`), `tools/pkg_repo_v1.py`
(host repo builder), boot call after `pkg_install_selftest`.
Proof: `tests/runtime/test_pkgmgr_v1.py`.

Full-OS implementation guide Part V.11 (operations): the package-manager core on
top of the single-blob fetch (`pkgfetch_v1.md`) and the one-payload sig-verify +
install (`pkg_install_v1.md`). This adds a **repo index of multiple packages**,
**selection by name**, **per-package integrity**, and a **signed manifest**.

## On-disk repo layout

Built host-side by `tools/pkg_repo_v1.py` into the free 12..63 scratch gap (clear
of pkgfetch@16, pkg-install@21):

- **LBA 24 — index sector**: `RPKG` magic, a package count, a 32-byte index
  signature, then 64-byte entries (`name[24]`, payload start-LBA, payload length,
  `SHA-256(payload)`).
- **LBA 25.. — payloads**: each package's bytes, sector-padded.

The index is signed as `HMAC-SHA256(KEY, SHA-256(entries))` — signing the 32-byte
manifest digest sidesteps the HMAC input cap and covers every entry regardless of
count. `KEY` is shared with the kernel (`rugo-repo-index-key-v1`).

## Behaviour (`pkg_manager_selftest`, every go boot)

1. Read LBA 24. If it is not an `RPKG` index (the common case — a blank scratch
   sector), report `PKGMGR: no repo` and do nothing (no writes — safe on every
   boot).
2. **Verify the signed index**: recompute `HMAC(KEY, SHA-256(entries))` and
   compare to the stored signature; also confirm a **forged** index (one flipped
   entry byte) does *not* verify. → `PKGMGR: index count=0x… sig ok`.
3. **Select** the package named `calc` by walking the entries.
4. Read its payload off the repo and **verify its `SHA-256`** against the entry's
   hash; confirm a **tampered** payload (one flipped byte) is rejected. →
   `PKGMGR: select calc lba=0x… len=0x… hash ok`, `PKGMGR: tamper rejected`.
5. **Install** the verified payload: write it to a scratch LBA and read it back
   byte-exact. → `PKGMGR: install ok`, then `PKGMGR: ok`.

Any failure reports a `PKGMGR: … FAIL` marker and aborts.

## Acceptance

`make test-pkgmgr-v1`: the test seeds a 3-package signed repo (`calc`/`edit`/
`term`) onto the boot disk, boots the go lane, and the transcript shows
`PKGMGR: index count=0x…03 sig ok`, `PKGMGR: select calc … hash ok`,
`PKGMGR: tamper rejected`, `PKGMGR: install ok`, `PKGMGR: ok`, reaching
`RUGO: halt ok` — with no `PKGMGR: … FAIL` and no `PKGMGR: no repo`.

## v1 boundary / carry-forward

- A real package-manager core: a **signed multi-package index**, name selection,
  per-package integrity, and verified install — the index/selection/signature
  layer the guide called for on top of the existing fetch + install. What remains:
  installing the selected package into the **live app region** so it can be
  spawned directly (vs a scratch-LBA install + read-back), an `apt`-style request
  CLI, dependency resolution, fetching the repo + index **over the network**
  (combining with `pkgfetch_v1.md`), version/upgrade handling, and asymmetric
  (public-key) signatures instead of a shared HMAC key.
