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
| `M24 Performance Baseline + Regression Budgets v1` | `Runtime-backed` | Performance baselines now come from booted default-image captures that exercise CPU, memory, block, network, restart, and mixed-service behavior before the regression gate runs. | Performance budgets are measured from real booted workloads on declared targets and gate releases when they regress. | Extend the same budget discipline to any broader target matrix before making claims beyond the default QEMU release lane. |
| `M29 Observability + Diagnostics v2` | `Runtime-backed` | The default lane now emits structured runtime logs, derived trace bundles, diagnostic snapshots, and boot-linked panic dumps that can be symbolized and correlated. | Kernel and Go services emit structured logs, traces, and crash artifacts that can be collected, symbolized, and correlated after failure. | Broaden runtime event coverage as the service graph grows, but keep the current trace and crash identifiers stable across future expansions. |
| `M40 Runtime-Backed Evidence Integrity v1` | `Runtime-backed` | Release gates now consume boot-backed runtime capture, performance, diagnostics, and panic artifacts with digest-linked provenance to the shipped image and boot instance. | Release gates are driven by evidence emitted from the running system with traceable provenance back to the exact build and boot instance. | Carry the same provenance discipline into any future non-QEMU promotion lane instead of falling back to synthetic-only evidence. |

## T3: Release And Recovery Infrastructure

| Backlog | Current class | Current repo reading | Literal implementation target | Still required |
|---|---|---|---|---|
| `M14 Productization + release engineering v1` | `Process-backed` | The repo now stages versioned `image-go` shipping bundles, release notes, signed update metadata, SBOMs, provenance, and support bundles from the same bootable artifacts users would install. | The project builds versioned installable artifacts, promotes them through defined channels, and can reproduce a release from source and inputs. | Extend the same shipping discipline beyond the default QEMU release lane before broadening support claims. |
| `M20 Operability + Release UX v2` | `Process-backed` | Operator workflows now run through a shared release bundle plus persisted install-state path, with support bundles and rollback context linked to bootable staged media and runtime capture. | Operators can inspect health, collect diagnostics, and perform supported release actions on real systems with the documented workflow. | Broaden the same operability UX beyond the current default-lane media reuse model as richer live-system surfaces land. |
| `M30 Installer/Upgrade/Recovery UX v3` | `Process-backed` | Installer, upgrade, rollback, and recovery drills now execute against versioned bootable media and install-state transitions derived from the shipped default lane bundle. | The system can install, upgrade, recover, and roll back on supported targets with the promised UX and safety rules. | Replace media reuse with role-specific runtime UX only when the default lane grows beyond the current shell and diagnostics surface. |
| `M31 Release Engineering + Support Lifecycle v2` | `Process-backed` | Lifecycle audits, SBOM/provenance revalidation, and attestation checks now attach to concrete release bundles and the exact bootable artifacts promoted through the default lane. | Release branches, support windows, SBOMs, provenance, and revalidation tasks are attached to real shipping releases. | Keep support claims scoped to the staged default lane until additional target classes receive the same release-bundle and revalidation treatment. |

## T4: Hardening, Qualification, And Fleet Operations

| Backlog | Current class | Current repo reading | Literal implementation target | Still required |
|---|---|---|---|---|
| `M28 Security Hardening Program v3` | `Runtime-backed` | Hardening now binds the attack suite to booted runtime capture, proves deny-path markers and service isolation defaults on the shipped image, and keeps advisory plus embargo workflows release-blocking. | The kernel and image have concrete hardening defaults, mitigations, and security workflows that the shipping lane actually uses. | Extend the same runtime-backed proof path as more kernel and service hardening knobs land. |
| `M32 Conformance + Profile Qualification v1` | `Runtime-backed` | Profile qualification now consumes booted runtime capture, release attestation, and package rebuild evidence and qualifies only the profiles the shipped lane actually satisfies. | Published product profiles are qualified against real target systems with reproducible pass or fail bundles. | Carry the same runtime-backed qualification flow to any future target classes before broadening profile claims. |
| `M33 Fleet-Scale Operations Baseline v1` | `Runtime-backed` | Fleet update, health, canary, and abort tooling now operate on a runtime-backed lab derived from the booted lane and coordinate rollback from live report inputs instead of synthetic-only summaries. | A real staged rollout path can monitor health, halt bad canaries, and coordinate rollback across multiple running instances. | Grow the same report-driven controller model beyond the scoped runtime lab only when additional supported lanes exist. |
| `M34 Maturity Qualification + LTS Declaration` | `Runtime-backed` | The maturity gate now aggregates booted runtime, security hardening, conformance, rollout, performance, package rebuild, and lifecycle evidence and narrows LTS to the explicit `qemu-q35-default-lane` server/appliance surface. | LTS claims rest on an explicit support matrix, release cadence, regression budgets, and security response obligations for the actual product surface. | Keep the declaration tied to the bounded default-lane support matrix until more surfaces receive the same qualification depth. |

## Cross-Cutting Closure Work

- Make release, provenance, performance, and observability artifacts originate
  from the same images and boot flows that the repo claims to ship.
- Convert policy docs into the default automation path. A policy that only
  exists in docs and isolated tests is not yet a release discipline.
- Keep non-default lanes clearly labeled. `G2` should not be mistaken for the
  default product state unless the runtime really reaches that bar.
