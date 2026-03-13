# Hybrid OS Milestone Framework

Date: 2026-03-12
Status: active framing document

This document re-centers milestone reporting on the stated product goal:

- Rust `no_std` kernel for the core OS mechanisms
- Go user-space services as the default system personality
- Tooling, release process, hardware breadth, compatibility breadth, and GUI
  work treated as supporting or expansion tracks rather than the main score

Use [../../MILESTONES.md](../../MILESTONES.md) for the exhaustive legacy
ledger. Use this document for the architecture-first program view.

## What "closer to the goal" means

The project is closer to its goal only when the default Rust-kernel plus Go
service lane can do more of the following on declared baseline platforms:

1. Boot from firmware into the Rust kernel and enter Go user space without
   fallback lanes.
2. Run the default Go init/service manager/shell/service stack on native
   kernel primitives rather than stubs or one-off demo paths.
3. Persist data, perform network I/O, restart services, and recover from
   bounded failures in the default lane.
4. Enforce the process, rights, and isolation model that the default Go
   services depend on.
5. Stay correct under soak, restart, and containment tests tied to the default
   runtime path.

Work that mainly adds gates, policy, packaging scale, desktop breadth,
additional hardware classes, or broader compatibility is valuable, but it is
not the primary answer to "how close is the repo to the hybrid OS goal?"

## New Three-Track Taxonomy

### Track 1: Core Hybrid OS

This is the main scoreboard. A core milestone is complete only if it deepens
the default Rust-kernel plus Go-service runtime itself.

| New milestone | Status | Meaning of done | Historical mapping |
|---|---|---|---|
| `C1` Kernel Mechanisms Baseline | done | The Rust kernel boots, manages memory, schedules threads, enters user mode, supports syscall/IPC paths, and performs block I/O on the default lane. | `M0-M5` |
| `C2` Default Go Service Bring-up | done | The default lane boots into Go init/service/user workflows with filesystem, package, shell, and basic network support visible in the default demo path. | `M6`, `M7`, `G1` |
| `C3` Contracted Service OS Runtime | done | The service OS has explicit process/scheduler, rights, init/service lifecycle, task accounting, and bounded scheduler-control surfaces that the default Go services actually use on the booted lane. | `M10`, `M16`, `M25` |
| `C4` Durable and Connected Runtime | done | The default service lane stages and replays journal-backed state across boots, exposes ordered fsync/flush behavior on attached block media, configures interfaces/routes, and exercises IPv6 stream-socket lifecycle on the shipped `image-go` path. | `M12`, `M13`, `M18`, `M19` |
| `C5` Reliable and Isolated Service OS | done | The default lane survives boot-backed mixed service/storage/network soak, applies per-service isolation/resource limits, and exposes cleanup-aware task observability on the shipped `image-go` lane. | `M22`, `M42` |
| `C6` Runtime Quality Under Load | open | Latency, CPU affinity, memory-pressure behavior, and I/O QoS are explicit and test-backed for mixed service workloads on the default lane. | future core target; likely fed by `M78`, `M79` |

### Historical Core Backlog Order

The flat ledger still records `M10`, `M12`, `M13`, `M16`, `M18`, `M19`,
`M22`, `M25`, and `M42` as completed execution backlogs. This is the order the
repo used to close that historical core backlog series; it is not the same
thing as the current runtime-backed scoreboard:

1. `M10` and `M16` closed first.
2. `M25` extended the existing manifest-driven runtime rather than opening a
   new runtime lane.
3. `M12` and `M13` followed as the first connected and durable runtime
   expansion on that lane.
4. `M18` and `M19` closed later and are now reinforced by the same boot-backed
   default-lane runtime.
5. `M22` and `M42` now close that historical sequence with runtime-backed reliability and isolation on the same default lane.

### Track 2: Tooling / Validation / Release Infrastructure

These milestones increase confidence, reproducibility, and release discipline.
They are important, but secondary to the core runtime scoreboard.

| New milestone | Status | Meaning of done | Historical mapping |
|---|---|---|---|
| `T1` Supported Stock-Go Lane and ABI Qualification | done | The supported stock-Go lane, runtime coverage, and ABI stability rules are explicit, reproducible, and testable without being confused for the default product lane. | `G2`, `M11`, `M21` |
| `T2` Observability, Performance, and Evidence Discipline | done | Performance budgets, diagnostics, and runtime-backed evidence are release-gated with machine-readable artifacts and trace linkage. | `M24`, `M29`, `M40` |
| `T3` Release and Recovery Infrastructure | done | Release engineering, upgrade/recovery workflows, operability, and lifecycle policy are versioned and release-gated. | `M14`, `M20`, `M30`, `M31` |
| `T4` Hardening, Qualification, and Fleet Operations | done | Hardening programs, conformance, fleet rollout discipline, and maturity bundles are wired into repeatable gates and policy. | `M28`, `M32`, `M33`, `M34` |
| `T5` Advanced Trust and Compliance Infrastructure | open | Compliance evidence, richer release trust automation, chaos qualification, and higher-order telemetry stay tied to reproducible release gates. | future candidates: `M75`, `M76`, `M80`, `M81`, `M84` |

