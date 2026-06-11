# Signals Contract v1

Status: live runtime (boot-verified)
Source: `kernel_rs/src/lib.rs` (`sys_signal_ctl`, `sig_deliver_if_pending`),
probe `apps/coreutils/sigprobe.asm`
Proof: `make test-signals-v1`, `tests/runtime/test_signals_runtime_v1.py`

Second slice of gap-analysis item 10: asynchronous notification of user
tasks with handler delivery and a default kill action.

## Syscall surface

`sys_signal_ctl` (id 48, first allocation in the additive v3.2 window
`48..63` — see `docs/abi/syscall_v3.md`):

- op 1: register handler — `a2` = user-space handler address (0 clears)
- op 2: kill — `a2` = target tid (`u64::MAX` = self), `a3` = signal
  0..63; permitted for self or a direct child only
- op 3: sigreturn — restores the interrupted frame; only valid while a
  handler is running

## Delivery

- One handler per task, a 64-bit pending bitmap, the lowest pending
  signal delivers first.
- Delivery points: every task dispatch (`r4_switch_to`) and the return
  of a self-directed op 2 — the kernel rewrites the trap frame: the
  interrupted state is saved per-task, RIP becomes the handler,
  RDI carries the signal number, RSP drops 256 bytes (red-zone clear)
  and is 16-byte aligned.
- `sigreturn` restores the saved frame exactly; the interrupted code
  resumes as if nothing happened (marker-proven by `sigprobe`).
- Signal 9 always kills. Any signal without a registered handler kills
  (`SIG: kill tid=0x<tid> sig=0x<sig>`, exit status 1, normal reaping).
- No nesting: pending signals wait until the running handler returns.
- Spawn-slot reuse clears all signal state.

## Marker contract

| Marker | Meaning |
|---|---|
| `SIG: kill tid=0x<tid> sig=0x<sig>` | default action terminated a task |
| `SIGPROBE: handler sig=15` | probe handler ran with the signal number |
| `SIGPROBE: resumed after handler` | sigreturn restored the interrupted path |

## v1 carry-forward

Signal masks/blocking, queued (rt) semantics, fault-to-signal mapping
(SIGSEGV/SIGFPE from the user-fault path), kill from the shell by tid
(needs a non-blocking spawn surface), libc wrappers (`signal()`,
`kill()`, `raise()`).
