# Architecture Overview

Rugo is a hybrid OS with a strict language split:

- Rust `no_std` kernel for mechanisms, traps, memory management, scheduling,
  low-level drivers, and kernel-side ABI enforcement
- Go user space for services, policy, and the eventual higher-level OS surface
- TinyGo-first as the default early integration lane for user space
- stock-Go support kept as an experimental porting lane until its runtime and
  repository shape are mature enough to become first-class

## Current Source Map

| Bucket | Current paths | Notes |
|--------|---------------|-------|
| Core runtime | `arch/`, `boot/`, `kernel_rs/` | This is the actual kernel lane, even though it is split across multiple top-level directories. |
| Userspace runtime | `services/go/` | Canonical TinyGo bootstrap lane: Go init, service manager, shell, and syscall-backed service. |
| Tooling and support | `tools/`, `.github/`, `vendor/`, `Makefile`, `Dockerfile` | Important, but not the product identity. |
| Validation | `tests/` | QEMU and artifact gate suite. Extensive, but secondary to the runtime. |
| Legacy and archive | `legacy/`, historical backlog docs | Useful for reference and closure history, not the active product story. |
| Experimental and research | `services/go_std/`, `tools/gostd_stock_builder/`, `docs/analysis/`, `docs/POST_G2_EXTENDED_MILESTONES.md` | Valuable, but should not read like the default runtime lane. |

## Architectural Priorities

1. Make the kernel path obvious.
2. Make the Go userspace path obvious.
3. Keep tooling and evidence strong, but visually secondary.
4. Keep legacy available, but clearly demoted to reference status.
5. Keep stock-Go work discoverable, but explicitly experimental.

## Related Docs

- repo strategy: [repo-strategy.md](repo-strategy.md)
- Go userspace bootstrap: [go_userspace_bootstrap_v1.md](go_userspace_bootstrap_v1.md)
- roadmap summary: [../roadmap/README.md](../roadmap/README.md)
- historical archive index: [../archive/README.md](../archive/README.md)
