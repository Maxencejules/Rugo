# sandbox / syscall allowlist — contract v1

Status: boot-verified via `make test-sandbox-v1`
Source: `kernel_rs/src/lib.rs` (`R4Task.sec_filter_mask`, `sys_sandbox`),
`kernel_rs/src/syscall.rs` (dispatch allowlist gate),
`apps/coreutils/sandboxprobe.asm`.
Proof: `tests/runtime/test_sandbox_v1.py`.

Full-OS implementation guide Part IV.10 (security), sandbox slice
(pledge/unveil-style restriction).

## ABI

`sys_sandbox` — ABI v3.2 id **59**: `rdi` = `allow_mask`. Restricts the
calling task to the syscalls whose bit is set (bit N = syscall N). Returns 0
on success, or -1 if `allow_mask` would re-grant a syscall not currently
allowed (monotonic narrowing only).

## Semantics

- Each `R4Task` carries `sec_filter_mask` (default `u64::MAX` = unrestricted;
  a thread/fork child inherits its parent's; reset on slot reuse).
- The syscall dispatcher denies (returns -1, marker `SANDBOX: deny nr=...`)
  any syscall whose bit is clear, for syscall numbers < 64.
- `sys_sandbox` always keeps syscalls **0** (debug_write) and **2**
  (thread_exit) set, so a sandboxed task can still report and exit.
- Narrowing is monotonic: once a syscall is dropped it cannot be re-granted.

## Markers

| Marker | Emitted when |
|--------|--------------|
| `SANDBOX: deny nr=0x<nr>` | a filtered syscall is rejected (kernel) |
| `SANDBOXPROBE: denied ok` | the probe observed its yield denied |

## v1 boundary / carry-forward

- Mask covers syscalls 0..63 only (the whole current ABI fits).
- No `unveil`-style path restriction yet (filesystem scoping is
  carry-forward).
- No audit-log entry on denial yet (the audit slice is separate); the
  `SANDBOX: deny` serial marker is the v1 record.

## Acceptance

`make test-sandbox-v1`: `probe sandboxprobe` calls `sys_sandbox(1<<0)`, then
attempts `sys_yield` (3); the kernel emits `SANDBOX: deny nr=...3`, the
syscall returns -1, and the probe prints `SANDBOXPROBE: denied ok` and exits
cleanly (exit stayed permitted).
