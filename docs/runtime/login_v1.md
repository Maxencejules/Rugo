# Multi-user authenticated login — contract v1

Status: boot-verified via `make test-login-v1`
Source: `kernel_rs/src/lib.rs` (`sys_proc_ctl` op 5, `login_verify`, `PASSWD`,
`djb2`); `apps/coreutils/loginprobe.asm`.
ABI: `sys_proc_ctl` (id 51) op 5.
Proof: `tests/runtime/test_login_v1.py`.

Full-OS guide Part IV.10 (security), multi-user: an **authenticated** privilege
change. The prior multi-user slice ([`userid_v1.md`](userid_v1.md)) added
getuid/setuid where setuid is root-only (drop, not gain). Login adds the
credential-gated path by which an unprivileged user *gains* a uid — the kernel
half of `login`/`su`.

## Behaviour

`sys_proc_ctl(op=5, a2=name, a3=pw)`:

- copies in the 8-byte username (`a2`) and the ≤16-byte NUL-terminated password
  (`a3`);
- `login_verify` scans the in-kernel password database `PASSWD` for a matching
  username whose stored hash equals `djb2(password)`;
- on a match it sets the caller's uid to that account's uid and returns the uid;
- on no match it records an `login-deny` audit event and returns `-1` (the uid is
  unchanged). A success records `login-ok`.

`PASSWD` holds `root` (uid 0, password `toor`) and `user` (uid 100, password
`pass`); the stored hashes are computed at **compile time** from the cleartext
via the `djb2` const fn, so the table is correct by construction.

## Acceptance

`make test-login-v1`: `probe loginprobe` (which starts as uid 100) shows
`LOGINPROBE: ok` — proving, in one ring-3 app: `getuid()==100`, a wrong-password
`login("root", …)` is denied with the uid unchanged, the correct root password
elevates the caller to uid 0, and `getuid()` then returns 0 — with no
`LOGINPROBE: FAIL`. The deny/ok events also land in the audit ring
([`audit_v1.md`](audit_v1.md)).

## v1 boundary / carry-forward

- **`djb2` is a demonstration hash, NOT a secure password hash.** A real system
  needs a salted, slow KDF (bcrypt/scrypt/argon2); this mirrors the demo disk
  cipher ([`disk_crypt_v1.md`](disk_crypt_v1.md)) in proving the *flow*, not the
  cryptography.
- In-kernel static `PASSWD` (two accounts); an on-disk `/etc/passwd` + `/etc/shadow`
  parsed by a userspace `login`/`getty`, groups/gids, PAM-style stacking, and
  session/audit-uid separation are carry-forward.
- No lockout/backoff on repeated failures (only an audit record).
