# lseek — contract v1

Status: boot-verified via `make test-lseek-v1`
Source: `kernel_rs/src/lib.rs` (`sys_fs_ctl_v1` op 6),
`apps/coreutils/lseekprobe.asm`.
Proof: `tests/runtime/test_lseek_v1.py`.

Full-OS implementation guide Part V.11 (rlibc completion), the `lseek`
primitive.

## ABI

Folded into `sys_fs_ctl` (id 47) as **op 6** (an additive op, not a new
syscall id): `rsi` = fd, `rdx` = absolute offset. Sets the file descriptor's
read/write cursor (SEEK_SET) and returns the new offset, or -1.

Owner-gated (`r4_fd_owner_ok`); no storage capability required (it is a
generic descriptor operation). Works on any seekable fd (the offset field is
the same cursor `read`/`write` advance).

## v1 boundary / carry-forward

- SEEK_SET only — `SEEK_CUR`/`SEEK_END` and negative offsets are
  carry-forward.
- No bounds clamp to file size (a read past EOF already returns 0).

## Acceptance

`make test-lseek-v1`: `lseekprobe` writes `ABCDE` to `/data/lstst`, reopens
it, `lseek`s to offset 2, reads 3 bytes, and confirms `CDE` — proving the
cursor moved.
