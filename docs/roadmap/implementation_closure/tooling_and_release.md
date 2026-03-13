# Tooling And Release Backlog Closure

This document covers the backlog-bearing milestones that map into the tooling,
validation, release, and process track.

For this track, docs, policy, and automation are legitimate implementation
surfaces. The important distinction is narrower: a process milestone can be
real without proving new runtime breadth, and some current closures still rely
too heavily on deterministic reports rather than on artifacts produced by the
booted system.

## T1: Supported Stock-Go Lane And ABI Qualification

| Backlog | Current class | Current repo reading | Literal implementation target | Still required |
|---|---|---|---|---|
| `G2 Full Go port` | `Runtime-partial` | The repo now treats stock Go as a supported non-default userspace lane with build, boot, smoke, and ABI gates, but it is still not the default service stack. | Stock Go boots as a first-class supported userspace lane with enough ABI surface to run non-trivial services. | Port richer service workflows onto that lane or keep product claims limited to the supported non-default stock-Go surface. |
| `M11 Runtime + toolchain maturity v1` | `Process-backed` | Toolchain and maturity docs plus gates now feed the real stock-Go build path and emit shipping artifacts, but host breadth is still limited. | Supported toolchain versions, bootstrap steps, and host reproducibility are versioned and actually used to build shipping artifacts. | Verify reproducibility across the declared host matrix and keep toolchain breakage from silently drifting past the published policy. |
| `M21 ABI + API Stability Program v3` | `Process-backed` | ABI stability gates now derive kernel and stock-Go interface reports from source and fail on drift, but the governed userspace surface remains bounded. | Syscall and API compatibility windows are generated from real source-of-truth interfaces and enforced against real consumers. | Extend deprecation and compatibility reporting as more shipped userspace begins to consume the wider ABI line. |

## T2: Observability, Performance, And Evidence Discipline

| Backlog | Current class | Current repo reading | Literal implementation target | Still required |
|---|---|---|---|---|
| `M24 Performance Baseline + Regression Budgets v1` | `Evidence-first` | Performance budgets and reports exist, but they are not yet a convincing picture of mixed-service runtime behavior. | Performance budgets are measured from real booted workloads on declared targets and gate releases when they regress. | Benchmark the actual default lane, cover CPU, memory, block, and network workloads together, and keep the budget source tied to the same image the repo ships. |
| `M29 Observability + Diagnostics v2` | `Evidence-first` | Observability and crash-dump contracts exist, but there is no obviously broad live trace, metrics, and dump pipeline in the default lane. | Kernel and Go services emit structured logs, traces, and crash artifacts that can be collected, symbolized, and correlated after failure. | Implement runtime event streams, crash capture, symbolization hooks, and cross-layer identifiers that survive reboots and service restarts. |
| `M40 Runtime-Backed Evidence Integrity v1` | `Evidence-first` | The milestone explicitly wants runtime-backed evidence, but the repo still has many synthetic and deterministic artifact generators. | Release gates are driven by evidence emitted from the running system with traceable provenance back to the exact build and boot instance. | Move evidence generation into live runtime probes, sign or digest-link the emitted artifacts, and reject gate inputs that were not produced by the booted system under test. |

## T3: Release And Recovery Infrastructure

| Backlog | Current class | Current repo reading | Literal implementation target | Still required |
|---|---|---|---|---|
| `M14 Productization + release engineering v1` | `Process-backed` | Release docs, reports, and gates exist, but the repo still lacks a visibly mature shipping image story. | The project builds versioned installable artifacts, promotes them through defined channels, and can reproduce a release from source and inputs. | Standardize the image pipeline, attach signed metadata and changelogs, and make the release process drive the artifacts users actually boot. |
| `M20 Operability + Release UX v2` | `Process-backed` | Operability is described and tested at the contract level, not yet as a broad operator experience on live systems. | Operators can inspect health, collect diagnostics, and perform supported release actions on real systems with the documented workflow. | Add real support-bundle commands, health inspection surfaces, and operator-safe release tooling that runs against the booted image. |
| `M30 Installer/Upgrade/Recovery UX v3` | `Evidence-first` | Upgrade and recovery workflows are modeled and gated, but the repo does not visibly expose a full installer or recovery product path. | The system can install, upgrade, recover, and roll back on supported targets with the promised UX and safety rules. | Implement the installer and recovery environment, validate rollback floors on real artifacts, and tie the UX claims to bootable media and actual state transitions. |
| `M31 Release Engineering + Support Lifecycle v2` | `Process-backed` | Branch, support-window, and supply-chain policies are present, which is useful, but support claims outrun the visible runtime breadth. | Release branches, support windows, SBOMs, provenance, and revalidation tasks are attached to real shipping releases. | Make the support policy operate on actual release trains, automate revalidation from shipped SBOMs and attestations, and ensure support claims only cover what the runtime truly supports. |

## T4: Hardening, Qualification, And Fleet Operations

| Backlog | Current class | Current repo reading | Literal implementation target | Still required |
|---|---|---|---|---|
| `M28 Security Hardening Program v3` | `Evidence-first` | Threat-model, hardening, fuzz, and response artifacts exist, but the runtime hardening surface is still limited. | The kernel and image have concrete hardening defaults, mitigations, and security workflows that the shipping lane actually uses. | Implement more hardening knobs in the runtime, prove they are enabled in shipped artifacts, and tie fuzz and attack evidence to the real code paths that matter most. |
| `M32 Conformance + Profile Qualification v1` | `Process-backed` | Profile qualification suites exist as deterministic contracts and reports. | Published product profiles are qualified against real target systems with reproducible pass or fail bundles. | Run the conformance bundles on real images and target classes, define who owns profile drift, and fail releases when promised profile behavior is not present. |
| `M33 Fleet-Scale Operations Baseline v1` | `Evidence-first` | Fleet and rollout behavior is currently simulated rather than exercised against a believable deployment control plane. | A real staged rollout path can monitor health, halt bad canaries, and coordinate rollback across multiple running instances. | Build the update coordinator, health telemetry ingestion, canary controller, and abort logic, then run them against a small but real multi-node lab. |
| `M34 Maturity Qualification + LTS Declaration` | `Process-backed` | The repo has maturity and LTS policy documents, but the runtime scope they cover is still bounded. | LTS claims rest on an explicit support matrix, release cadence, regression budgets, and security response obligations for the actual product surface. | Narrow the LTS claim to what is genuinely supported, prove the qualification bundle on that surface, and keep the declaration tied to shipped releases rather than to milestone count. |

## Cross-Cutting Closure Work

- Make release, provenance, performance, and observability artifacts originate
  from the same images and boot flows that the repo claims to ship.
- Convert policy docs into the default automation path. A policy that only
  exists in docs and isolated tests is not yet a release discipline.
- Keep non-default lanes clearly labeled. `G2` should not be mistaken for the
  default product state unless the runtime really reaches that bar.
