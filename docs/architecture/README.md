# Architecture Overview

Rugo is a hybrid OS with a strict language split:

- Rust `no_std` kernel for mechanisms, traps, memory management, scheduling,
  low-level drivers, and kernel-side ABI enforcement
- Go user space for services, policy, and the default higher-level system
  personality
- TinyGo-first as the default early integration lane for user space
- stock-Go support kept as a supported non-default lane while TinyGo-first
  remains the default demo path

## Current Source Map

Conceptual top-level labels:
- `kernel/`
- `userspace/`
- `validation/`
- `support/`
- `experimental/`

Current physical implementation:

| Bucket | Current paths | Notes |
|--------|---------------|-------|
| Core runtime | `arch/`, `boot/`, `kernel_rs/src/` | This is the actual kernel lane. |
| Userspace runtime | `services/go/` | Canonical TinyGo bootstrap lane: Go init, service manager, shell, and syscall-backed service. |
| Supported non-default runtime | `services/go_std/` | Stock-Go build, boot, and ABI qualification lane. |
| Tooling and support | `tools/`, `.github/`, `vendor/`, `Makefile`, `Dockerfile` | Important, but not the product identity. |
| Validation | `tests/` | Mix of live QEMU proofs, contract checks, and aggregate gate wiring. |
| Legacy and archive | `legacy/`, archived backlog docs | Useful for reference and closure history, not the active product story. |

## Implementation Versus Qualification

This repo needs to be read in two layers:

- implementation
  The runtime code lives in `arch/`, `boot/`, `kernel_rs/src/`, `services/go/`,
  and `services/go_std/`.
- qualification
  The repo also contains a large amount of deterministic evidence tooling,
  policy docs, and gate wiring under `tools/`, `tests/`, and `docs/`.

That distinction matters. Many later milestone lanes are meaningful repo
discipline, but they do not imply a similarly large runtime source tree.

See [SOURCE_MAP.md](SOURCE_MAP.md) for the proof-path table that separates live
runtime evidence from deterministic qualification scaffolding.

## Architectural Priorities

1. Make the kernel path obvious.
2. Make the Go userspace path obvious.
3. Keep tooling and evidence strong, but visually secondary.
4. Keep legacy available, but clearly demoted to reference status.
5. Keep supported non-default work discoverable without confusing it for the default
   runtime.

## Related Docs

- runtime source map: [SOURCE_MAP.md](SOURCE_MAP.md)
- repo strategy: [repo-strategy.md](repo-strategy.md)
- Go userspace bootstrap: [go_userspace_bootstrap_v1.md](go_userspace_bootstrap_v1.md)
- roadmap summary: [../roadmap/README.md](../roadmap/README.md)
- historical archive index: [../archive/README.md](../archive/README.md)
