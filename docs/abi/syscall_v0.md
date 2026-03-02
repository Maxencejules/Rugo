# Syscall ABI v0

## Lane

Rugo (Rust no_std kernel). This ABI applies to the Rugo lane only.

## Invocation

Use `int 0x80` (IDT vector 128, gate DPL=3 so user mode can invoke it).

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

### Clobbered registers

The kernel preserves all callee-saved registers (rbx, rbp, r12-r15, rsp).
Caller-saved registers (rcx, r11) may be clobbered. The return value
overwrites rax.

### Return values

Non-negative values indicate success. The value `(u64)-1` (0xFFFFFFFFFFFFFFFF)
indicates an error.

## Syscall Table

| # | Name | Args | Returns | Status (R4) | Description |
|---|------|------|---------|-------------|-------------|
| 0 | sys_debug_write | rdi=buf, rsi=len | bytes written | Implemented (M3) | Write user buffer to serial (max 256 bytes, validates pointer) |
| 1 | sys_thread_spawn | rdi=entry | tid or -1 | Stub (quota variant implemented) | Spawn user thread |
| 2 | sys_thread_exit | -- | -- | Stub | Terminate current thread |
| 3 | sys_yield | -- | 0 | Implemented (minimal cooperative yield) | Yield CPU to scheduler |
| 4 | sys_vm_map | rdi=vaddr, rsi=size | 0 or -1 | Stub | Map memory |
| 5 | sys_vm_unmap | rdi=vaddr, rsi=size | 0 or -1 | Stub | Unmap memory |
| 6 | sys_shm_create | rdi=size | shm_handle or -1 | **Implemented (R4)** | Create shared memory region (max 4096 bytes) |
| 7 | sys_shm_map | rdi=shm_handle, rsi=addr_hint, rdx=flags | mapped_addr or -1 | **Implemented (R4)** | Map shared memory into caller's address space |
| 8 | sys_ipc_send | rdi=endpoint, rsi=buf, rdx=len | 0 or -1 | **Implemented (R4)** | Send message to IPC endpoint (`len` must be 1..256; no truncation) |
| 9 | sys_ipc_recv | rdi=endpoint, rsi=buf, rdx=cap | bytes_received or -1 | **Implemented (R4)** | Receive from IPC endpoint (blocks if empty; second waiter is rejected) |
| 10 | sys_time_now | -- | tick count | Implemented (M3) | Current monotonic tick count |
| 11 | sys_svc_register | rdi=name_ptr, rsi=name_len, rdx=endpoint | 0 or -1 | **Implemented (R4)** | Register service name → endpoint mapping (endpoint must be active) |
| 12 | sys_svc_lookup | rdi=name_ptr, rsi=name_len | endpoint or -1 | **Implemented (R4)** | Lookup service endpoint by name |
| 13 | sys_blk_read | rdi=lba, rsi=buf, rdx=len | bytes_read or -1 | **Implemented (M5)** | Read sectors from VirtIO block device |
| 14 | sys_blk_write | rdi=lba, rsi=buf, rdx=len | bytes_written or -1 | **Implemented (M5)** | Write sectors to VirtIO block device |
| 15 | sys_net_send | rdi=buf, rsi=len | bytes_sent or -1 | **Implemented (M7)** | Send raw Ethernet frame via VirtIO net |
| 16 | sys_net_recv | rdi=buf, rsi=cap | bytes_received or 0 | **Implemented (M7)** | Receive raw Ethernet frame (non-blocking) |
| 17 | sys_ipc_endpoint_create | -- | endpoint or -1 | **Implemented (`quota_endpoints_test`)** | Create IPC endpoint for current task |

Stubs return -1 (0xFFFFFFFFFFFFFFFF) and will be implemented in later milestones.

## IPC model (R4)

### Endpoints

- IPC endpoints are identified by small integer IDs (0-15).
- Each endpoint has a single-slot message buffer (max 256 bytes per message).
- Only one message can be buffered per endpoint at a time.

### Single-slot semantics

Each endpoint holds at most one buffered message. If `sys_ipc_send` is
called on an endpoint whose slot is already occupied (i.e. a previous
message was buffered and has not yet been consumed by `sys_ipc_recv`),
the call returns -1 and the existing message is **not** overwritten.
This makes send behavior deterministic: callers can detect back-pressure
without silent data loss.

