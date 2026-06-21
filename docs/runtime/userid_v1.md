# Multi-user uid privilege model — contract v1

Status: boot-verified via `make test-userid-v1`
Source: `kernel_rs/src/lib.rs` (`sys_proc_ctl` ops 3/4),
`apps/coreutils/userprobe.asm`.
Proof: `tests/runtime/test_userid_v1.py`.

Full-OS implementation guide Part IV.10 (security), multi-user slice — the uid
query + privilege-change primitives a login/session layer builds on. (Per-uid
file ownership already exists in the SimpleFS `/data` tree; this adds the
process-side uid controls.)

## Behaviour

`sys_proc_ctl` (id 51):
- **op 3 = getuid** → the caller task's uid. External apps run as uid 100; the
  kernel/init runs as uid 0.
- **op 4 = setuid(`rsi`)** → only a task already at uid 0 (root) may change its
  uid (the privilege-drop case); a non-root caller is **denied** (returns -1)
  and the attempt is recorded in the security audit log
  ([`audit_v1.md`](audit_v1.md)) as `setuid-deny`. uid is a `u8`.

This enforces the core privilege rule: privilege can be dropped by root but not
gained by an unprivileged task.

## v1 boundary / carry-forward

- No `/etc/passwd`, no authentication, no login flow, no groups/gids, no
  saved-set-uid semantics. A `login` userspace program that authenticates and
  then `setuid`s from a root session, plus per-uid resource accounting, are
  carry-forward.

## Acceptance

`make test-userid-v1`: `userprobe` (an external app, uid 100) confirms
`getuid` = 100, `setuid(0)` is denied (returns -1), and the uid is unchanged —
printing `USERPROBE: uid=100 setuid-denied ok`.
