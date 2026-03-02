# Visual assets maintenance

This folder is the source of truth for README visuals.

## Rendering approach

Approach A is selected: Mermaid is kept inline in Markdown and as `.mmd` source files.
No rendering dependency is required.

## Files

- `architecture.mmd`: Limine -> Rust kernel -> syscall boundary -> Go services
- `boot-flow.mmd`: `make build`, `make image`, `make run`, `make test-qemu`
- `lanes.mmd`: Legacy vs Rugo lane status view from `MILESTONES.md`
- `syscall-boundary.mmd`: syscall groups from `docs/abi/syscall_v0.md`
- `screenshots/README.md`: strict screenshot/GIF capture rules

## Update workflow

1. Update the relevant `.mmd` file in this folder.
2. Mirror the same Mermaid block in `README.md`.
3. Verify every node label maps to a real path or documented concept.
4. If a status is ambiguous, label it `TODO/UNKNOWN` and cite the exact file needing clarification.

## Accuracy checklist

- Directory anchors: `boot/`, `kernel_rs/`, `arch/x86_64/`, `legacy/`, `services/`, `tools/`, `tests/`, `vendor/limine/`, `docs/`
- Syscall names and IDs must match `docs/abi/syscall_v0.md` exactly.
- If you add networking visuals, only reflect behaviors in `docs/net/udp_echo_v0.md`.

## Current TODO/UNKNOWN

- None currently.
