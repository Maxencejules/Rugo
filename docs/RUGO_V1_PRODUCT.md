# Rugo v1 Product Definition

Date: 2026-03-19  
Status: draft

This document defines a bounded `v1` product target for Rugo.

It answers a different question than the milestone ledger:

- not "what backlogs have been closed?"
- but "what would count as a small, real OS product that someone could
  install, boot, use, update, and recover?"

Use this document with:

- [roadmap/README.md](roadmap/README.md)
- [roadmap/MILESTONE_FRAMEWORK.md](roadmap/MILESTONE_FRAMEWORK.md)
- [roadmap/implementation_closure/README.md](roadmap/implementation_closure/README.md)

## Product Statement

Rugo `v1` is a small, installable desktop OS with a Rust `no_std` kernel and a
Go service layer.

It is not trying to match the breadth of Windows, macOS, or a large Linux
distribution in its first release. The `v1` target is narrower:

- a single default product lane
- a tightly bounded support matrix
- a usable desktop session
- durable local storage
- baseline wired networking
- signed update and recovery flow
- clear diagnostics and support boundaries

If Rugo can do those things reliably on declared targets, it is a real product
OS. Breadth can expand after that.

## Product Rules

These rules keep the `v1` target honest:

1. The default shipped lane is the product.
   If the shipped default image does not exercise a behavior, that behavior
   does not count as product progress.
2. Runtime-backed behavior wins over ledger closure.
   Docs, generators, and gates matter, but they do not substitute for booted
   runtime behavior.
3. Support claims stay bounded.
   `v1` must name the exact virtual and physical targets it supports.
4. One coherent user story matters more than broad surface claims.
   Installer, desktop, storage, network, update, and recovery depth matter
   more than ecosystem or compatibility breadth.
5. Expansion follows proof.
   GPU, Wi-Fi, app-compat, and community-release claims should widen only after
   the default lane proves them end to end.

## Target User

The `v1` user is a developer or early adopter who wants a bounded but coherent
desktop OS experience on a declared target.

The `v1` user should be able to:

- install or boot the system without source-level intervention
- reach a desktop session consistently
- manage local files and settings
- use terminal and shell workflows
- connect to a network on the declared baseline
- install or update a curated package set
- recover from bounded failures using documented recovery flows

`v1` is not a broad consumer, enterprise, or general hardware release.

## v1 Support Matrix

The `v1` support matrix must stay narrow.

### Required release profile

| Area | `v1` baseline |
|---|---|
| Primary profile | `qemu-q35-default-desktop` |
| Firmware path | UEFI/boot path already used by the default lane |
| Storage | NVMe on the default path |
| Network | wired baseline on the declared default virtual target |
| Display | one declared desktop display path on the default desktop image |
| Input | keyboard and pointer on the default desktop image |
| Update medium | signed package/update path used by the default lane |

### Optional follow-on profiles

These do not belong in the initial `v1` promise unless they meet the same
runtime-backed bar:

- real reference desktop or laptop hardware
- AHCI fallback lane
- Wi-Fi adapter support
- broader GPU class support
- alternate userspace lanes

## Must-Ship v1 Scope

### Core runtime

- The default image boots from firmware into the Rust kernel and enters the
  default Go init and service runtime.
- The default lane provides stable process, syscall, storage, networking, and
  isolation behavior that the shipped services actually use.
- Crash, restart, and bounded soak behavior are tied to the shipped default
  image rather than only to synthetic qualification lanes.

### Storage and recovery

- The default lane uses durable storage on the declared `v1` target.
- Ordered flush, reboot persistence, and journal or replay behavior are visible
  in the live runtime.
- The system supports bounded update rollback and recovery boot flow.
- A user can reach diagnostics and recovery without source edits or debugger
  intervention.

### Networking

- The system provides baseline wired networking on the declared support matrix.
- The default desktop or shell workflows can consume that network path.
- Network configuration and failure state are visible through user-facing or
  operator-facing diagnostics.

### Desktop and workflows

- The system boots into a real desktop session on the default desktop image.
- Display, input, windowing, and shell flows are part of the shipped default
  experience.