Payloads are never silently truncated:
- `sys_ipc_send` returns -1 if `len == 0` or `len > 256`.
- If a buffered message exists and `sys_ipc_recv` is called with `cap < msg_len`,
  the call returns -1 and the message remains buffered.

### Blocking semantics

- `sys_ipc_recv` blocks if no message is available. The kernel saves the
  caller's state and switches to the next ready task.
- `sys_ipc_send` is non-blocking. If a task is blocked on recv for the
  target endpoint, the message is delivered directly to the waiter's
  buffer and the waiter is marked ready.
- If no waiter exists **and** the endpoint's slot is empty, the message
  is stored in the endpoint's buffer. If the slot is occupied, -1 is
  returned.
- At most one blocked waiter is allowed per endpoint. A second
  `sys_ipc_recv` waiter on the same endpoint returns -1 (no waiter overwrite).
- If a waiter exists but its receive capacity is smaller than the send
  payload length, `sys_ipc_send` returns -1 and does not partially deliver.

### Message format

Messages are opaque byte sequences. The kernel does not interpret payload
contents. Sender and receiver agree on format by convention.

## Shared memory model (R4)

### Create

- Quota hardening variants allow a larger handle space than the baseline shm_test image.

- `sys_shm_create(size)` allocates a kernel-backed page (up to 4096 bytes).
- Returns a small integer handle.
- The kernel zero-initializes the backing page.

### Map

- `sys_shm_map(handle, addr_hint, flags)` installs user-accessible PTEs
  pointing to the shared physical page.
- `addr_hint` specifies the desired virtual address (must be page-aligned
  and in user range).
- Returns the mapped virtual address on success.
- Multiple tasks can map the same handle; they share the same physical page.

### Constraints

- `pressure_shm_test` / `quota_shm_test`: maximum 64 SHM regions.

- Maximum 2 SHM regions in baseline `shm_test` image.
- Each region is exactly one 4K page.
- `flags` is reserved (must be 0).

## Service registry (R4)

- Kernel-side name→endpoint lookup table (4 slots max).
- `sys_svc_register` stores a mapping; `sys_svc_lookup` retrieves it.
- Names are compared byte-for-byte (up to 16 bytes).
- `sys_svc_register` returns -1 unless `endpoint` refers to an active IPC endpoint.
- Duplicate registrations deterministically overwrite the endpoint value of
  the existing entry and return 0. No additional slot is consumed.
- If the name is new and all 4 slots are occupied, returns -1.
- Acceptance variants: `svc_overwrite_test`, `svc_full_test`, and `svc_bad_endpoint_test`.

## Quota limits (R4 hardening)

- `MAX_ENDPOINTS_PER_PROC = 16` (per-task): `sys_ipc_endpoint_create` returns -1 when exceeded.
- `R4_MAX_ENDPOINTS = 16` (global endpoint table): `sys_ipc_endpoint_create` returns -1 when full.
- `MAX_SHM_PER_PROC = 32` (per-task): `sys_shm_create` returns -1 when exceeded.
- `R4_MAX_SHM` (global SHM table): `sys_shm_create` returns -1 when full.
- `MAX_THREADS_PER_PROC = 16` (per-task): `sys_thread_spawn` returns -1 when exceeded in `quota_threads_test`.
- `MAX_THREADS_GLOBAL = 64` (global spawned-thread counter): `sys_thread_spawn` returns -1 when exceeded in `quota_threads_test`.
- `R4_MAX_SERVICES = 4` (global service table): `sys_svc_register` returns -1 when full.

Quota accounting is monotonic for this milestone:
- Endpoints: no endpoint-destroy syscall yet.
- SHM objects: `sys_shm_unmap` unmaps VA only; no SHM-destroy syscall yet.
- Threads: no implemented release path for spawned quota-test threads.

## User pointer safety (M3+)

All pointer-taking syscalls use a uniform kernel copy layer (`copyin_user`,
`copyout_user`) that enforces the following before any dereference:

1. Buffer must be in user address range (below 0x0000_8000_0000_0000).
2. `ptr + len` must not overflow.
3. Every page spanned by the buffer must be present and user-accessible
   with required permissions (`user_pages_ok` page-table walk).
