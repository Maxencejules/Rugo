# dmesg — kernel log ring buffer — contract v1

Status: boot-verified via `make test-dmesg-v1`
Source: `kernel_rs/src/lib.rs` (`serial_write` tap, `klog_append`, `klog_read`,
`sys_sysinfo` op 4), `kernel_rs/src/syscall.rs` (id 61 dispatch),
`apps/coreutils/dmesgprobe.asm`.
Proof: `tests/runtime/test_dmesg_v1.py`.

Full-OS implementation guide Part V.11 (observability: dmesg/syslog) and
IV.10 (audit trail) — userspace can read the kernel log.

## Behaviour

`serial_write` (the single kernel/serial/framebuffer output chokepoint) mirrors
every line into a fixed `KLOG` ring (`KLOG_CAP` = 8192 bytes). When full, the
oldest bytes are overwritten — no heap, no blocking, no allocation.

`sys_sysinfo` (id 61) **op 4** = dmesg read: `rsi` = user buffer, `rdx` =
capacity. It copies the most recent `min(rdx, valid)` bytes of the ring, in
oldest→newest order, into the buffer and returns the byte count (or `u64::MAX`
if the buffer is not user-writable). The ring is only captured on the
`go_test` lane (gated `all(go_test, not(compat_real_test))`).

## v1 boundary / carry-forward

- Read-only tail snapshot. No severity levels, no per-facility filtering, no
  follow/blocking mode, no timestamps, no clear/rotate op.
- The ring captures `serial_write` output; that already includes userspace
  console writes (they route through `sys_debug_write` → `serial_write`).
- The console-write syscall (`sys_debug_write`) caps a single write at 256
  bytes, so a reader that echoes the log must chunk it; `dmesgprobe` reads a
  200-byte tail to stay within one write.
- A structured audit log (distinct from raw dmesg) and a `dmesg`/`syslog`
  userspace utility are carry-forward.

## Acceptance

`make test-dmesg-v1`: `dmesgprobe` writes a unique cookie
(`DMESGCOOKIE-7142`) — captured by the ring as it prints — then reads the
dmesg tail back and echoes it. The cookie therefore appears **twice** in the
transcript (once written, once read back from the ring), proving capture and
read-back, followed by `DMESGPROBE: ok`.
