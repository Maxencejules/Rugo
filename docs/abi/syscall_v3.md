# Syscall ABI v3

## Lane

Rugo (Rust `no_std` kernel). This ABI applies to the Rugo lane only.

## Status

Active contract closed in M21 on 2026-03-09.

## Contract identity

Syscall ABI identifier: `rugo.syscall_abi.v3`.

## Relationship to v2

- Invocation mechanism and register convention are unchanged from v2.
- Syscall IDs `0..27` are carried forward without renumbering.
- v3 establishes long-window compatibility obligations via:
  - `docs/runtime/abi_stability_policy_v2.md`
  - `docs/runtime/deprecation_window_policy_v1.md`
- IDs `28..47` are reserved for additive v3.x expansion and are not required in
  v3.0.
- IDs `48..63` are reserved for additive v3.2 expansion under the same
  rules (the `28..47` window is fully allocated). Allocated so far:
  `48` = `sys_signal_ctl` (op 1 = register handler, op 2 = kill(tid, sig),
  op 3 = sigreturn; contract in `docs/runtime/signals_v1.md`);
  `49` = `sys_net_query` (op 1 = DHCP discover, op 2 = DNS A query,
  op 3 = poll, op 4 = ICMP echo self-test, op 5 = ARP responder self-test,
  op 6 = TCP passive-open (listener) self-test, op 7 = ICMPv6 echo self-test,
  op 8 = UDP echo (port 7) self-test (→ 1 ok / 0 fail); contract in
  `docs/runtime/netcfg_v1.md`, `docs/runtime/icmp_v1.md`,
  `docs/runtime/arp_v1.md`, `docs/runtime/tcp_listen_v1.md`,
  `docs/runtime/icmpv6_v1.md`, `docs/runtime/udp_echo_v1.md`);
  `50` = `sys_vm_ctl` (op 1 = mmap(va, sz, prot) → va, op 2 = munmap(va,
  sz) → 0, op 3 = brk(new) → old break, op 4 = mprotect(va, sz, prot) → 0;
  contract in `docs/runtime/mmap_v1.md`);
  `51` = `sys_proc_ctl` (op 1 = fork — copy-on-write duplicate of the
  caller, returns child tid to the parent and 0 to the child; op 2 = clone
  — new thread sharing the caller's address space, `rsi` = entry
  (`rdi` is the op selector = 2); contract in `docs/runtime/fork_v1.md`);
  `52` = `sys_futex` (op 1 = wait(`rsi`=uaddr, `rdx`=val), op 2 = wake(uaddr,
  n); contract in `docs/runtime/futex_v1.md`);
  `53` = `sys_time` (op 1 = clock_gettime(`rsi`=clockid: 0 = MONOTONIC
  nanoseconds since boot, 1 = REALTIME seconds since the Unix epoch); op 2 =
  nanosleep(`rsi`=nanoseconds), op 3 = timerfd_create(`rsi`=nanoseconds) →
  fd; contract in `docs/runtime/clock_v1.md`);
  `54` = `sys_getrandom` (`rdi`=buf, `rsi`=len → bytes written; contract in
  `docs/runtime/rng_v1.md`);
  `59` = `sys_sandbox` (`rdi`=allow_mask → restrict the caller to the
  syscalls whose bit is set; monotonic; syscalls 0 and 2 always kept;
  contract in `docs/runtime/sandbox_v1.md`);
  `56` = `sys_ioctl` (generic device control; op 1 = framebuffer blit,
  `rsi` packs the rect x<<48|y<<32|w<<16|h, `rdx` = XRGB color; op 2 = openpty
  → (slave_fd << 32) | master_fd; op 3 = PC speaker beep(`rsi`=Hz) → gate bits;
  contract in `docs/runtime/graphics_v1.md`, `docs/runtime/pty_v1.md`,
  `docs/runtime/audio_v1.md`);
  `58` = `sys_power` (op 0 = shutdown via ACPI S5 / debug-exit, op 1 =
  reboot via 8042; uid 0 only; contract in `docs/runtime/power_v1.md`);
  `61` = `sys_sysinfo` (op 1 = live task count, op 2 = free physical
  frames, op 3 = uptime ticks, op 4 = dmesg read(`rsi`=buf, `rdx`=cap) →
  bytes copied, op 5 = MBR partition parse → partition count (logs each
  entry), op 6 = FAT16 read HELLO.TXT(`rsi`=buf, `rdx`=cap) → file bytes,
  op 7 = security audit-log read(`rsi`=buf, `rdx`=cap) → bytes copied,
  op 8 = FAT16 root directory list → entry count (logs each);
  contract in `docs/runtime/sysinfo_v1.md`, `docs/runtime/dmesg_v1.md`,
  `docs/runtime/partitions_v1.md`, `docs/runtime/fat16_v1.md`,
  `docs/runtime/audit_v1.md`).

