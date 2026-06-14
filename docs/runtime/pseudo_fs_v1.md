# Pseudo-filesystem (/dev) — contract v1

Status: boot-verified via `make test-devfs-v1`
Source: `kernel_rs/src/lib.rs` (`M8FdKind::Dev*`, `sys_open_v1` /dev routing,
`sys_read_v1` / `sys_write_v1` / `sys_poll_v1` Dev arms),
`apps/coreutils/devprobe.asm`.
Proof: `tests/runtime/test_devfs_v1.py`.

Full-OS implementation guide Part II.5 (filesystem maturity), pseudo-fs
slice — the `/dev` character devices. Generated on the fly; no disk I/O.

## Devices

| path | open mode | read | write |
|------|-----------|------|-------|
| `/dev/zero` | RDONLY | endless zero bytes | EPERM (read-only) |
| `/dev/urandom` | RDONLY | CSPRNG bytes (`sys_getrandom` pool) | EPERM |
| `/dev/null` | WRONLY | EOF (0) | accepted and discarded |

`/tmp/<name>` is a small in-memory tmpfs (read+write, create-on-open): a
fixed `TMPFS` store of up to 8 nodes × 512 bytes, indexed from the fd via
`M8_FD_VFS_NODE`. Contents are lost on reboot. Public (no capability). Use
the shell's `fswrite`/`fscat` builtins.

`/proc/self/stat` (read-only) generates the caller's stat line on demand:
`tid=0x<hex16> uid=0x<hex2> state=run\n` (hex in v1; `R4_CURRENT` is the
reader). Read via `fscat /proc/self/stat`.

No new syscall: these route through the existing `open`/`read`/`write`/
`poll` (ids 18/19/20/23) by path, like `/dev/console`. They are public — no
capability required (a sandboxed app can still read entropy). `poll` reports
`/dev/zero`, `/dev/urandom`, `/proc/self/stat` readable and `/dev/null`
writable.

## v1 boundary / carry-forward

- `/dev`: the three classic character devices. `/proc`: only
  `/proc/self/stat` (hex fields). Per-`<tid>` `/proc` entries with decimal
  fields and rss, `/tmp` tmpfs, and a mount-table pseudo-fs factory are
  carry-forward.
- `/dev/urandom` shares the `sys_getrandom` pool; its v1 entropy caveats
  (see [`rng_v1.md`](rng_v1.md)) apply.

## Acceptance

`make test-devfs-v1`: `probe devprobe` reads 16 bytes from `/dev/zero`
(all zero), 16 from `/dev/urandom` (not all zero), and writes 8 bytes to
`/dev/null` (accepted), printing `DEVPROBE: ok`.
