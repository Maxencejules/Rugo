# Repo Strategy

This repository is being reshaped to read like a hybrid OS repository rather
than a tooling repository.

## Current Constraint

Build scripts, tests, CI, and milestone gates still reference the current
paths directly. A disruptive physical move today would break working flows
across `Makefile`, `tests/`, `.github/workflows/ci.yml`, and multiple tooling
scripts.

## Near-Term Strategy

Use an architecture-first documentation layer and explicit command aliases now,
then migrate paths only after the build and test system is path-parameterized.

Implemented now:
- architecture-first `README.md`
- architecture and roadmap docs under `docs/architecture/` and `docs/roadmap/`
- archive index under `docs/archive/`
- non-breaking make aliases: `make demo-go`, `make run-kernel`, `make validate`

Deferred until path-parameterization:
- moving `kernel_rs/` to `kernel/`
- moving `arch/` and `boot/` under the kernel tree
- moving `services/go/` under an explicit `userspace/` tree
- moving `tests/` under a `validation/` tree
- moving `tools/` under a `support/` tree

## Target Layout

```text
/
  kernel/
    arch/
    boot/
    src/

  userspace/
    tinygo/
    services/
    test-programs/

  experimental/
    stock-go-port/
    research/

  support/
    tools/
    build/
    release/

  validation/
    boot/
    runtime/
    hw/
    security/
    desktop/
    pkg/

  docs/
    architecture/
    roadmap/
    contracts/
    operations/
    archive/

  legacy/
```

## Migration Order

1. Keep current paths stable while the front-door docs and commands are fixed.
2. Introduce path variables/helpers in `Makefile`, scripts, tests, and CI.
3. Move kernel paths.
4. Move userspace paths.
5. Move support and validation paths.
6. Leave legacy as reference-only throughout.