4. Data is copied to/from a kernel-side buffer; the kernel never operates
   directly on user mappings.
5. On validation failure, the syscall returns -1 (0xFFFFFFFFFFFFFFFF)
   without crashing or faulting.

Invalid user pointers are always treated as syscall errors (`-1`), never as a kernel crash path.

This applies to `sys_debug_write`, `sys_ipc_send`, `sys_ipc_recv`,
`sys_svc_register`, `sys_svc_lookup`, `sys_blk_read`, `sys_blk_write`,
`sys_net_send`, and `sys_net_recv`.

## User fault containment (M3+)

If a page fault or GPF occurs while CS indicates ring 3 (user mode):
- The kernel terminates the user task.
- In M3 tests: prints `USER: killed` and continues kernel execution.
- In R4 tests: silently kills the task and schedules the next ready task.
  When no tasks remain, the kernel exits.

## Block I/O model (M5)

### Device

- VirtIO block device detected via PCI scan (vendor 0x1AF4, device 0x1001).
- Legacy VirtIO transport (I/O port BAR0, `disable-modern=on` in QEMU).
- Polling driver (no interrupts); deterministic bounded timeout.

### Syscalls

- `sys_blk_read(lba, buf, len)`: Read `len` bytes starting at sector `lba`.
- `sys_blk_write(lba, buf, len)`: Write `len` bytes starting at sector `lba`.

### Constraints

- `len` must be a multiple of 512 bytes.
- Maximum transfer size per call: 4096 bytes (one page).
- `lba` is a sector number (512-byte units).
- User pointer is validated via page table walk before DMA.
- Data is copied through a kernel DMA buffer (copyin/copyout).

## Filesystem model (M6)

### Architecture

Architecture A — services over IPC. The R4 IPC framework is ready. For v0,
the kernel orchestrates filesystem and package operations in kernel space:

- **fsd** (kernel-side): reads SimpleFS v0 from VirtIO block device, validates
  superblock, prints `FSD: mount ok`.
- **pkg** (kernel-side): reads `hello.pkg` from mounted filesystem, parses
  PKG v0 header, extracts hello binary.
- **sh** (kernel-side): loads extracted binary into user-mode page, enters
  ring 3 via `iretq`.
- **hello** (ring 3): prints `APP: hello world` via `sys_debug_write`.

### Disk format

SimpleFS v0 — see `docs/storage/fs_v0.md` for on-disk layout, package format,
and image generation.

### Kernel changes

M6 reuses the M5 VirtIO block driver and the M3 user-mode infrastructure.
No new syscalls are added; the kernel reads the disk directly and only the
hello app runs in user mode. No VFS is added to the kernel.

## Net I/O model (M7)

### Device

- VirtIO net device detected via PCI scan (vendor 0x1AF4, device 0x1000).
- Legacy VirtIO transport (I/O port BAR0, `disable-modern=on` in QEMU).
- Polling driver (no interrupts); deterministic bounded timeout.
- Two virtqueues: RX (queue 0) and TX (queue 1).
- 10-byte `virtio_net_hdr` prepended/stripped by driver (transparent to syscalls).

### Syscalls

- `sys_net_send(buf, len)`: Send `len` bytes as a raw Ethernet frame.
- `sys_net_recv(buf, cap)`: Non-blocking receive into `buf` (max `cap` bytes).

### Constraints

- `len`/`cap` must be 1–1514 bytes (standard Ethernet MTU).
- User pointer is validated (must be below 0x0000_8000_0000_0000).
- `sys_net_recv` returns 0 if no frame is available (non-blocking).
- Frames are raw Ethernet II (14-byte header + payload).
- No socket abstraction; the caller handles protocol parsing.

## Notes

- M3/R4/M5/M6 use `int 0x80` (software interrupt) rather than the `SYSCALL`/`SYSRET`
  MSR mechanism. This simplifies the first user-mode bringup.
- Ring 3 entry is via `iretq` with proper user CS/SS selectors.
- Interrupts may be disabled in user mode for M3/R4/M5/M6. Syscalls and faults still work.
- R4 uses cooperative two-task scheduling: tasks switch only on blocking
  syscalls (ipc_recv) or task death (fault/halt).
