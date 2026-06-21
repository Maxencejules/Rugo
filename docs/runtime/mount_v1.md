# Mount table (prefix → filesystem registry) — contract v1

Status: boot-verified via `make test-mount-v1`
Source: `kernel_rs/src/mount.rs` (`mount_register`, `mount_lookup`,
`mount_selftest`).
Proof: `tests/runtime/test_mount_v1.py`.

Full-OS guide Part II.5 (filesystem maturity), mounts: a path-prefix →
filesystem-type registry with longest-prefix matching, generalizing the hardcoded
`/data`, `/tmp`, `/dev`, `/proc`, `/mnt` routing in `sys_open`.

## Behaviour

`MOUNTS` (8 entries `{prefix, fs_type}`):

- **`mount_register(prefix, fs_type)`**: install a mount point.
- **`mount_lookup(path)`**: returns the `fs_type` of the **longest** registered
  prefix that matches `path` on a **component boundary** — the prefix must be the
  whole path, be followed by `/`, or itself end in `/` (the root mount). So
  `/data` matches `/data` and `/data/x` but **not** `/database`; a nested
  `/data/special` wins over `/data` for `/data/special/x`; and `/` is the
  fallback.

## Acceptance

`make test-mount-v1`: the boot self-test registers `/`→rootfs, `/data`→SimpleFS,
`/mnt`→FAT, `/data/special`→(nested) and confirms `/data/file`→SimpleFS,
`/mnt/HELLO.TXT`→FAT, `/data/special/x`→nested (longest), `/other`→root,
`/database/x`→root (boundary, **not** `/data`), and exact `/data`→SimpleFS —
`MOUNT: table ok`, with no `MOUNT: table fail`.

## v1 boundary / carry-forward

- The registry + matching logic + self-test. Re-routing the live `sys_open` path
  through `mount_lookup` (so a new mount appears in the namespace without code
  changes) and a `mount`/`umount` syscall to populate it are carry-forward — the
  existing hardcoded routes still serve today.
- 8 mounts, 24-byte prefixes; per-mount filesystem private data (root block,
  device) beyond the `fs_type` tag is carry-forward.
