# Expansion And Breadth Backlog Closure

This document covers the completed backlog milestones that broaden
compatibility, hardware claims, packaging scope, and desktop surface.

Most of these milestones are useful, but most of them are also the easiest to
close from contracts, deterministic models, and CI gates without materially
changing the default runtime. The tables below spell out what would still need
to exist before each breadth claim is literal.

## X1: Compatibility Surface Expansion

| Backlog | Current class | Current repo reading | Literal implementation target | Still required |
|---|---|---|---|---|
| `M8 Compatibility Profile v1` | `Runtime-backed` | The default lane now loads real ELF payloads for the external package path and proves file plus loader behavior through the runtime compatibility corpus. | A declared compatibility subset runs real external programs on the default lane with stable loader, process, file, and socket behavior. | Extend the same real-binary corpus as more supported APIs graduate, but do not broaden claims without new runtime-backed programs. |
| `M17 Compatibility Profile v2` | `Runtime-backed` | The v2 lane is now tied to a real-ELF runtime corpus instead of only richer contracts and gate models. | The supported compatibility profile is wider in the actual runtime, not only in the documents. | Keep loader, signal, fd, and error-surface growth attached to the same runtime-backed lane before raising profile scope. |
| `M27 External App Compatibility Program v3` | `Runtime-backed` | Tiering and class matrices now sit beside representative real ELF apps that launch on the default lane and through the package bootstrap path. | The declared app tiers launch and behave on the real system with packaging, lifecycle, and failure reporting that users can consume. | Add more representative third-party binaries before widening tier claims beyond the bounded runtime corpus. |
| `M36 Compatibility Surface Expansion v1` | `Runtime-backed` | The milestone now pairs compatibility and POSIX accounting with a live runtime corpus that exercises loader, file, socket, and readiness paths. | The OS actually exposes the additional process, file, socket, and readiness surfaces that the profile claims. | Keep deferred behavior explicit in code and extend the real corpus whenever new supported surfaces are added. |
| `M41 Process + Readiness Compatibility Closure v1` | `Runtime-backed` | Fork, clone, epoll, and readiness behavior are now checked at the ABI boundary by real ELF probes in addition to the contract suites. | The runtime either implements these surfaces or rejects them with consistent, documented behavior at the ABI boundary. | Only replace the stable non-support path when a real runtime implementation and corpus coverage exist. |

## X2: Hardware, Firmware, And Driver Breadth

The historical X2 hardware backlog is now runtime-backed through the shared
device registry, probe/bind lifecycle bundle, firmware plus SMP evidence, and
declared target-class qualification gate in
`docs/hw/x2_hardware_runtime_qualification_v1.md` and
`make test-x2-hardware-runtime-v1`.

| Backlog | Current class | Current repo reading | Literal implementation target | Still required |
|---|---|---|---|---|
| `M9 Hardware enablement matrix v1` | `Runtime-backed` | The historical q35 and i440fx matrix is now anchored to shared target qualification entries that carry deterministic boot and runtime markers for transitional block and net classes. | The system boots and exercises declared hardware classes with driver behavior that matches the published matrix. | Extend only by adding new target-class captures or stronger live-lab runs; do not widen claims without updating the shared X2 qualification lane. |
| `M15 Hardware Enablement Matrix v2` | `Runtime-backed` | v2 policy and tiering now feed the same X2 device registry and target gate instead of living only as separate matrix docs. | More device classes become truly bootable and supportable. | Keep any new tier policy bound to the same runtime-backed target evidence instead of reintroducing policy-only expansion. |
| `M23 Hardware Enablement Matrix v3` | `Runtime-backed` | Measured boot, firmware resilience, suspend or hotplug diagnostics, and target qualification now land in one runtime-backed hardware bundle. | Hardware and firmware claims are backed by real boot validation, measured boot flow, and device operation on supported targets. | Broaden only when new platforms carry the same firmware-attested target qualification depth. |
| `M37 Hardware Breadth + Driver Matrix v4` | `Runtime-backed` | The driver matrix now feeds a reusable device registry and probe or bind lifecycle foundation instead of staying isolated as a milestone-specific artifact. | The kernel has a real driver lifecycle with probe, bind, recover, detach, and negative-path handling across supported devices. | Add future device classes through the shared registry and lifecycle lane rather than standalone matrices. |
| `M43 Hardware/Firmware Breadth + SMP v1` | `Runtime-backed` | Firmware table validation, SMP interrupt behavior, and affinity-safe driver expectations now back the X2 runtime gate directly. | The kernel brings up multiple CPUs, validates firmware tables, routes interrupts correctly, and runs declared hardware classes under that model. | Preserve the same SMP-safe evidence when adding new devices or interrupt models. |
| `M45 Modern Virtual Platform Parity v1` | `Runtime-backed` | Modern VirtIO transport, display binding, and bind-lifecycle evidence now qualify explicit modern target classes in the shared X2 lane. | Modern virtual platforms are supported by the real runtime, not only qualified on paper. | Keep new modern-platform claims tied to qualified targets and live lifecycle evidence. |
| `M46 Bare-Metal I/O Baseline v1` | `Runtime-backed` | Wired NIC, USB input, removable media, desktop input, and recovery paths now roll into named Tier 2 target classes with runtime marker capture. | The OS can use baseline bare-metal network, input, and removable-media hardware in the live system. | Add broader bare-metal classes only after they join the same target-class qualification path. |
| `M47 Hardware Claim Promotion Program v1` | `Runtime-backed` | Promotion and support-tier audit now consume the same target-class qualification and device-registry evidence that the live X2 lane emits. | Support claims are promoted only after real boot, install, suspend, resume, I/O, and recovery campaigns on the target tier. | Keep promotion policy subordinate to the runtime-backed target evidence; do not add claim rows without a corresponding qualified target class. |