### Track 3: Expansion / Research / Platform Breadth

These milestones broaden what the system can target or what kinds of workflows
it can claim. They should not dominate the top-line progress signal.

| New milestone | Status | Meaning of done | Historical mapping |
|---|---|---|---|
| `X1` Compatibility Surface Expansion | done | The repo supports a larger external app and POSIX-like surface than the core Go-service lane strictly needs, with explicit deferred behavior and compatibility gates. | `M8`, `M17`, `M27`, `M36`, `M41` |
| `X2` Hardware, Firmware, and Driver Breadth | done | Support claims broaden from QEMU-centric reference platforms into tiered device, firmware, and audited claim-promotion programs. | `M9`, `M15`, `M23`, `M37`, `M43`, `M45`, `M46`, `M47` |
| `X3` Platform and Ecosystem Feature Breadth | done | Storage/platform extras, package/repo expansion, and catalog/distribution scale exceed the minimum core-service OS baseline. | `M26`, `M38`, `M39` |
| `X4` Desktop and Workflow Breadth | done | Display, input, windowing, GUI runtime, shell, installer, and bounded desktop workflows exist as an additional product surface on top of the core OS. | `M35`, `M44`, `M48`, `M49`, `M50`, `M51`, `M52` |
| `X5` Next-Wave Breadth Research | open | Future work expands native drivers, alternate architectures, advanced storage/network features, desktop usability breadth, SDK/app distribution, and community surface without being mistaken for core-runtime completion. | likely fed by `M53-M77`, `M82-M83` |

## Mapping From Old Milestones To New Tracks

This is the condensed remap for the current completed ledger.

| Old milestones | New track | New milestone |
|---|---|---|
| `M0-M5` | Core Hybrid OS | `C1` Kernel Mechanisms Baseline |
| `M6-M7`, `G1` | Core Hybrid OS | `C2` Default Go Service Bring-up |
| `M10`, `M16`, `M25` | Core Hybrid OS | `C3` Contracted Service OS Runtime |
| `M12`, `M13`, `M18`, `M19` | Core Hybrid OS | `C4` Durable and Connected Runtime |
| `M22`, `M42` | Core Hybrid OS | `C5` Reliable and Isolated Service OS |
| `G2`, `M11`, `M21` | Tooling / Validation / Release Infrastructure | `T1` Supported Stock-Go Lane and ABI Qualification |
| `M24`, `M29`, `M40` | Tooling / Validation / Release Infrastructure | `T2` Observability, Performance, and Evidence Discipline |
| `M14`, `M20`, `M30`, `M31` | Tooling / Validation / Release Infrastructure | `T3` Release and Recovery Infrastructure |
| `M28`, `M32`, `M33`, `M34` | Tooling / Validation / Release Infrastructure | `T4` Hardening, Qualification, and Fleet Operations |
| `M8`, `M17`, `M27`, `M36`, `M41` | Expansion / Research / Platform Breadth | `X1` Compatibility Surface Expansion |
| `M9`, `M15`, `M23`, `M37`, `M43`, `M45-M47` | Expansion / Research / Platform Breadth | `X2` Hardware, Firmware, and Driver Breadth |
| `M26`, `M38`, `M39` | Expansion / Research / Platform Breadth | `X3` Platform and Ecosystem Feature Breadth |
| `M35`, `M44`, `M48-M52` | Expansion / Research / Platform Breadth | `X4` Desktop and Workflow Breadth |

Legacy-only note:

- `G0` stays archived with the legacy lane. It should not appear on the
  default Rugo scoreboard.

## Implementation Closure Companions

The flat completion ledger and the three-track taxonomy still leave one
important question open: what would it take for the completed backlogs to be
literally implemented rather than merely closed by docs, gates, and artifacts?

Use the companion set in
[implementation_closure/README.md](implementation_closure/README.md):

- [implementation_closure/core_runtime.md](implementation_closure/core_runtime.md)
- [implementation_closure/tooling_and_release.md](implementation_closure/tooling_and_release.md)
- [implementation_closure/expansion_breadth.md](implementation_closure/expansion_breadth.md)
- [implementation_closure/next_wave_native.md](implementation_closure/next_wave_native.md)

The historical core-runtime backlog is closed in the ledger.
[implementation_closure/core_runtime.md](implementation_closure/core_runtime.md)
records `M10`, `M12`, `M13`, `M16`, `M18`, `M19`, `M22`, `M25`, and `M42` as
runtime-backed closures on the default lane.

Those documents cover every backlog-bearing milestone currently marked done in
the ledger, including `G2` and `M8-M54`.