- The desktop includes a minimal daily-use workflow set:
  - launcher or shell entry point
  - terminal
  - file management
  - settings or system-control surface
  - installer or recovery entry point

### App and package model

- Rugo `v1` ships with a curated package or app set that works on the default
  lane.
- The package and update path used by the user is the same one qualified by the
  release process.
- External app compatibility claims remain bounded to the real runtime corpus
  that the default lane can actually run.

### Security and release discipline

- Release artifacts are versioned and signed.
- Update and trust policy match the code and artifacts that actually ship.
- Default service isolation and capability boundaries are enabled in the
  shipped image.
- Vulnerability response, release notes, and recovery instructions exist for
  the declared support matrix.

### Diagnostics and supportability

- The system produces readable boot, runtime, and crash diagnostics for the
  declared targets.
- Support claims are explicit about what is in scope.
- Regression, conformance, and evidence gates stay attached to real booted
  runtime behavior on the shipped lane.

## Explicit Non-Goals For v1

The following are specifically out of scope for the initial `v1` product:

- broad parity with Windows, macOS, or mainstream Linux distributions
- universal hardware support
- broad laptop power-management parity
- multi-architecture release support
- large application catalog claims
- universal Linux compatibility claims
- advanced media and video acceleration breadth
- multiple desktop environments or shells
- fleet-scale or enterprise management beyond the bounded update path

## Release Bar

The release bar is staged. Each stage must remain subordinate to the default
product lane.

### Alpha

`alpha` proves that the product exists at all.

Required:

- bootable default image on the declared `q35` profile
- durable NVMe-backed default storage path
- wired networking on the declared profile
- boot to desktop or shell without source-level intervention
- install, update, and recovery path demonstrated on the shipped image
- readable crash and diagnostics flow

Repo proof path:

- `make test-product-alpha-v1`
- report: `out/product-alpha-v1.json`

Not required:

- broad hardware expansion
- Wi-Fi support
- strong GPU breadth beyond one declared display path
- community release-train promises

### Beta

`beta` proves that the product is coherent and testable as a daily-use bounded
system.

Required:

- desktop session is stable across repeated boot, suspend or resume if claimed,
  update, and rollback cycles
- one declared GPU path is runtime-backed if the desktop experience depends on
  it
- curated app and package set is usable on the shipped image
- installer, recovery, and diagnostics flows are operator-readable
- release artifacts, support matrix, and known limitations are published

Likely milestone dependencies:

- `M55` if GPU acceleration is part of the declared desktop baseline
- follow-on desktop or workflow milestones only if they are part of the shipped
  user story

### 1.0

`1.0` proves that the product can be supported honestly.

Required:

- the support matrix is explicit, small, and enforced
- release and update behavior are stable on the shipped image
- recovery flow works without engineering-only intervention
- default security, isolation, and signed-artifact paths are enabled
- public support and advisory promises match actual staffing and runtime scope

Conditional:

- `M56` is required only if Wi-Fi is part of the declared `1.0` support matrix
- `M84` is required once community release-train and support-channel promises
  become part of the public product contract

## Current Repo Reading Against v1

The current repo already has useful `v1` ingredients:

- runtime-backed core closure on the default lane
- runtime-backed desktop and workflow closure on a bounded desktop profile
- release, recovery, hardening, and evidence discipline
- a live NVMe runtime seed in the current M54 closure addendum

The main remaining `v1` risk is not lack of milestone names. It is making sure
the shipped default image is the thing that actually carries the product claim.

In practical terms, that means:

- keep moving critical behavior into the runtime rather than into only
  generators and gate wiring
- keep the support matrix narrow
- treat GPU, Wi-Fi, and public support promises as conditional `v1` scope, not
  automatic entitlement from the roadmap

## Success Condition

Rugo `v1` succeeds when a user can take the shipped default image for a
declared target and do all of the following without source edits:

- boot it
- reach the default desktop or shell experience
- persist data across reboot
- connect to the declared network baseline
- install or update the bounded app set
- recover from a bounded failure
- understand the system state from the provided diagnostics

If that is true, Rugo is no longer just a milestone-driven OS repo. It is a
small but real OS product.
