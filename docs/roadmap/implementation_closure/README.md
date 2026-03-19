# Completed Backlog Implementation Closure

This directory turns the backlog-bearing milestones that are already marked
`done` or `completed` in [../../../MILESTONES.md](../../../MILESTONES.md) into
literal implementation-closure documents.

The point is simple: a backlog can be closed in the repo ledger while still
falling short of "the feature actually exists in the runtime." These documents
separate those two ideas and describe what would still need to be built before
the closure claim is literal.

## Scope

This set covers every completed execution backlog currently visible in the
ledger:

- `G2`
- `M8-M54`

It does not cover `M0-M7`, `G0`, or `G1`, because those are baseline
milestones rather than the later execution-backlog series.

## How To Read The Classifications

Each backlog in the companion docs is assigned one broad class:

- `Runtime-backed`: the main thing exists in the kernel or the default Go lane.
- `Runtime-partial`: a narrow implementation exists, but the backlog language is
  broader than what the runtime currently does.
- `Evidence-first`: closure is mostly docs, Python tooling, deterministic
  models, pytest gates, and CI wiring.
- `Process-backed`: the milestone is mainly about release, support, or policy
  discipline. Those can be real work, but they are not proof of runtime
  feature breadth.

These are not moral judgments. They are a way to distinguish "repo evidence"
from "product/runtime substance."

## Current Repo Anchors

The current repo has real runtime substance in a small number of places:

- `kernel_rs/src/lib.rs` contains the monolithic Rust kernel path for boot,
  traps, scheduling, syscalls, IPC/shared memory, VirtIO block I/O, and the
  VirtIO net UDP echo path.
- `services/go/main.go` contains a small default Go bootstrap with the
  hand-wired service-manager, time-service, and shell demo path.
- Most later milestones are expressed primarily as contracts under `docs/`,
  deterministic generators under `tools/`, pytest suites under `tests/`, and
  `Makefile` or CI gate wiring.

That means later backlog closure often proves that the repo can describe and
validate a feature contract, not that the feature is already implemented in the
default Rust-kernel plus Go-service product lane.

## Document Map

| Document | Coverage | Main question |
|---|---|---|
| [core_runtime.md](core_runtime.md) | `M10`, `M12`, `M13`, `M16`, `M18`, `M19`, `M22`, `M25`, `M42` | What is still missing in the actual kernel and default Go runtime? |
| [tooling_and_release.md](tooling_and_release.md) | `G2`, `M11`, `M14`, `M20`, `M21`, `M24`, `M28`, `M29`, `M30`, `M31`, `M32`, `M33`, `M34`, `M40` | What is still missing before these process and validation milestones are literal rather than merely gated? |
| [expansion_breadth.md](expansion_breadth.md) | `M8`, `M9`, `M15`, `M17`, `M23`, `M26`, `M27`, `M35-M39`, `M41`, `M43-M52` | What runtime, driver, package, and desktop work would make the breadth claims literal? |
| [next_wave_native.md](next_wave_native.md) | `M53-M54` | What native-driver and native-storage runtime closure now exists, and what breadth still remains? |

## Rule For Literal Closure

A completed backlog should only be treated as literally implemented when all of
the following are true:

1. The primary behavior exists in the runtime, not only in docs and test
   models.
2. The default product lane uses that behavior, or the milestone clearly says it
   belongs to an experimental lane.
3. The tests and CI gates are confirming real behavior from booted code paths,
   not only synthetic or schema-level evidence.
4. The declared support or compatibility claim matches the code that actually
   ships.

Use the companion docs as the conversion plan from "closed backlog history" to
"literal implementation state."