## Recommended Renames

The following labels are more accurate for the architecture-first story:

| Current label | Recommended label | Why |
|---|---|---|
| `G1 Go services (TinyGo)` | `Default Go Service Bring-up (TinyGo-first)` | This is the actual default user-space path, not just a side Go experiment. |
| `G2 Full Go port` | `Supported Stock-Go Lane Qualification` | The repo now ships a supported stock-Go lane, but "full port" still overstates the visible scope. |
| `M34 Maturity Qualification + LTS Declaration` | `Release Qualification Bundle + LTS Policy` | The current evidence is strong process discipline, not broad runtime maturity parity. |
| `M39 Ecosystem Scale + Distribution Workflow v1` | `Catalog and Distribution Workflow Expansion v1` | The visible evidence is bounded catalog/install/audit workflows, not an ecosystem at real scale. |
| `M44 Real Desktop + Ecosystem Qualification v2` | `Bounded Desktop Workflow Qualification v2` | "Real desktop" implies broader end-user parity than the stated bounded workflow profile. |
| `M45 Modern Virtual Platform Parity v1` | `Modern VirtIO Platform Qualification v1 (shadow)` | The milestone is a shadow-gate qualification effort, not parity in the broad platform sense. |

## Likely Overclaiming If Foregrounded

These items are not invalid, but they overstate visible runtime substance when
used as top-line maturity proof:

| Milestone | Why it likely overclaims when foregrounded |
|---|---|
| `G2 Full Go port` | Evidence is a supported stock-Go artifact lane, not a default or feature-complete Go port. |
| `M34 Maturity Qualification + LTS Declaration` | LTS/process claims can outrun the still-bounded scope of the core runtime and supported product surface. |
| `M39 Ecosystem Scale + Distribution Workflow v1` | Simulation, install-rate, and audit workflows do not equal a demonstrated large ecosystem. |
| `M44 Real Desktop + Ecosystem Qualification v2` | The repo documents bounded desktop profiles and workflow claims, not broad desktop parity. |
| `M45 Modern Virtual Platform Parity v1` | Selected virtio qualification plus a shadow gate is not platform parity in the normal OS sense. |
| `M47 Hardware Claim Promotion Program v1` | This is about audited support claims and promotion policy, not deeper hybrid OS runtime capability. |

## Suggested README Scoreboard

Use a three-row top-level table instead of a flat "M0-M52 done" headline.

| Track | What counts as progress | Current phase | Historical mapping |
|---|---|---|---|
| Core Hybrid OS | The default Rust-kernel plus Go-service lane boots, runs native services, persists data, performs network I/O, and enforces runtime isolation on declared baseline targets. | `C3` done; `C4` done; `C5` done. | `M0-M7`, `G1`, `M10`, `M12`, `M13`, `M16`, `M18`, `M19`, `M22`, `M25`, `M42` |
| Tooling / Validation / Release Infrastructure | Confidence, reproducibility, qualification, release, and fleet discipline around the core lane improve. | `T4` complete; next infrastructure phase is `T5 Advanced Trust and Compliance Infrastructure`. | `G2`, `M11`, `M14`, `M20`, `M21`, `M24`, `M28`, `M29`, `M30-M34`, `M40` |
| Expansion / Research / Platform Breadth | Compatibility, hardware breadth, desktop breadth, packaging breadth, and other product-surface expansion increase. | `X4` complete; next breadth phase is `X5 Next-Wave Breadth Research`. | `M8`, `M9`, `M15`, `M17`, `M23`, `M26`, `M27`, `M35-M39`, `M41`, `M43-M52` |

Recommended note below the table:

- Main progress signal: the first row.
- `G1` should be described as the default Go-service lane.
- `G2` should be described as the supported stock-Go lane, not as the repo's
  primary state of completion.
- The flat milestone ledger remains available in [../../MILESTONES.md](../../MILESTONES.md).

## What To Foreground vs Archive

Foreground in `README.md` and `docs/roadmap/README.md`:

- The three-track scoreboard above, with Core Hybrid OS first.
- The observable definition of progress for the core lane.
- The default demo path and baseline evidence that show Rust kernel plus Go
  services actually running.
- The historical core backlog order
  (`M10/M16 -> M25 -> M12/M13 -> M18/M19/M22 -> M42`), not just the highest
  completed milestone number.

Archive or de-emphasize:

- Flat "M0-M52 done" headlines as the main story.
- Repeated checkpoint strings and "latest GUI milestone" style callouts.
- Detailed execution backlogs, per-gate ledgers, and long milestone closure
  histories.
- Hardware-promotion, release-policy, and desktop-expansion milestones as the
  first proof of overall maturity.

Practical rule:

- If a milestone can be removed without changing whether the Rust kernel boots
  the default Go-service OS and runs its native service workflows, it should
  not drive the main scoreboard.