## X3: Platform And Ecosystem Feature Breadth

The historical X3 platform and ecosystem backlog is now runtime-backed through
the shared package/update/storage/catalog qualification bundle in
`docs/pkg/x3_platform_ecosystem_runtime_qualification_v1.md` and
`make test-x3-platform-runtime-v1`.

| Backlog | Current class | Current repo reading | Literal implementation target | Still required |
|---|---|---|---|---|
| `M26 Package/Repo Ecosystem v3` | `Runtime-backed` | The default lane now boots a real `pkgsvc` path that verifies signed metadata, persists package state, rotates trust material, and services shell-driven package work through the shared X3 runtime gate. | The system has a real repository client or server flow, reproducible packages, trust rotation, and update behavior used by the default lane. | Extend future package/repo breadth only through the same runtime-backed service path and real package corpus rather than reopening a policy-only lane. |
| `M38 Storage + Platform Feature Expansion v1` | `Runtime-backed` | Snapshot, resize, xattr, reflink, and capability negotiation now execute on persistent runtime media through the booted package/platform service flow instead of only deterministic models. | The filesystem and platform layers support the claimed advanced features in the real runtime. | Broaden storage/platform claims only by extending the same runtime-backed media and service path with new real-media captures. |
| `M39 Ecosystem Scale + Distribution Workflow v1` | `Runtime-backed` | Catalog load, staged promotion, install telemetry, audit linkage, and rollback-safe distribution now run through the live `pkgsvc` replay flow and the shared X3 aggregate gate. | The repo supports a real package catalog, distribution flow, and audit trail at the declared scale. | Add broader catalogs or replication tiers only after they join the same runtime-backed distribution lane with real package evidence. |

## X4: Desktop And Workflow Breadth

The historical X4 desktop and workflow backlog is now runtime-backed through
the shared desktop profile runtime qualification bundle in
`docs/desktop/desktop_profile_runtime_v1.md` and
`make test-desktop-profile-runtime-v1`.

| Backlog | Current class | Current repo reading | Literal implementation target | Still required |
|---|---|---|---|---|
| `M35 Desktop + Interactive UX Baseline v1` | `Runtime-backed` | The desktop profile now boots a real `os-go-desktop.iso` userspace path and the shared X4 gate binds the baseline desktop/session artifacts to that booted image. | The system can start a real interactive desktop session with display, input, windows, and basic GUI behavior. | Extend only by widening the same boot-backed desktop profile and shared X4 qualification lane with new declared workflows or target classes. |
| `M44 Real Desktop + Ecosystem Qualification v2` | `Runtime-backed` | The bounded desktop workflows and app/catalog qualification reports now sit beside a booted desktop-profile image that emits matching shell, GUI, and installer markers. | A real desktop session can launch, render, manage files, and host supported GUI apps on the declared targets. | Broaden only by attaching new GUI/app classes and workflow evidence to the same desktop-profile image and X4 aggregate gate. |
| `M48 Display Runtime + Scanout v1` | `Runtime-backed` | Display and scanout artifacts now feed the same desktop-profile boot path instead of standing alone as contracts. | The kernel and desktop stack can drive scanout on supported virtual or physical display devices. | Keep future display claims subordinate to the shared desktop-profile runtime lane before widening supported outputs. |
| `M49 Input + Seat Management v1` | `Runtime-backed` | Seat and HID reports now bind to the live desktop-profile image through explicit boot markers and aggregate qualification. | Keyboard, pointer, hotplug, and seat routing work in the live system with consistent focus semantics. | Add broader input classes only after they join the same desktop-profile runtime lane and boot-backed evidence path. |
| `M50 Window System + Composition v1` | `Runtime-backed` | Window lifecycle and compositor behavior now feed the shared desktop-profile runtime qualification report instead of remaining isolated lower-layer artifacts. | The desktop stack manages windows, composition, damage, focus, and resize or move semantics in real time. | Broaden composition or client protocols only by extending the same boot-backed desktop-profile lane. |
| `M51 GUI Runtime + Toolkit Bridge v1` | `Runtime-backed` | GUI runtime, toolkit, and font evidence now sit beside a booted desktop-profile image that emits matching GUI bridge markers. | Supported toolkits can render, process events, and interoperate with the OS window and input stack. | Add richer toolkit or text behavior only after it is wired into the same desktop-profile image and aggregate gate. |
| `M52 Desktop Shell + Workflow Baseline v1` | `Runtime-backed` | Desktop shell and installer workflows are now tied to a real desktop-profile image instead of remaining workflow-only evidence. | The OS ships a working shell, launcher, settings path, and bounded graphical workflows on top of the desktop runtime. | Keep broader shell or installer claims subordinate to the shared desktop-profile runtime lane with real boot-backed evidence. |

## Cross-Cutting Closure Work

- Define the declared supported targets first. Breadth milestones only become
  meaningful when the repo says which hardware, virtual platforms, and desktop
  profiles are actually in scope.
- Build reusable subsystems instead of milestone-specific models. Driver core,
  input stack, display stack, package service, and compatibility layer work
  should each land once in the runtime and then be reused by many milestones.
- Keep breadth milestones subordinate to the core lane. A new matrix, desktop,
  or compatibility doc should not be mistaken for proof that the default hybrid
  OS is broadly complete.
