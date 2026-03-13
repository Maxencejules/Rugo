# Experimental And Supported Lanes

This directory names work that originated as experimental and tracks its
promotion status.

## Supported lanes

- **stock-Go userspace (G2)**: Stock Go is a first-class bootable userspace
  lane.  It builds via `make build-go-std`, boots via `make boot-std`, and is
  smoke-tested via `make smoke-std`.  The ABI surface it relies on (syscall IDs
  0–27) is frozen in `docs/abi/syscall_v3.md` and validated against the kernel
  source of truth by `tools/extract_kernel_syscalls.py`.

  Key paths:
  - runtime/services: `services/go_std/`
  - builder tooling: `tools/gostd_stock_builder/`
  - bootstrap verification: `tools/bootstrap_go_port_v1.sh`
  - toolchain contract: `tools/runtime_toolchain_contract_v1.py`

## Experimental work

- long-horizon research roadmap: `docs/POST_G2_EXTENDED_MILESTONES.md`

Important note:
- Experimental work is preserved and tested but clearly labeled.
- The stock-Go lane is no longer experimental: it is a supported build and boot
  path with ABI, toolchain, and smoke-test gates.
