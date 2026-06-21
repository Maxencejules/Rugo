# PID namespaces — contract v1

Status: boot-verified via `make test-pidns-v1` (go C4 runtime lane)
Source: `kernel_rs/src/lib.rs` (`sys_nsctl`, `R4Task.pid_ns`, `NS_NEXT_PID`,
`r4_init_task` inheritance), dispatch in `kernel_rs/src/syscall.rs` (id 57);
`apps/coreutils/nsprobe.asm`.
ABI: `sys_nsctl` (id 57).
Proof: `tests/runtime/test_pidns_v1.py`.

Full-OS implementation guide Part I (process model), namespaces: isolate a task's
view of the system so it sees only its own group of processes — the kernel
primitive a container runtime builds on.

## Behaviour

Each task carries a `pid_ns` tag (a `u32`; `0` = the global namespace). It is
inherited across `r4_init_task`, so a clone thread joins its creator's namespace
and a spawned app inherits the shell's global namespace (and may unshare its own).
`pid_ns` is independent of `isolation_domain` (which gates resource quotas), so
unsharing a namespace never perturbs quota grouping.

`sys_nsctl(op)`:

- **op 1 = unshare_pid** — assign the caller a fresh `pid_ns` (a monotonically
  increasing id, so each call is unique) and return it. The caller is now the
  sole, first member of a new namespace — its "init".
- **op 2 = ns_task_count** — the number of **live** tasks visible in the caller's
  namespace: the whole system in the global namespace (`pid_ns == 0`), or only the
  tasks sharing the caller's `pid_ns` otherwise.
- **op 3 = ns_getpid** — the caller's namespace-**local** pid: its real tid in the
  global namespace, or its rank (by tid) among same-namespace tasks otherwise, so
  the first member of a fresh namespace is **pid 1**.
- **op 4 = sethostname(`a2`=ptr, `a3`=len)** — set the caller's **UTS-namespace**
  hostname (the global hostname when in ns 0). v1 stores 8 bytes.
- **op 5 = gethostname** — the caller's namespace hostname (8 bytes,
  little-endian). A fresh namespace inherits the global `"rugo"` until it sets its
  own, so the hostname view is namespace-scoped (a UTS namespace).

## Acceptance

`make test-pidns-v1`: `nsprobe` (a single ring-3 client) reads `ns_task_count`
**before** unsharing and gets the whole system (`> 1` — boot task, services,
shell, itself); calls `unshare_pid`; reads `ns_task_count` **after** and gets
exactly **1** (only itself is in the fresh namespace); and reads `ns_getpid` and
gets **1** (its namespace-local "init" pid, distinct from its global tid). Prints
`NS: pid-namespace isolated ok` — the process-view isolation + namespace-local pid
that define a PID namespace, proven without spawn/clone choreography.

It then proves the **UTS namespace**: `gethostname` returns the global `"rugo"`
(a fresh namespace inherits it), `sethostname("ctr")` sets the namespace's own,
and `gethostname` returns `"ctr"` — the hostname view is namespace-scoped (`NS:
uts-namespace hostname ok`).

## v1 boundary / carry-forward

- **PID view + local pid.** v1 isolates the *count* and the *local pid*. Scoping
  the `/proc` listing and `sys_proc_info` enumeration to the namespace, and
  restricting `kill`/signals to same-namespace targets, are the next isolation
  layers.
- **Single namespace type.** Only the PID namespace exists; mount (a per-task VFS
  root), UTS (hostname), and network namespaces are carry-forward.
- **No nesting / reaping semantics.** A namespace's pid-1 is not yet special for
  orphan reaping or "kill all on init exit"; nested namespaces are flat (a fresh
  id, not a tree).
