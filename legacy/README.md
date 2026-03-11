# Legacy C Kernel (M0-M7 + G0)

This directory contains the original C+Go implementation. It is preserved as a
reference baseline for comparison, regression context, and historical
architecture review. It is not the active product lane.

## Status

All 16 milestone tests pass (M0-M7 + G0):
- Boot, paging, traps, scheduler, user mode, syscalls
- IPC, shared memory, service registry
- VirtIO block driver, SimpleFS filesystem
- Package manager, shell, runtime app loading
- Go kernel entry via gccgo + rtshim bridge
- VirtIO-net driver, user-space netd with ARP + UDP echo

## Building

Prerequisites: nasm, gcc, gccgo, ld, objcopy, xorriso, QEMU, Python 3 + pytest.

From this directory:

```bash
make build     # compile legacy C kernel
make image     # build bootable ISO (uses ../tools/mkimage.sh)
make run       # boot in QEMU
make test-qemu # run legacy acceptance tests (16 tests)
```

Note: the legacy build outputs to `../out/` (shared with the root build). Building the legacy kernel will overwrite the Rust kernel's output and vice versa.
