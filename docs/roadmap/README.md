# Roadmap Summary

This is the architecture-first roadmap view. Use it to understand direction.
Use [../../MILESTONES.md](../../MILESTONES.md) for the exhaustive completion
matrix and [../archive/README.md](../archive/README.md) for historical
execution records.

## Current Phase

Completed baseline:
- kernel and service foundation through `M0-M52`
- Go bring-up through `G1`
- stock-Go contract path through `G2`

Current repo-shaping priority:
- make the Rust-kernel plus Go-userspace identity obvious in structure,
  documentation, and demo paths before doing disruptive directory moves

## Milestone Streams

### Kernel and Runtime

- foundational kernel milestones: `M0-M7`
- runtime maturity and stability: `M11`, `M21`, `M22`, `M24`, `M29`, `M32`
- service and init model: `M25`, `M33`, `M34`

### Userspace and Compatibility

- compatibility and ABI surface: `M8`, `M16`, `M17`, `M27`, `M36`, `M41`
- TinyGo-first user-space path: `G1`
- stock-Go contract path: `G2` (experimental lane, not default repo identity)

### Hardware and Platform

- hardware matrix and promotion: `M9`, `M15`, `M23`, `M37`, `M43`, `M45`,
  `M46`, `M47`
- platform and storage breadth: `M18`, `M19`, `M38`, `M39`

### Security and Release

- security hardening and isolation: `M10`, `M28`, `M42`
- release, support, and operations: `M14`, `M20`, `M30`, `M31`

### Desktop and Workflow

- desktop baseline and ecosystem qualification: `M35`, `M44`
- display, input, windowing, toolkit, and shell: `M48`, `M49`, `M50`, `M51`,
  `M52`

## What Is Primary vs Secondary

Primary:
- Rust kernel
- Go userspace
- TinyGo-first demo path

Secondary but preserved:
- tooling and validation gates
- historical milestone backlogs
- legacy C baseline

Experimental:
- stock-Go porting work
- extended research roadmap details in `docs/POST_G2_EXTENDED_MILESTONES.md`