Spawned tasks get a copy-on-write-isolated private address space, a random
page-aligned stack offset (stack ASLR, drawn from `sys_getrandom`'s pool),
and the boot-time PCI device inventory is logged (`PCI:`/`PROBE:` markers).

## Invocation

Use `int 0x80` (vector 128, DPL=3).

### Register convention

| Register | Purpose |
|----------|---------|
| `rax` | Syscall number (in) / return value (out) |
| `rdi` | Argument 1 |
| `rsi` | Argument 2 |
| `rdx` | Argument 3 |
| `r10` | Argument 4 |
| `r8` | Argument 5 |
| `r9` | Argument 6 |

## v3 freeze and compatibility rules

Freeze window: `v3.x`.

- No syscall ID renumbering is allowed in `v3.x`.
- Existing argument semantics and side effects cannot change in `v3.x`.
- Existing deterministic failure classes cannot be weakened in `v3.x`.
- New behavior in `v3.x` must be additive and backward-compatible.

Breaking changes require all of:

1. an ABI-line bump (for example `v3` to `v4`);
2. an explicit migration document under `docs/abi/`;
3. green policy checks from:
   - `tools/check_abi_diff_v3.py`
   - `tools/check_syscall_compat_v3.py`.

## Deterministic error classes

`-1` remains the error return encoding. Error classes are contract-level labels
for deterministic behavior:

| Class | Meaning |
|-------|---------|
| `E_INVAL` | Invalid argument combination |
| `E_RANGE` | Value outside supported limits |
| `E_FAULT` | Invalid user pointer/range |
| `E_BUSY` | Resource is temporarily busy |
| `E_AGAIN` | Retry is allowed |
| `E_PERM` | Rights/capability denied |
| `E_UNSUP` | Explicitly unsupported operation |
| `E_NOSYS` | Unknown syscall ID |
| `E_IO` | Device/transport I/O failure |
| `E_TIMEOUT` | Bounded operation timed out |

## Frozen syscall surface in v3.0 (M21)

| ID | Name | Class | v3.0 state |
|----|------|-------|------------|
| 0 | `sys_debug_write` | required | active |
| 1 | `sys_thread_spawn` | required | active |
| 2 | `sys_thread_exit` | required | active |
| 3 | `sys_yield` | required | active |
| 4 | `sys_vm_map` | required | active |
| 5 | `sys_vm_unmap` | required | active |
| 6 | `sys_shm_create` | required | active |
| 7 | `sys_shm_map` | required | active |
| 8 | `sys_ipc_send` | required | active |
| 9 | `sys_ipc_recv` | required | active |
| 10 | `sys_time_now` | required | active |
| 11 | `sys_svc_register` | required | active |
| 12 | `sys_svc_lookup` | required | active |
| 13 | `sys_blk_read` | required | active |
| 14 | `sys_blk_write` | required | active |
| 15 | `sys_net_send` | required | active |
| 16 | `sys_net_recv` | required | active |
| 17 | `sys_ipc_endpoint_create` | required | active |
| 18 | `sys_open` | required | active |
| 19 | `sys_read` | required | active |
| 20 | `sys_write` | required | active |
| 21 | `sys_close` | required | active |
| 22 | `sys_wait` | required | active |
| 23 | `sys_poll` | required | active |
| 24 | `sys_fd_rights_get` | required | active |
| 25 | `sys_fd_rights_reduce` | required | active |
| 26 | `sys_fd_rights_transfer` | required | active |
| 27 | `sys_sec_profile_set` | required | active |

## Deprecation registry (v3 line)

No syscalls are deprecated in v3.0.

When deprecations are introduced in v3.x they must include:

- first deprecation release;
- earliest removal release;
- replacement syscall/API path;
- linked migration notes.

## Conformance references

- `docs/runtime/abi_stability_policy_v2.md`
- `docs/runtime/deprecation_window_policy_v1.md`
- `tests/runtime/test_abi_docs_v3.py`
- `tests/runtime/test_abi_window_v3.py`
- `tests/runtime/test_abi_diff_gate_v3.py`
- `tests/compat/test_abi_compat_matrix_v3.py`

