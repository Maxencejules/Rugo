# G2 Gap Report (Rugo Lane)

- [x] **P1 - Unified user-copy is applied to pointer syscalls (resolved 2026-03-02)**: `sys_debug_write`, `sys_blk_read`, `sys_blk_write`, `sys_net_send`, and `sys_net_recv` use the shared `copyin_user`/`copyout_user` path, matching `docs/abi/syscall_v0.md` user-pointer safety requirements.

- [x] **P2 - IPC waiter/truncation semantics hardened (resolved 2026-03-02)**: `sys_ipc_recv_r4` now rejects a second waiter on the same endpoint with `-1` (no overwrite), `sys_ipc_send_r4` rejects invalid lengths (`len == 0` or `len > 256`), and direct-delivery truncation is rejected (`send` returns `-1` when waiter cap is too small). Coverage added via `ipc_waiter_busy_test` (`tests/ipc/test_ipc_waiter_busy.py`).

- [x] **P3 - Service registry endpoint-validity semantics are defined (resolved 2026-03-02)**: `sys_svc_register_r4` now rejects inactive/out-of-range endpoint IDs with `-1` and only stores mappings to active IPC endpoints. Coverage includes overwrite/full behavior (`tests/ipc/test_svc_overwrite.py`, `tests/ipc/test_svc_full.py`) and invalid-endpoint rejection (`tests/ipc/test_svc_bad_endpoint.py`).

- [ ] **P4 - VirtIO blk invariant failure paths are not tested**: init hardening checks exist and `BLK: invariants ok` is tested (`tests/drivers/test_virtio_blk_init_invariants.py`), but failure-path validation for `BLK: init failed` is still missing.

- [x] **P5 - TinyGo dependency contract for full test entrypoint clarified (resolved 2026-03-02)**: `docs/BUILD.md` now states that Go/TinyGo are required for local `make image-go` and full `make test-qemu`, while remaining optional for `make build`/`make image`.
