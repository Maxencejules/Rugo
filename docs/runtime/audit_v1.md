# Security audit log — contract v1

Status: boot-verified via `make test-audit-v1`
Source: `kernel_rs/src/lib.rs` (`AUDIT` ring, `audit_event`/`audit_read`,
`sys_net_query` cap-deny hook, `sys_sysinfo` op 7),
`apps/coreutils/auditprobe.asm`.
Proof: `tests/runtime/test_audit_v1.py`.

Full-OS implementation guide Part IV.10 (security), audit slice — a structured
security-event trail, distinct from raw dmesg, that userspace can query.

## Behaviour

The kernel keeps a fixed `AUDIT` ring (1 KiB) separate from the dmesg ring.
`audit_event(tag, nr)` appends a structured line:

```
AUDIT: <tag> nr=0x<syscall> tid=0x<caller>
```

recording WHO (the caller's task id) attempted WHAT (the syscall number) and the
outcome tag. v1 hooks the **capability-denial** checkpoint: a task without the
`NETWORK` capability calling `sys_net_query` (id 49) is recorded as
`cap-deny`. (The same `audit_event` helper is intended for the other
capability/sandbox checkpoints and privileged operations — extending the hook
set is carry-forward.)

`sys_sysinfo` (id 61) **op 7** = audit read: `rsi` = buffer, `rdx` = capacity;
copies the most recent bytes of the ring (oldest→newest) and returns the count.
Reading is public (any task may inspect the trail).

## v1 boundary / carry-forward

- One hooked checkpoint (network capability denial). Sandbox-mask denials,
  storage-capability denials, privileged power/spawn events, and per-uid
  attribution are carry-forward.
- Text-line ring (not fixed binary records); no severity, no persistence to
  disk, no rotation policy beyond the ring overwrite.
- Distinct from dmesg ([`dmesg_v1.md`](dmesg_v1.md)): dmesg is all kernel
  output; the audit log is security events only and is not echoed to serial.

## Acceptance

`make test-audit-v1`: `auditprobe` (an external app with only the STORAGE
capability) calls `sys_net_query`, which is denied for lack of NETWORK; it then
reads the audit ring (op 7) and echoes it. The transcript shows
`AUDIT: cap-deny nr=0x0000000000000031 tid=0x..` (49 = 0x31) and
`AUDITPROBE: ok`.
