# Post-G2 Extended Milestones (Research Roadmap)

Date: 2026-03-04  
Status: Complete (M8-M14 complete)  
Scope: Rugo lane after G2 completion

---

## Why this document exists

M0-M7 and G1-G2 prove the kernel and Go-service direction. This roadmap answers:

1. What is still missing to approach Linux/Windows/macOS class capability.
2. How to implement those gaps with realistic sequencing.
3. What "done" looks like for each step.

This is a research-backed implementation plan, not just a feature wishlist.

---

## Summary of the 7 missing areas

1. Compatibility layer and app ecosystem.
2. Real hardware support breadth.
3. Security hardening and isolation.
4. Runtime and toolchain maturity.
5. Full networking stack maturity.
6. Filesystem and storage robustness.
7. Productization (updates, release, supply chain, operations).

---

## Milestone map (proposed)

| Milestone | Focus | Primary missing area |
|---|---|---|
| M8 | Compatibility Profile v1 | 1 |
| M9 | Hardware Enablement Matrix v1 | 2 |
| M10 | Security Baseline v1 | 3 |
| M11 | Runtime + Toolchain Maturity v1 | 4 |
| M12 | Network Stack v1 | 5 |
| M13 | Storage Reliability v1 | 6 |
| M14 | Productization + Release Engineering v1 | 7 |

Recommended sequencing: M8 -> M9 -> M10 -> M11 -> M12 -> M13 -> M14.

Reason: compatibility and hardware unlock real workloads; security/toolchain/network/storage make them reliable; productization makes them sustainable.

---

## M8 - Compatibility Profile v1

### Goal

Define and implement a stable user-space compatibility profile that can run meaningful third-party software, not just project demos.

### Implementation strategy

1. Freeze an ABI profile:
   - System call surface versioned as `syscall_v1`.
   - x86_64 SysV calling convention and ELF64 loader behavior documented.
2. Build a POSIX-first API target:
   - Start with high-value process, file, signal, time, socket, poll/epoll, futex primitives.
   - Track explicitly unsupported APIs.
3. Introduce compatibility tiers:
   - Tier A: native Rugo binaries.
   - Tier B: POSIX-oriented apps built against selected libc profile.
   - Tier C: optional Linux ABI shim (if adopted).
4. Add ecosystem bootstrap:
   - Minimal package format and repository metadata.
   - Base userland profile (shell, core tools, init/service manager).

### Work packages

- M8.1 ABI contract and test fixtures (`docs/abi/syscall_v1.md`, ABI tests).
- M8.2 ELF loader hardening (dynamic linking, relocations, aux vector policy).
- M8.3 libc target decision (`musl`-style compatibility profile or equivalent).
- M8.4 package manager and repository signing format v1.
- M8.5 compatibility CI lane (build-run matrix for canonical user-space apps).

### Exit criteria

- Versioned ABI document with no unresolved TODOs.
- At least one external app suite installs and runs from packages.
- Compatibility test suite reaches a fixed pass threshold and is release-gated.

### Acceptance tests (examples)

- Static and dynamic ELF binaries run with expected loader semantics.
- Process/thread/signal regression pack passes.
- File/socket API conformance pack passes for selected POSIX subset.

---

## M9 - Hardware Enablement Matrix v1

### Goal

Move from mostly QEMU-centric support to a maintainable, tiered hardware target matrix.

### Implementation strategy

1. Define hardware tiers:
   - Tier 0: QEMU reference platforms.
   - Tier 1: common x86_64 desktop/laptop virtual and bare-metal targets.
   - Tier 2: opportunistic/experimental.
2. Build a unified driver lifecycle:
   - Probe -> init -> runtime health -> teardown.
   - Standard IRQ, DMA, and MMIO patterns.
3. Expand device coverage deliberately:
   - Storage: NVMe/AHCI following virtio-blk maturity.
   - Network: additional NIC families beyond current baseline.
   - Input/timers/power needed for interactive system use.
4. Add hardware regression harness:
   - Boot and smoke tests per target class.
   - Device capability and fallback reporting in serial logs and artifacts.

### Work packages

- M9.1 `docs/hw/support_matrix_v1.md`.
- M9.2 PCI subsystem cleanup for multi-driver registration and resource arbitration.
- M9.3 DMA safety checks and IOMMU strategy draft.
- M9.4 ACPI and UEFI table handling hardening.
- M9.5 bare-metal bring-up runbook and diagnostics.

### Exit criteria

- Published and CI-enforced hardware matrix with pass/fail history.
- Deterministic boot + storage + network smoke on all Tier 0 and Tier 1 targets.
- Driver model conventions adopted across in-tree drivers.

### Acceptance tests (examples)

- Per-device probe/init markers and negative-path tests.
- Hotplug and reset path tests for supported devices.
- DMA boundary and invalid-mapping rejection tests.

---

## M10 - Security Baseline v1

### Goal

Establish enforceable least-privilege and secure-boot/update foundations.

### Implementation strategy

1. Adopt explicit rights model:
   - Per-handle/per-object rights with reduction and transfer semantics.
2. Tighten memory and execution protections:
   - W^X discipline, executable mapping policy, kernel/user separation checks.
3. Introduce sandboxing primitives:
   - Per-process syscall and resource policy (seccomp/landlock-like direction).
4. Build trusted boot chain:
   - Signed boot artifacts, measured boot/attestation-ready metadata.
5. Add continuous vulnerability discovery:
   - Coverage-enabled syscall fuzzing.
   - Security regression packs as release gates.

### Work packages

- M10.1 rights and capability model doc + kernel enforcement.
- M10.2 secure boot policy and key rotation procedure.
- M10.3 syscall filtering framework and default profiles.
- M10.4 kernel fuzz harness integration (coverage + crash triage loop).
- M10.5 incident response and security advisory process.

### Exit criteria

- Rights model enforced by default for user-space resource access.
- Boot path validates signed artifacts.
- Fuzzing runs continuously with tracked crash SLA and fix workflow.

### Acceptance tests (examples)

- Unauthorized operations denied with deterministic errors.
- Tampered boot artifacts rejected.
- Sandbox profiles prevent forbidden syscalls and paths.

---

## M11 - Runtime and Toolchain Maturity v1

Milestone status: done (2026-03-04).

### Goal

Turn the G2 stock-Go bring-up path into a maintainer-grade, release-gated
`GOOS=rugo` port with reproducible toolchain outputs and a documented ABI
stability window.

### Current baseline (post-M10)

- G2 proves stock-Go artifact generation (`tools/build_go_std_spike.sh`,
  `tools/gostd_stock_builder/main.go`).
- Runtime bridge markers and contract metadata are test-backed
  (`tests/go/test_std_go_binary.py`, `docs/abi/go_port_spike_v0.md`).
- Compatibility, hardware, and security gates are already release-gated.
- Missing for M11 closure: runtime syscall/behavior coverage accounting,
  full bootstrap/rebuild workflow, a port qualification gate, and ABI stability
  policy docs.

### Implementation strategy

1. Freeze target and ABI contracts:
   - publish runtime-facing syscall contract and deprecation window policy.
   - declare required toolchain versions and deterministic build knobs.
2. Close runtime syscall and behavior gaps:
   - build a coverage matrix from runtime requirements.
   - implement or explicitly defer unsupported paths with deterministic behavior.
3. Build a reproducible bootstrap lane:
   - add a one-command setup/check flow for host and container environments.
   - emit machine-readable contract/reproducibility artifacts.
4. Add release-blocking qualification:
   - create an `all.bash`-style port qualification target.
   - gate CI on runtime stress and ABI-window checks.
5. Establish long-term ownership:
   - name maintainers and define breakage/deprecation workflow.

### Execution plan (3 PRs)

#### PR-1: Port Contract + Runtime Coverage Matrix

##### Objective

Define what the runtime port guarantees, then make all gaps explicit and
measurable.

##### Scope

- Add docs:
  - `docs/runtime/port_contract_v1.md`
  - `docs/runtime/syscall_coverage_matrix_v1.md`
  - `docs/runtime/abi_stability_policy_v1.md`
- Map runtime-required syscall/behavior surface against current kernel
  implementation.
- Add doc contract tests for completeness and format.
- Record unsupported features as explicit deferred items with owner/date.

##### Acceptance checks

- `python -m pytest tests/runtime/test_runtime_contract_docs_v1.py -v`
- `python -m pytest tests/go/test_std_go_binary.py -v`

##### Done criteria for PR-1

- Coverage matrix has no unowned TODO rows.
- ABI stability window and compatibility policy are versioned and reviewable.

#### PR-2: Bootstrap + Reproducibility Toolchain Lane

##### Objective

Make `GOOS=rugo` build/test setup reproducible across hosts and CI.

##### Scope

- Add bootstrap artifacts:
  - `tools/bootstrap_go_port_v1.sh`
  - `tools/runtime_toolchain_contract_v1.py`
  - optional reference container file (`tools/runtime_port.Dockerfile`).
- Emit deterministic contract/report artifacts:
  - `out/runtime-toolchain-contract.env`
  - `out/runtime-toolchain-repro.json`
- Document end-to-end setup and rebuild workflow:
  - `docs/runtime/toolchain_bootstrap_v1.md`

##### Acceptance checks

- `bash tools/bootstrap_go_port_v1.sh --check`
- `python tools/runtime_toolchain_contract_v1.py --out out/runtime-toolchain-contract.env`
- `python tools/runtime_toolchain_contract_v1.py --repro --out out/runtime-toolchain-repro.json`

##### Done criteria for PR-2

- A clean environment can reproduce expected artifacts with documented steps.
- Contract and repro artifacts are stable across two fresh runs.

#### PR-3: Qualification Gate + Milestone Closure

##### Objective

Create a release-blocking M11 gate and wire ownership/process requirements.

##### Scope

- Add runtime qualification target:
  - `make test-runtime-maturity`
- Add CI gate:
  - `.github/workflows/ci.yml` step `Runtime + toolchain maturity v1 gate`.
- Add runtime qualification tests:
  - `tests/runtime/test_runtime_stress_v1.py`
  - `tests/runtime/test_runtime_abi_window_v1.py`
- Record maintainers and breakage policy:
  - `docs/runtime/maintainers_v1.md`
- Wire status docs for milestone closure once gate is green.

##### Acceptance checks

- `make test-runtime-maturity`
- `python -m pytest tests/runtime -v`
- `python -m pytest tests/go/test_std_go_binary.py tests/compat/test_posix_subset.py -v`

##### Done criteria for PR-3

- Runtime maturity gate is mandatory in CI.
- Port ownership and deprecation workflow are documented.
- M11 is evidence-backed and ready to be marked `done`.

### Work packages

- M11.1 `GOOS=rugo` port contract v1 (`docs/runtime/port_contract_v1.md`).
- M11.2 runtime syscall/behavior coverage matrix + owned gap list.
- M11.3 bootstrap + reproducibility toolchain lane and artifacts.
- M11.4 `all.bash`-style qualification gate (`make test-runtime-maturity` + CI).
- M11.5 ABI stability + maintainership/deprecation policy docs.

### Exit criteria

- Runtime coverage matrix is complete; all open gaps are closed or explicitly
  deferred with owner/date.
- Toolchain bootstrap and reproducibility flow is documented, executable, and
  artifact-backed.
- Port has named maintainers and release-blocking CI gate.

### Acceptance tests (examples)

- Runtime stress tests (threads, timers, net IO, memory pressure) pass.
- Cross-version binary compatibility tests pass for frozen ABI window.
- Toolchain contract/repro artifacts match across clean rebuilds.

### Risks and non-goals

- Non-goals:
  - shipping full upstream-Go port acceptance in this milestone.
  - solving network/storage feature parity issues owned by M12/M13.
- Risks to track:
  - hidden runtime assumptions in unsupported syscall paths.
  - host toolchain drift causing false reproducibility failures.

---

## M12 - Network Stack v1

Milestone status: done (2026-03-04).

### Goal

Turn the M7 UDP-echo baseline into a maintainer-grade, release-gated network
lane with explicit socket semantics, TCP correctness, and interop/soak
evidence.

### Current baseline (post-M11)

- M7 provides a kernel-side VirtIO-net polling path with ARP + IPv4 UDP echo
  coverage (`tests/net/test_udp_echo.py`, `docs/net/udp_echo_v0.md`).
- ABI v1 already reserves raw frame networking via `sys_net_send` and
  `sys_net_recv` (`docs/abi/syscall_v1.md`), but no network v1 socket contract
  is published yet.
- QEMU fixtures already support deterministic UDP inject and tiered net-machine
  profiles (`tests/conftest.py`: `_boot_iso_with_net`, `qemu_serial_net_*`).
- M9 hardware matrix proves basic net probe/init across tiered machine profiles.
- Kickoff gaps (closed in M12): versioned network contracts, TCP state/timer
  correctness coverage, IPv6 baseline docs/tests, and a dedicated network CI
  maturity gate.

### Implementation strategy

1. Freeze network contracts and observability:
   - Publish versioned L2/L3/L4/socket docs with deterministic error behavior.
   - Define serial/trace artifacts for repeatable network triage.
2. Raise transport correctness:
   - Keep UDP behavior deterministic under load/loss.
   - Add TCP state machine baseline with retransmission/timer rules.
3. Lock socket semantics:
   - Define blocking/non-blocking and poll integration behavior.
   - Document unsupported operations explicitly as deterministic failures.
4. Add IPv6 baseline:
   - Addressing, ICMPv6 essentials, and neighbor discovery baseline.
5. Promote reliability to release gates:
   - Add interop and soak/fault-injection lanes with artifact outputs.

### Execution plan (3 PRs)

#### PR-1: Contract Freeze + IPv4/UDP Baseline Gate

##### Objective

Make current IPv4/UDP behavior explicit and test-gated before adding TCP/IPv6
complexity.

##### Scope

- Add docs:
  - `docs/net/network_stack_contract_v1.md`
  - `docs/net/socket_contract_v1.md`
  - `docs/net/ipv4_udp_profile_v1.md`
- Add/extend executable checks:
  - `tests/net/test_udp_echo.py` (positive + deterministic negative paths)
  - `tests/net/test_ipv4_udp_contract_v1.py`
  - `tests/net/test_socket_contract_docs_v1.py`
- Add optional trace/report helper:
  - `tools/net_trace_capture_v1.py`

##### Acceptance checks

- `python -m pytest tests/net/test_udp_echo.py tests/net/test_ipv4_udp_contract_v1.py -v`
- `python -m pytest tests/net/test_socket_contract_docs_v1.py -v`

##### Done criteria for PR-1

- Network v1 contract docs are versioned and test-referenced.
- IPv4/UDP behavior (including failure paths) is deterministic and executable.

#### PR-2: TCP State/Timer Engine + Socket Semantics

##### Objective

Implement a deterministic TCP baseline and bind it to explicit socket behavior
contracts.

##### Scope

- Add TCP baseline behavior:
  - handshake, established data flow, close/teardown, reset handling.
  - retransmission and timer policy with deterministic retry/fail semantics.
- Add tests:
  - `tests/net/test_tcp_state_machine_v1.py`
  - `tests/net/test_tcp_retransmission_v1.py`
  - `tests/net/test_socket_poll_semantics_v1.py`
- Add docs:
  - `docs/net/tcp_state_machine_v1.md`
  - `docs/net/retransmission_timer_policy_v1.md`

##### Acceptance checks

- `python -m pytest tests/net/test_tcp_state_machine_v1.py tests/net/test_tcp_retransmission_v1.py -v`
- `python -m pytest tests/net/test_socket_poll_semantics_v1.py -v`

##### Done criteria for PR-2

- TCP lifecycle and timer/retransmission behavior are deterministic and
  test-backed.
- Socket poll/blocking semantics are documented and enforced by tests.

#### PR-3: IPv6 Baseline + Interop/Soak Gate + Milestone Closure

##### Objective

Close M12 with IPv6 baseline behavior and mandatory network maturity gates in
local and CI lanes.

##### Scope

- Add IPv6 baseline docs/tests:
  - `docs/net/ipv6_baseline_v1.md`
  - `tests/net/test_ipv6_nd_icmpv6_v1.py`
- Add interop/soak tooling and checks:
  - `tools/run_net_interop_matrix_v1.py`
  - `tools/run_net_soak_v1.py`
  - `tests/net/test_net_interop_matrix_v1.py`
  - `tests/net/test_net_soak_v1.py`
- Add release gates:
  - `Makefile` target `test-network-stack-v1`
  - `.github/workflows/ci.yml` step `Network stack v1 gate`
- Mark milestone/status docs once gate is green.

##### Acceptance checks

- `make test-network-stack-v1`
- `python -m pytest tests/net -v`
- `python tools/run_net_interop_matrix_v1.py --out out/net-interop-v1.json`
- `python tools/run_net_soak_v1.py --out out/net-soak-v1.json`

##### Done criteria for PR-3

- Network maturity gate is mandatory in CI.
- IPv4/UDP/TCP/IPv6 baseline behavior has interop + soak evidence artifacts.
- M12 is evidence-backed and marked `done` in milestone/status docs.

### Work packages

- M12.1 network stack contract docs v1 (IPv4/UDP/TCP/socket semantics).
- M12.2 TCP state/timer and retransmission engine with traceability artifacts.
- M12.3 IPv6 baseline (addressing, ND, ICMPv6 essentials) and contract tests.
- M12.4 network soak/fault-injection harness and machine-readable reports.
- M12.5 interop matrix + release-blocking local/CI network maturity gate.

### Exit criteria

- Socket/network contracts are versioned, executable, and release-gated.
- Deterministic TCP/UDP suites pass under clean and degraded link conditions.
- Interop + soak lanes produce stable reports and satisfy target thresholds.

### Acceptance tests (examples)

- RFC-aligned UDP/TCP conformance vectors (including failure paths).
- DNS + HTTPS client flow succeeds under clean and degraded links.
- Connection churn and retransmission stress suites pass.

### Risks and non-goals

- Non-goals:
  - full Linux socket API parity in M12.
  - advanced congestion-control variants beyond a baseline policy.
  - full routing/firewall/NAT feature set in this milestone.
- Risks to track:
  - QEMU user-net behavior may hide physical-wire edge cases.
  - timer granularity drift may destabilize retransmission determinism.
  - interop flakiness without pinned peer versions/configs.

---

## M13 - Storage Reliability v1

Milestone status: done (2026-03-04).

### Goal

Turn the current functional `SimpleFS v0` path into a crash-consistent,
recoverable, and release-gated storage reliability lane with explicit
durability semantics.

### Current baseline (post-M12)

- M6 delivers functional filesystem behavior with deterministic mount and app
  package flow (`tests/fs/test_fsd_smoke.py`, `tests/pkg/test_pkg_install_run.py`).
- On-disk format is documented as `SimpleFS v0` (`docs/storage/fs_v0.md`) and
  built deterministically by `tools/mkfs.py`.
- Basic invalid-superblock rejection exists (`tests/fs/test_fsd_bad_magic.py`).
- M5/M9 already provide deterministic VirtIO block probe/read/write baselines.
- Kickoff gaps (closed in M13): crash model and durability contract,
  write-ordering guarantees, replay/recovery tooling,
  corruption/power-loss campaigns, and a dedicated storage reliability CI gate.

### Implementation strategy

1. Freeze storage correctness contracts first:
   - define v1 crash model, durability classes, and `fsync`/flush guarantees.
   - document deterministic failure-path behavior for partial/aborted writes.
2. Implement recovery-capable write path:
   - choose reliability strategy (journaled metadata path or equivalent).
   - enforce write ordering and barrier policy in block-facing storage path.
3. Add verification and repair tooling:
   - replay/recovery utility, integrity checks, and offline verifier workflow.
4. Build realistic fault campaigns:
   - model repeated power-loss/reset points and corruption injection.
5. Promote reliability to release gates:
   - storage fault/recovery reports as mandatory local/CI evidence.

### Execution plan (3 PRs)

#### PR-1: Storage Contract Freeze + Crash Model Baseline

##### Objective

Define storage v1 guarantees and make durability semantics executable before
recovery implementation lands.

##### Scope

- Add docs:
  - `docs/storage/fs_v1.md`
  - `docs/storage/durability_model_v1.md`
  - `docs/storage/write_ordering_policy_v1.md`
- Add/extend executable checks:
  - `tests/storage/test_storage_contract_docs_v1.py`
  - `tests/storage/test_fsync_semantics_v1.py`
  - `tests/storage/test_write_ordering_contract_v1.py`
- Record explicit deferred items with owner/target date where needed.

##### Acceptance checks

- `python -m pytest tests/storage/test_storage_contract_docs_v1.py -v`
- `python -m pytest tests/storage/test_fsync_semantics_v1.py tests/storage/test_write_ordering_contract_v1.py -v`

##### Done criteria for PR-1

- Storage v1 contracts are versioned, reviewable, and test-referenced.
- Crash/durability semantics have no unowned placeholder rows.

#### PR-2: Replay/Recovery Tooling + Fault-Campaign Harness

##### Objective

Implement and validate deterministic recovery behavior under corruption and
power-loss scenarios.

##### Scope

- Add tooling:
  - `tools/storage_recover_v1.py`
  - `tools/run_storage_fault_campaign_v1.py`
- Add tests:
  - `tests/storage/test_storage_recovery_v1.py`
  - `tests/storage/test_storage_fault_campaign_v1.py`
  - `tests/storage/test_storage_integrity_checker_v1.py`
- Add docs:
  - `docs/storage/recovery_playbook_v1.md`
  - `docs/storage/fault_campaign_v1.md`

##### Acceptance checks

- `python tools/storage_recover_v1.py --check --out out/storage-recovery-v1.json`
- `python tools/run_storage_fault_campaign_v1.py --seed 20260304 --out out/storage-fault-campaign-v1.json`
- `python -m pytest tests/storage/test_storage_recovery_v1.py tests/storage/test_storage_fault_campaign_v1.py -v`

##### Done criteria for PR-2

- Recovery and fault-campaign tooling emits stable machine-readable artifacts.
- Corruption/power-loss scenarios produce deterministic pass/fail outcomes.

#### PR-3: Storage Reliability Gate + Milestone Closure

##### Objective

Close M13 with release-blocking storage reliability gates and evidence-linked
status closure.

##### Scope

- Add local gate:
  - `Makefile` target `test-storage-reliability-v1`
- Add CI gate:
  - `.github/workflows/ci.yml` step `Storage reliability v1 gate`
- Add aggregate storage test lane:
  - `tests/storage/test_storage_reliability_gate_v1.py`
- Wire artifact uploads:
  - `out/storage-recovery-v1.json`
  - `out/storage-fault-campaign-v1.json`
- Mark milestone/status docs once gate is green.

##### Acceptance checks

- `make test-storage-reliability-v1`
- `python -m pytest tests/storage -v`
- `python tools/storage_recover_v1.py --check --out out/storage-recovery-v1.json`
- `python tools/run_storage_fault_campaign_v1.py --seed 20260304 --out out/storage-fault-campaign-v1.json`

##### Done criteria for PR-3

- Storage reliability gate is mandatory in CI.
- Recovery/fault artifacts are stable and linked in milestone evidence.
- M13 is ready to be marked `done` in milestone/status docs.

### Work packages

- M13.1 storage v1 contracts (crash model, durability classes, fsync semantics).
- M13.2 write-ordering/barrier policy with deterministic failure behavior.
- M13.3 replay/recovery tooling + offline integrity verifier.
- M13.4 power-loss/corruption fault-campaign harness + artifact schema.
- M13.5 release-blocking local/CI storage reliability gate.

### Exit criteria

- Crash-consistency and durability claims are versioned, explicit, and tested.
- Recovery tooling restores a clean mountable state across campaign scenarios.
- Storage reliability artifacts are generated and enforced by release gates.

### Acceptance tests (examples)

- Repeated forced power cuts during metadata/data update phases.
- Post-recovery integrity/replay checks on clean and corrupted images.
- Mixed workload stress (large-file, many-file, metadata-heavy) with bounded
  failure thresholds.

### Risks and non-goals

- Non-goals:
  - full production filesystem feature parity (quotas, snapshots, online resize).
  - complete CoW snapshot/rollback system in M13 unless explicitly selected.
  - broad storage hardware family expansion beyond current virtio baseline.
- Risks to track:
  - hidden ordering assumptions between FS metadata and block flush behavior.
  - long fault campaigns increasing CI flakiness/runtime.
  - recovery correctness gaps that only appear under repeated power cuts.

---

## M14 - Productization and Release Engineering v1

Milestone status: done (2026-03-04).

### Goal

Turn the current milestone-complete platform into a shippable, supportable
product lane with versioned releases, signed updates, supply-chain evidence,
and installer/recovery baselines enforced by release gates.

### Current baseline (post-M13)

- M8-M13 are complete with release-blocking gates for compatibility, hardware,
  security, runtime/toolchain, network, and storage.
- Build determinism already exists at image level (`make repro-check`,
  `tools/mkimage.sh`, `docs/BUILD.md`).
- Security baseline already includes signed boot-manifest and key-rotation
  policy artifacts (`tools/secure_boot_manifest_v1.py`,
  `docs/security/secure_boot_policy_v1.md`).
- M11/M12/M13 already established the machine-readable artifact pattern in
  `out/*.json` and CI artifact uploads (`.github/workflows/ci.yml`).
- Kickoff gaps for M14: no release-channel contract, no signed OTA/update
  repository/client workflow, no SBOM/provenance release artifacts, no
  installer/recovery operation contract, and no dedicated release-engineering
  gate.

### Implementation strategy

1. Freeze release governance and channel contracts first:
   - stable/beta/nightly semantics, support windows, backport policy, and
     ownership/approval flow.
2. Build a signed update path with explicit rollback defenses:
   - signed metadata/targets, monotonic version policy, key-rotation workflow,
     and offline verification behavior.
3. Add supply-chain evidence to every releasable artifact:
   - SBOM output, provenance attestations, and artifact manifest checksums.
4. Promote reproducibility + release assembly to mandatory gates:
   - deterministic release-image checks and policy validation in local/CI.
5. Define operator-facing product UX baseline:
   - installer flow, recovery mode, and support bundle collection contract.

### Execution plan (3 PRs)

#### PR-1: Release Contract Freeze + Channel Governance

##### Objective

Make release/versioning policy executable before update transport mechanics are
implemented.

##### Scope

- Add release contract docs:
  - `docs/build/release_policy_v1.md`
  - `docs/build/versioning_scheme_v1.md`
  - `docs/build/release_checklist_v1.md`
- Add contract/report helper:
  - `tools/release_contract_v1.py`
- Add executable checks:
  - `tests/build/test_release_contract_docs_v1.py`
  - `tests/build/test_release_contract_report_v1.py`
- Freeze release-role ownership and escalation paths in docs.

##### Acceptance checks

- `python tools/release_contract_v1.py --out out/release-contract-v1.json`
- `python -m pytest tests/build/test_release_contract_docs_v1.py tests/build/test_release_contract_report_v1.py -v`

##### Done criteria for PR-1

- Release policy is versioned, reviewable, and test-referenced.
- Channel/support/backport rules have no unowned placeholders.
- Release contract report schema is stable.

#### PR-2: Signed Update Pipeline + Rollback Protection

##### Objective

Ship a deterministic, attack-aware OTA/update baseline with explicit replay,
freeze, and rollback defenses.

##### Scope

- Add update protocol docs:
  - `docs/pkg/update_protocol_v1.md`
  - `docs/pkg/update_repo_layout_v1.md`
  - `docs/security/update_signing_policy_v1.md`
- Add update tooling:
  - `tools/update_repo_sign_v1.py`
  - `tools/update_client_verify_v1.py`
  - `tools/run_update_attack_suite_v1.py`
- Add executable checks:
  - `tests/pkg/test_update_metadata_v1.py`
  - `tests/pkg/test_update_rollback_protection_v1.py`
  - `tests/pkg/test_update_attack_suite_v1.py`

##### Acceptance checks

- `python tools/update_repo_sign_v1.py --repo out/update-repo-v1 --out out/update-metadata-v1.json`
- `python tools/update_client_verify_v1.py --repo out/update-repo-v1 --expect-version 1.0.0`
- `python tools/run_update_attack_suite_v1.py --seed 20260304 --out out/update-attack-suite-v1.json`
- `python -m pytest tests/pkg/test_update_metadata_v1.py tests/pkg/test_update_rollback_protection_v1.py tests/pkg/test_update_attack_suite_v1.py -v`

##### Done criteria for PR-2

- Update metadata/targets are signed and verifiable with documented key-rotation
  behavior.
- Replay/freeze/rollback attack simulations have deterministic pass/fail
  outcomes.
- Update reports are machine-readable and schema-validated.

#### PR-3: Supply-Chain + Reproducibility Gate + Installer/Recovery Closure

##### Objective

Close M14 with release-blocking productization gates and evidence-linked
milestone closure.

##### Scope

- Add supply-chain docs/tooling:
  - `docs/build/supply_chain_policy_v1.md`
  - `tools/generate_sbom_v1.py`
  - `tools/generate_provenance_v1.py`
- Add installer/recovery baseline docs/tooling:
  - `docs/build/installer_recovery_baseline_v1.md`
  - `tools/collect_support_bundle_v1.py`
- Add release gate wiring:
  - `Makefile` target `test-release-engineering-v1`
  - `.github/workflows/ci.yml` step `Release engineering v1 gate`
- Add aggregate checks:
  - `tests/build/test_release_engineering_gate_v1.py`
- Wire release artifacts:
  - `out/release-contract-v1.json`
  - `out/update-attack-suite-v1.json`
  - `out/sbom-v1.spdx.json`
  - `out/provenance-v1.json`
- Mark milestone/status docs once gate is green.

##### Acceptance checks

- `make test-release-engineering-v1`
- `make repro-check`
- `python tools/generate_sbom_v1.py --out out/sbom-v1.spdx.json`
- `python tools/generate_provenance_v1.py --out out/provenance-v1.json`
- `python -m pytest tests/build/test_release_engineering_gate_v1.py -v`

##### Done criteria for PR-3

- Release-engineering gate is mandatory in local and CI lanes.
- Release artifacts include reproducibility, update-security, SBOM, and
  provenance evidence.
- M14 is evidence-backed and ready to be marked `done` in milestone/status docs.

### Work packages

- M14.1 release policy and versioning scheme.
- M14.2 signed OTA/update repository + client verification path.
- M14.3 provenance + SBOM artifact emission and schema validation in CI.
- M14.4 release-blocking reproducible build and release-assembly gate.
- M14.5 installer, recovery, and support-bundle operational baseline.

### Exit criteria

- Release/channel policy is versioned, executable, and owner-closed.
- Signed OTA/update flow enforces rollback/replay/freeze defenses.
- Every release candidate emits reproducibility + SBOM + provenance artifacts.
- Release engineering gate is mandatory for local and CI release lanes.

### Acceptance tests (examples)

- Update attack simulations (replay, freeze, rollback, key-rotation drills).
- Fresh install + upgrade + rollback + recovery mode end-to-end flow.
- Multi-run release-image reproducibility and artifact checksum verification.
- Support-bundle generation and redaction policy checks.

### Risks and non-goals

- Non-goals:
  - full enterprise fleet-management platform in M14.
  - complete package ecosystem governance beyond baseline update/release policy.
  - broad hardware-specific installer UX permutations beyond current QEMU/virtio
    baseline.
- Risks to track:
  - update signing key lifecycle mistakes causing lockout or unsafe bypass paths.
  - reproducibility drift from host/toolchain variance outside pinned baseline.
  - CI runtime growth from combined reproducibility + security + artifact gates.
  - installer/recovery flows diverging from documented operator procedures.

---

## Cross-cutting governance (applies to M8-M14)

1. Every milestone has:
   - design doc,
   - threat model update,
   - CI lane,
   - rollback plan.
2. No milestone is marked done without:
   - at least one negative-path test suite,
   - performance baseline capture,
   - release note with known limitations.
3. Keep milestone cadence fixed:
   - target 8-12 weeks per milestone,
   - avoid parallelizing more than two high-risk milestones at once.

---

## Suggested immediate next actions

1. Keep the release-engineering gate green and trendable:
   - reproducibility pass rate,
   - update attack-suite pass rate,
   - SBOM/provenance artifact completeness.
2. Extend the post-G2 dashboard with productization signals:
   - OTA success and rollback trend,
   - SBOM/provenance coverage,
   - release reproducibility pass rate.
3. Freeze steady-state ownership:
   - one milestone owner,
   - one release-owner,
   - one update-pipeline owner,
   - one doc owner.

---

## Extension: M53-M84 roadmap after M52 closure

This extension continues the roadmap from the current M52 desktop-shell
baseline. It keeps the existing contract-first approach: every milestone
introduces versioned docs, deterministic gates, explicit negative paths, and
auditable promotion criteria before broad claims are made.

### Sequencing summary

| Range | Area | Outcome |
|---|---|---|
| M53-M57 | Hardware support and drivers | Native driver breadth, firmware policy, and shadow multi-arch bring-up |
| M58-M61 | Filesystems and storage | Journaled, encrypted, multi-device, and integrity-focused storage |
| M62-M65 | Networking and connectivity | Firewall, VPN, wireless control plane, routing/QoS |
| M66-M70 | User interface and desktop | Accessibility, file workflows, settings, multi-monitor, productivity shell |
| M71-M74 | Application ecosystem and packaging | Package solver, app bundles, SDK, federated catalog |
| M75-M77 | Security, isolation, and compliance | Update orchestration, compliance profiles, attested fleet admission |
| M78-M81 | Performance, stability, and observability | Scheduler/QoS, telemetry, chaos qualification |
| M82-M84 | Community, docs, and release engineering | Governance, localization, public release operations |

### Hardware Support and Drivers

#### Milestone ID: M53 - Native Driver Contract Expansion v1

- Description: Freeze the post-M52 hardware expansion boundary for native
  PCIe-class devices beyond virtio/USB basics. Define reusable no_std driver
  contracts for MSI/MSI-X, DMA/IOMMU safety, firmware loading, power states,
  and user-visible diagnostics before NVMe/GPU/Wi-Fi drivers land.
- Dependencies: M45-M47, M48-M52, `docs/hw/support_matrix_v6.md`,
  `docs/hw/driver_lifecycle_contract_v6.md`.
- Contracts/Interfaces: `docs/hw/native_driver_contract_v1.md`,
  `docs/hw/pcie_dma_contract_v1.md`,
  `docs/hw/firmware_blob_policy_v1.md`,
  `docs/hw/native_driver_diag_schema_v1.md`.
- Tests/Gates: `make test-native-driver-contract-v1`;
  `tests/hw/test_native_driver_docs_v1.py`,
  `tests/hw/test_pcie_dma_contract_v1.py`,
  `tests/hw/test_firmware_blob_policy_v1.py`; serial markers
  `DRV: bind`, `DMA: map ok`, `FW: denied unsigned`.
- Hardware Tiers: QEMU Tier 0/Tier 1 remain the release floor; native NVMe,
  GPU, and Wi-Fi rows are added as Tier 1 candidate or Tier 2 shadow targets
  only after contract audit.
- Implications: Reproducibility is preserved by keeping firmware outside the
  base kernel image and pinning it with signed manifests. The security model
  gains an explicit firmware trust boundary and stricter DMA containment rules.

#### Milestone ID: M54 - Native Storage Drivers v1

- Description: Implement AHCI and NVMe baseline drivers with identify/admin
  paths, queue setup, IRQ handling, flush/FUA semantics, namespace discovery,
  reset recovery, and bounded power-management hooks so storage is no longer
  virtio-only.
- Dependencies: M53, M38, M45-M47, `docs/storage/fs_v1.md`.
- Contracts/Interfaces: `docs/hw/nvme_ahci_contract_v1.md`,
  `docs/hw/support_matrix_v7.md`,
  `docs/storage/block_flush_contract_v1.md`.
- Tests/Gates: `make test-native-storage-v1`;
  `tests/hw/test_nvme_identify_v1.py`,
  `tests/hw/test_nvme_io_queue_v1.py`,
  `tests/hw/test_ahci_rw_v1.py`,
  `tests/storage/test_nvme_fsync_integration_v1.py`; serial markers
  `NVME: ready`, `AHCI: port up`, `BLK: fua ok`.
- Hardware Tiers: Emulated NVMe becomes a Tier 1 target; audited bare-metal
  AHCI/NVMe desktops remain Tier 2 until repeatable reset and durability
  campaigns stay green.
- Implications: Throughput and latency improve materially, but reproducibility
  now depends on pinned namespace geometry and deterministic flush timing. DMA
  and interrupt isolation become release-blocking.

#### Milestone ID: M55 - GPU Driver Baseline + Acceleration v1

- Description: Add a bounded GPU lane with modeset, buffer allocation, fence
  signaling, hardware cursor, and 2D acceleration baseline. Keep policy in the
  Go compositor/shell while the kernel exposes only the minimal queue,
  framebuffer, and sync primitives needed for interactive desktop use.
- Dependencies: M53, M48-M52, `docs/desktop/gpu_fallback_policy_v1.md`.
- Contracts/Interfaces: `docs/hw/gpu_driver_contract_v1.md`,
  `docs/desktop/gpu_userspace_abi_v1.md`,
  `docs/desktop/gpu_fallback_policy_v2.md`.
- Tests/Gates: `make test-gpu-accel-v1`;
  `tests/hw/test_gpu_probe_v1.py`,
  `tests/desktop/test_gpu_modeset_v1.py`,
  `tests/desktop/test_gpu_scanout_accel_v1.py`,
  `tests/desktop/test_gpu_fallback_v2.py`; serial markers
  `GPU: probe ok`, `GPU: modeset ok`, `GPU: fallback clean`.
- Hardware Tiers: Virtio-GPU stays Tier 1; native GPU families enter Tier 2
  first and do not reach Tier 1 until security review, crash-dump coverage, and
  deterministic fallback evidence exist.
- Implications: Desktop viability improves sharply, but the GPU path enlarges
  the attack surface. Proprietary firmware must remain optional, separately
  signed, and excluded from default audited images.

#### Milestone ID: M56 - Wireless Adapter + Firmware Baseline v1

- Description: Add a first real Wi-Fi adapter lane with PCIe/USB probe,
  firmware load, station-mode connect, WPA2/WPA3 baseline hooks, regulatory
  domain handling, power-save, and bounded recovery from RF loss. Kernel work
  stays at device and frame primitives; supplicant policy remains in Go.
- Dependencies: M53, M46, M54, M12, M19.
- Contracts/Interfaces: `docs/hw/wifi_driver_contract_v1.md`,
  `docs/net/wifi_control_plane_contract_v1.md`,
  `docs/security/firmware_provenance_policy_v1.md`.
- Tests/Gates: `make test-wifi-baseline-v1`;
  `tests/hw/test_wifi_adapter_probe_v1.py`,
  `tests/net/test_wifi_auth_assoc_v1.py`,
  `tests/net/test_wifi_power_save_v1.py`,
  `tests/net/test_wifi_firmware_policy_v1.py`; serial markers
  `WIFI: firmware ok`, `WIFI: assoc ok`, `WIFI: denied unsigned-fw`.
- Hardware Tiers: Wi-Fi begins as Tier 2 bare-metal only; promotion to Tier 1
  requires audited adapter allowlists, signed firmware provenance, and two-run
  reproducible roam/reconnect evidence.
- Implications: Laptop readiness improves, but proprietary firmware handling is
  now a first-class supply-chain and revocation problem. Release images must
  distinguish blob-free and signed-firmware variants.

#### Milestone ID: M57 - Secondary Architecture Shadow Bring-Up v1

- Description: Establish an experimental multi-arch framework for a secondary
  `aarch64` QEMU target while keeping x86-64 as the only release floor. The
  milestone focuses on boot, interrupt/MMU abstraction, arch-neutral syscall
  contracts, and Go runtime smoke rather than full platform parity.
- Dependencies: M53-M56, M11, M21, M43.
- Contracts/Interfaces: `docs/runtime/arch_port_contract_v1.md`,
  `docs/hw/multi_arch_support_policy_v1.md`,
  `docs/abi/arch_neutral_syscall_profile_v1.md`.
- Tests/Gates: `make test-multiarch-shadow-v1`;
  `tests/boot/test_aarch64_boot_smoke_v1.py`,
  `tests/runtime/test_arch_neutral_syscall_profile_v1.py`,
  `tests/go/test_go_port_shadow_v1.py`; serial markers
  `ARCH: aarch64 boot ok`, `ARCH: syscall ok`.
- Hardware Tiers: x86-64 remains Tier 0/Tier 1. `aarch64` enters Tier 2 shadow
  only, with no stable support claim and no release promotion before separate
  MMU, interrupt, and toolchain audits.
- Implications: Portability improves without diluting the x86-64 roadmap, but
  emulator drift and per-arch security differences can easily damage
  reproducibility if not isolated from the main release floor.

### Filesystems and Storage

#### Milestone ID: M58 - Journaling Filesystem Baseline v1

- Description: Introduce a journaled filesystem baseline with metadata
  transactions, optional ordered-data mode, journal checksums, replay-on-mount,
  quota hooks, and deterministic power-fail behavior. Update image and mkfs
  tooling so journal state is reproducible.
- Dependencies: M13, M18, M38, M54, `docs/storage/fs_v1.md`.
- Contracts/Interfaces: `docs/storage/fs_journal_contract_v1.md`,
  `docs/storage/journal_replay_policy_v1.md`,
  `docs/abi/fs_quota_contract_v1.md`.
- Tests/Gates: `make test-storage-reliability-v3`;
  `tests/storage/test_journal_replay_v1.py`,
  `tests/storage/test_journal_checksum_v1.py`,
  `tests/storage/test_quota_contract_v1.py`,
  `tests/storage/test_powerfail_matrix_v1.py`; serial markers
  `FSJ: commit ok`, `FSJ: replay ok`.
- Hardware Tiers: Tier 1 on virtio-blk and NVMe profiles; Tier 2 on audited
  bare-metal storage only after replay and quota campaigns stay deterministic.
- Implications: Data integrity improves substantially, but no_std memory
  overhead and image-build complexity rise. `tools/mkfs.py` and recovery
  tooling must remain byte-for-byte reproducible.

#### Milestone ID: M59 - Encryption + Keyslot Baseline v1

- Description: Add at-rest encryption for block volumes or filesystem roots
  with keyslots, passphrase unlock, recovery key path, and optional measured
  unlock integration. Keep secrets handling minimal and explicit; no hidden
  auto-unlock flows.
- Dependencies: M58, M10, M28, M31.
- Contracts/Interfaces: `docs/storage/encrypted_volume_contract_v1.md`,
  `docs/security/storage_key_policy_v1.md`,
  `docs/abi/storage_crypto_syscalls_v1.md`.
- Tests/Gates: `make test-storage-encryption-v1`;
  `tests/storage/test_volume_unlock_v1.py`,
  `tests/storage/test_encrypted_powerfail_recovery_v1.py`,
  `tests/security/test_storage_key_policy_v1.py`; serial markers
  `ENC: unlock ok`, `ENC: denied bad-key`.
- Hardware Tiers: Tier 1 on virtio/NVMe with software crypto fallback; Tier 2
  for TPM-bound unlock on audited secure-boot hardware only.
- Implications: Confidentiality improves, but CI must use deterministic escrow
  fixtures rather than real secrets. The security model now includes key
  rotation, unlock policy, and panic-path secret zeroization.

#### Milestone ID: M60 - Multi-Device RAID Baseline v1

- Description: Add mirrored and striped multi-device volume support with
  degraded boot policy, rebuild/scrub, write-intent tracking, and deterministic
  failure handling for disk loss and reintegration.
- Dependencies: M58-M59, M54.
- Contracts/Interfaces: `docs/storage/raid_contract_v1.md`,
  `docs/storage/device_set_policy_v1.md`,
  `docs/storage/rebuild_scrub_policy_v1.md`.
- Tests/Gates: `make test-storage-raid-v1`;
  `tests/storage/test_raid_mirror_rebuild_v1.py`,
  `tests/storage/test_raid_degraded_boot_v1.py`,
  `tests/storage/test_raid_scrub_v1.py`; serial markers
  `RAID: degraded ok`, `RAID: rebuild ok`.
- Hardware Tiers: Tier 1 in QEMU dual-disk virtio/NVMe profiles; Tier 2 for
  audited bare-metal mirror sets and hot-replace workflows.
- Implications: Availability improves, but rebuild ordering and fault-injection
  campaigns can become non-deterministic without fixed seeds and pinned disk
  timing models.

#### Milestone ID: M61 - CoW Snapshots + Integrity Repair v1

- Description: Add copy-on-write snapshots/subvolumes, end-to-end checksums,
  background scrub, and repair semantics for mirrored data so Rugo has a
  bounded Btrfs/APFS-class feature lane without claiming unlimited feature
  parity.
- Dependencies: M58-M60, M38.
- Contracts/Interfaces: `docs/storage/cow_snapshot_contract_v1.md`,
  `docs/storage/integrity_scrub_policy_v1.md`,
  `docs/abi/snapshot_management_contract_v1.md`.
- Tests/Gates: `make test-storage-integrity-v1`;
  `tests/storage/test_cow_snapshot_semantics_v1.py`,
  `tests/storage/test_scrub_selfheal_v1.py`,
  `tests/storage/test_snapshot_send_receive_v1.py`; serial markers
  `SNAP: create ok`, `SCRUB: repaired`.
- Hardware Tiers: Tier 1 on NVMe and mirrored virtio profiles; Tier 2 on
  audited bare-metal workstations and server-class storage sets.
- Implications: Rollback and repair become much stronger, but metadata
  amplification and memory pressure must be budgeted explicitly for no_std
  environments and reproducible image generation.

### Networking and Connectivity

#### Milestone ID: M62 - Packet Filter + Firewall Primitives v1

- Description: Add kernel packet-filter hooks, stateful flow tables, interface
  zoning, and packet mark primitives while keeping rule authoring and policy
  composition in Go user space. This is the minimal firewall baseline, not a
  full nftables clone.
- Dependencies: M12, M19, M42, M56,
  `docs/net/network_stack_contract_v1.md`.
- Contracts/Interfaces: `docs/net/firewall_hook_contract_v1.md`,
  `docs/net/net_policy_daemon_api_v1.md`,
  `docs/abi/socket_policy_extensions_v1.md`.
- Tests/Gates: `make test-firewall-v1`;
  `tests/net/test_firewall_stateful_rules_v1.py`,
  `tests/net/test_firewall_zone_policy_v1.py`,
  `tests/net/test_firewall_negative_paths_v1.py`; serial markers
  `FW: allow ok`, `FW: drop ok`.
- Hardware Tiers: Wired Tier 1 profiles must pass latency and denial-path
  budgets; Wi-Fi profiles stay Tier 2 until M64 closes roam and policy races.
- Implications: Security improves via capability-scoped network policy, but the
  kernel must enforce bounded state tables and deterministic rule ordering to
  avoid DoS regressions.

#### Milestone ID: M63 - VPN Tunnel Primitives v1

- Description: Add WireGuard-class encrypted tunnel primitives with key
  rotation, timer handling, roaming-safe endpoints, and kill-switch hooks.
  Kernel work stays limited to packet/crypto/session primitives; peer policy
  and config orchestration remain in Go.
- Dependencies: M62, M19, M28.
- Contracts/Interfaces: `docs/net/wireguard_primitives_contract_v1.md`,
  `docs/net/tunnel_socket_abi_v1.md`,
  `docs/security/vpn_key_rotation_policy_v1.md`.
- Tests/Gates: `make test-vpn-baseline-v1`;
  `tests/net/test_wireguard_handshake_v1.py`,
  `tests/net/test_wireguard_rekey_v1.py`,
  `tests/net/test_vpn_killswitch_v1.py`; serial markers
  `VPN: handshake ok`, `VPN: rekey ok`.
- Hardware Tiers: Tier 1 on wired hosts; Tier 2 on audited Wi-Fi adapters until
  roam and power-save interactions are verified.
- Implications: Remote-access security improves materially, but determinism now
  depends on pinned cryptographic test vectors, timer granularity, and endpoint
  mobility handling.

#### Milestone ID: M64 - Wireless Control Plane + Roaming v1

- Description: Build the user-space wireless manager boundary with scan, select,
  roam, WPA2/WPA3 handoff, regulatory updates, and power-save transitions. The
  kernel exposes only 802.11 control and event primitives; policy lives in Go.
- Dependencies: M56, M62-M63.
- Contracts/Interfaces: `docs/net/wifi_control_plane_contract_v2.md`,
  `docs/net/roaming_policy_v1.md`,
  `docs/abi/wireless_socket_extensions_v1.md`.
- Tests/Gates: `make test-wireless-stack-v1`;
  `tests/net/test_wifi_scan_roam_v1.py`,
  `tests/net/test_wpa3_handshake_v1.py`,
  `tests/net/test_wifi_loss_recovery_v1.py`; serial markers
  `WIFI: roam ok`, `WIFI: recover ok`.
- Hardware Tiers: Audited Wi-Fi adapters can move from Tier 2 to Tier 1
  candidate only after deterministic roam, reconnect, and power-save campaigns
  stay green.
- Implications: Laptop-class connectivity improves, but firmware provenance,
  regulatory updates, and network-manager privilege boundaries become part of
  the security model.

#### Milestone ID: M65 - Routing, NAT, and Traffic Control v1

- Description: Add forwarding, NAT state, DHCP/DNS service baseline, and
  traffic-shaping/qdisc primitives for router and edge-node use. Policy and
  orchestration stay in Go services; the kernel owns only deterministic data
  plane hooks and counters.
- Dependencies: M62-M64, M19.
- Contracts/Interfaces: `docs/net/routing_dataplane_contract_v1.md`,
  `docs/net/traffic_control_contract_v1.md`,
  `docs/abi/network_flow_stats_v1.md`.
- Tests/Gates: `make test-network-stack-v3`;
  `tests/net/test_nat_forwarding_v1.py`,
  `tests/net/test_traffic_shaping_v1.py`,
  `tests/net/test_lossy_link_service_recovery_v1.py`; serial markers
  `ROUTE: forward ok`, `QDISC: rate ok`.
- Hardware Tiers: Tier 1 on dual-NIC wired profiles; mixed wired/Wi-Fi gateway
  profiles remain Tier 2 until M64 hardware promotions are complete.
- Implications: Rugo moves toward edge and appliance use, but packet-loss
  simulation, queue timing, and service startup order must stay seeded and
  replayable to avoid flaky gates.

### User Interface and Desktop Environment

#### Milestone ID: M66 - Accessibility + Assistive Hooks v1

- Description: Add accessibility tree export, keyboard-only navigation,
  high-contrast themes, reduced-motion policy, focus narration events, and
  magnifier hooks. Keep the baseline lightweight and event-driven rather than
  bundling a heavyweight accessibility suite.
- Dependencies: M48-M52, M49, M51.
- Contracts/Interfaces: `docs/desktop/accessibility_contract_v1.md`,
  `docs/desktop/assistive_event_api_v1.md`,
  `docs/desktop/focus_semantics_v2.md`.
- Tests/Gates: `make test-accessibility-v1`;
  `tests/desktop/test_screen_reader_hooks_v1.py`,
  `tests/desktop/test_keyboard_navigation_v1.py`,
  `tests/desktop/test_high_contrast_policy_v1.py`; serial markers
  `A11Y: tree ok`, `A11Y: nav ok`.
- Hardware Tiers: Tier 1 desktop profiles on single-monitor USB input paths;
  multi-monitor validation starts as Tier 2 shadow coverage.
- Implications: Usability and inclusion improve immediately, but accessibility
  events must stay machine-readable and deterministic to avoid visual-only
  release criteria.

#### Milestone ID: M67 - File Manager + Content Workflows v1

- Description: Ship a Go-based file explorer with copy/move/delete/trash,
  removable-media workflows, MIME/content handlers, and bounded file-open/save
  integrations for the existing app shell.
- Dependencies: M52, M58, M61, M46.
- Contracts/Interfaces: `docs/desktop/file_manager_contract_v1.md`,
  `docs/pkg/content_handler_contract_v1.md`,
  `docs/storage/removable_media_ui_policy_v1.md`.
- Tests/Gates: `make test-file-manager-v1`;
  `tests/desktop/test_file_manager_workflows_v1.py`,
  `tests/desktop/test_mime_launch_v1.py`,
  `tests/desktop/test_removable_media_ui_v1.py`; serial markers
  `FM: copy ok`, `FM: mount ok`.
- Hardware Tiers: Tier 1 desktop and removable-media profiles; Tier 2 for
  large-volume NVMe and multi-monitor drag/drop coverage.
- Implications: Daily-use desktop value rises, but automount and content-launch
  paths must remain capability-gated to avoid privilege escalation.

#### Milestone ID: M68 - Settings, Notifications, and Background UX v1

- Description: Add a unified settings app, notification center, network/power/
  display panels, and bounded background-service prompts so users can manage
  the system without falling back to shell-only workflows.
- Dependencies: M52, M56, M65, M66.
- Contracts/Interfaces: `docs/desktop/settings_panel_contract_v1.md`,
  `docs/desktop/notification_center_contract_v1.md`,
  `docs/runtime/background_service_ux_policy_v1.md`.
- Tests/Gates: `make test-desktop-services-v1`;
  `tests/desktop/test_settings_panels_v1.py`,
  `tests/desktop/test_notification_center_v1.py`,
  `tests/desktop/test_background_service_prompts_v1.py`; serial markers
  `SHELL: notify ok`, `SHELL: panel ok`.
- Hardware Tiers: Tier 1 desktop profiles with wired/Wi-Fi and basic power
  states; Tier 2 for laptop battery and suspend/resume UI integration.
- Implications: The OS becomes operable for non-shell users, but permission and
  service prompts must stay explicit to preserve the capability model.

#### Milestone ID: M69 - Multi-Monitor + HiDPI Workspace v1

- Description: Extend the compositor and shell for multi-head layout, mirrored
  and extended modes, HiDPI scaling, per-display profiles, and deterministic
  hotplug behavior across virtio and native GPU paths.
- Dependencies: M55, M66-M68.
- Contracts/Interfaces: `docs/desktop/multi_monitor_contract_v1.md`,
  `docs/desktop/hidpi_scaling_policy_v1.md`,
  `docs/desktop/display_profile_contract_v1.md`.
- Tests/Gates: `make test-multi-monitor-v1`;
  `tests/desktop/test_multi_monitor_layout_v1.py`,
  `tests/desktop/test_hidpi_scaling_v1.py`,
  `tests/desktop/test_gpu_hotplug_display_v1.py`; serial markers
  `DISPLAY: head2 ok`, `DISPLAY: scale ok`.
- Hardware Tiers: Virtio-GPU remains Tier 1; native GPU multi-head support is
  Tier 2 until hotplug, fallback, and crash-recovery audits are complete.
- Implications: Workstation usability improves, but visual assertions must use
  fixed framebuffer capture and pinned DPI fixtures to stay reproducible.

#### Milestone ID: M70 - Desktop Productivity Workflow v1

- Description: Add app search/launcher, clipboard and drag-drop, session
  restore, and bounded productivity workflows that make the shell feel like a
  usable desktop without claiming full office/browser parity.
- Dependencies: M66-M69, M39, M44.
- Contracts/Interfaces: `docs/desktop/productivity_workflow_profile_v1.md`,
  `docs/desktop/clipboard_dragdrop_contract_v1.md`,
  `docs/desktop/session_restore_policy_v1.md`.
- Tests/Gates: `make test-desktop-productivity-v1`;
  `tests/desktop/test_app_search_launch_v1.py`,
  `tests/desktop/test_clipboard_dragdrop_v1.py`,
  `tests/desktop/test_session_restore_v1.py`; serial markers
  `DESKTOP: launch ok`, `DESKTOP: restore ok`.
- Hardware Tiers: Tier 1 desktops; Tier 2 for GPU-accelerated multi-monitor
  restore and removable-media productivity flows.
- Implications: End-user adoption improves, but UI timing and persistence logic
  must be captured through deterministic artifacts, not human-only validation.

### Application Ecosystem and Packaging

#### Milestone ID: M71 - Package Manager + Dependency Solver v1

- Description: Add a first-class Go package manager with dependency solving,
  transaction rollback, repo pinning, delta download support, and content-
  addressed local cache management.
- Dependencies: M26, M39, M70.
- Contracts/Interfaces: `docs/pkg/package_manager_contract_v1.md`,
  `docs/pkg/dependency_solver_policy_v1.md`,
  `docs/pkg/repo_metadata_v4.md`.
- Tests/Gates: `make test-pkg-manager-v1`;
  `tests/pkg/test_dependency_solver_v1.py`,
  `tests/pkg/test_pkg_transaction_rollback_v1.py`,
  `tests/pkg/test_repo_pinning_v1.py`; serial markers
  `PKG: solve ok`, `PKG: rollback ok`.
- Hardware Tiers: N/A; cross-tier feature with desktop and server coverage.
- Implications: Community porting accelerates, but reproducible vendoring and
  signed metadata remain non-negotiable for trust and repeatable builds.

#### Milestone ID: M72 - Sandboxed App Bundles + Permissions v1

- Description: Introduce installable app bundles with declared permissions,
  per-app capability grants, desktop portals, and deterministic denial behavior
  so third-party applications can be distributed without full system trust.
- Dependencies: M42, M71, M70.
- Contracts/Interfaces: `docs/pkg/app_bundle_contract_v1.md`,
  `docs/security/app_permission_policy_v1.md`,
  `docs/desktop/portal_api_contract_v1.md`.
- Tests/Gates: `make test-app-bundles-v1`;
  `tests/pkg/test_app_bundle_manifest_v1.py`,
  `tests/security/test_app_permission_prompts_v1.py`,
  `tests/desktop/test_portal_file_dialog_v1.py`; serial markers
  `APP: install ok`, `APP: denied capability`.
- Hardware Tiers: Cross-tier; desktop portal paths validated on Tier 1 GUI
  profiles only.
- Implications: Safer app distribution becomes possible, but permission prompts
  and portal behavior must be deterministic and auditable, not policy-by-UI.

#### Milestone ID: M73 - Developer SDK + Porting Kit v1

- Description: Publish headers, templates, toolchains, CI scaffolds, and
  porting guides for Go, Rust, and TinyGo apps so external developers can build
  native software against Rugo's bounded ABI and desktop profiles.
- Dependencies: M11, M36, M41, M71-M72.
- Contracts/Interfaces: `docs/pkg/sdk_contract_v1.md`,
  `docs/abi/app_porting_profile_v1.md`,
  `docs/build/sdk_repro_policy_v1.md`.
- Tests/Gates: `make test-sdk-porting-v1`;
  `tests/pkg/test_sdk_template_builds_v1.py`,
  `tests/compat/test_app_porting_profile_v1.py`,
  `tests/build/test_sdk_reproducibility_v1.py`; serial markers
  `SDK: template ok`, `SDK: repro ok`.
- Hardware Tiers: Cross-tier; TinyGo bare-metal utility templates remain bound
  to Tier 0/Tier 1 reference hardware.
- Implications: Developer adoption improves sharply, but SDK manifests and
  toolchain versions must stay locked to prevent silent drift.

#### Milestone ID: M74 - Catalog Federation + Build Farm v1

- Description: Add a federated app catalog, community package ingestion,
  reproducible build-farm attestations, moderation/quarantine workflow, and
  health scoring so the ecosystem can grow without losing provenance.
- Dependencies: M31, M39, M40, M71-M73.
- Contracts/Interfaces: `docs/pkg/catalog_federation_policy_v1.md`,
  `docs/pkg/build_farm_attestation_v1.md`,
  `docs/pkg/moderation_workflow_v1.md`.
- Tests/Gates: `make test-app-catalog-health-v2`;
  `tests/pkg/test_catalog_federation_v1.py`,
  `tests/pkg/test_build_farm_attestation_v1.py`,
  `tests/pkg/test_catalog_quarantine_v1.py`; serial markers
  `CAT: ingest ok`, `CAT: attest ok`.
- Hardware Tiers: N/A.
- Implications: Rugo gains a real community distribution story, but moderation,
  storage cost, and provenance verification become ongoing operational work.

### Security, Isolation, and Compliance

#### Milestone ID: M75 - Trusted Update Orchestrator v1

- Description: Build a secure update agent with staged rollout policy, rollback
  guardrails, maintenance windows, snapshot-aware recovery, and mandatory
  signature/monotonic-version enforcement.
- Dependencies: M14, M31, M61, M71,
  `docs/security/rights_capability_model_v1.md`.
- Contracts/Interfaces: `docs/security/update_orchestrator_policy_v1.md`,
  `docs/pkg/update_rollout_contract_v1.md`,
  `docs/runtime/reboot_coordinator_contract_v1.md`.
- Tests/Gates: `make test-security-hardening-v4`;
  `tests/security/test_update_orchestrator_v1.py`,
  `tests/pkg/test_staged_rollout_policy_v1.py`,
  `tests/security/test_rollback_guardrails_v1.py`; serial markers
  `UPDATE: stage ok`, `UPDATE: rollback blocked`.
- Hardware Tiers: Tier 1 can consume signed manual updates; audited Tier 2
  secure-boot systems are the first candidates for unattended update claims.
- Implications: Enterprise readiness improves, but signing-key hygiene and
  rollback-safety semantics become release-critical parts of the threat model.

#### Milestone ID: M76 - Compliance Profiles + Audit Evidence v1

- Description: Add formal compliance profiles (baseline, hardened, regulated)
  with mapped technical controls, retention/redaction rules, exportable audit
  bundles, and release checklists tied to actual runtime evidence.
- Dependencies: M28, M40, M75,
  `docs/security/rights_capability_model_v1.md`.
- Contracts/Interfaces: `docs/security/compliance_profile_v1.md`,
  `docs/security/audit_evidence_contract_v1.md`,
  `docs/build/compliance_release_checklist_v1.md`.
- Tests/Gates: `make test-compliance-profile-v1`;
  `tests/security/test_compliance_control_mapping_v1.py`,
  `tests/security/test_audit_bundle_redaction_v1.py`,
  `tests/build/test_compliance_release_gate_v1.py`; serial markers
  `COMP: profile ok`, `COMP: redaction ok`.
- Hardware Tiers: Tier 1 can claim baseline compliance evidence; audited Tier 2
  secure-boot/TPM platforms can claim managed-profile evidence.
- Implications: Auditability increases, but process and documentation burden
  rises. Reproducibility now covers evidence-bundle schemas and retention rules.

#### Milestone ID: M77 - Secrets, Attestation, and Fleet Admission v1

- Description: Add device identity, secret sealing, attestation-backed policy,
  and fleet admission controls so only trusted systems can enroll, receive
  packages, or join managed update rings.
- Dependencies: M23, M33, M75-M76.
- Contracts/Interfaces: `docs/security/attested_identity_contract_v1.md`,
  `docs/security/secret_sealing_policy_v1.md`,
  `docs/pkg/fleet_admission_contract_v1.md`.
- Tests/Gates: `make test-fleet-admission-v1`;
  `tests/security/test_secret_sealing_v1.py`,
  `tests/security/test_attestation_admission_v1.py`,
  `tests/pkg/test_fleet_policy_denial_v1.py`; serial markers
  `ATTEST: admit ok`, `SECRET: unseal ok`.
- Hardware Tiers: Attested admission claims are Tier 2 audited-only; QEMU
  remains simulation-only and cannot qualify for real fleet trust claims.
- Implications: Fleet security strengthens substantially, but certificate
  lifecycle and hardware root-of-trust diversity become material risks.

### Performance, Stability, and Observability

#### Milestone ID: M78 - Scheduler Latency + CPU Affinity v1

- Description: Raise the scheduler from a basic fairness baseline to explicit
  latency budgets, CPU affinity, priority classes, and bounded real-time style
  behavior for interactive and service workloads on SMP systems.
- Dependencies: M16, M22, M24, M43.
- Contracts/Interfaces: `docs/runtime/scheduler_latency_contract_v1.md`,
  `docs/runtime/cpu_affinity_policy_v1.md`,
  `docs/abi/thread_priority_contract_v1.md`.
- Tests/Gates: `make test-process-scheduler-v3`;
  `tests/sched/test_priority_preemption_v1.py`,
  `tests/sched/test_cpu_affinity_v1.py`,
  `tests/runtime/test_scheduler_latency_budget_v1.py`; serial markers
  `SCHED: rt ok`, `SCHED: affinity ok`.
- Hardware Tiers: Multi-core Tier 1 and Tier 2 systems are in scope; single-
  core QEMU profiles stay as a regression floor, not the performance target.
- Implications: Performance closes toward the legacy C baseline and Linux RT-
  style responsiveness, but starvation and inversion bugs become more likely.

#### Milestone ID: M79 - Memory Pressure + I/O QoS v1

- Description: Add reclaim policies, pressure accounting, OOM selection,
  storage/network I/O priorities, and isolation-aware resource backpressure so
  the system stays stable under mixed interactive and service load.
- Dependencies: M18, M38, M42, M78.
- Contracts/Interfaces: `docs/runtime/memory_pressure_contract_v1.md`,
  `docs/storage/io_qos_policy_v1.md`,
  `docs/abi/resource_pressure_events_v1.md`.
- Tests/Gates: `make test-memory-pressure-v1`;
  `tests/runtime/test_memory_reclaim_v1.py`,
  `tests/storage/test_io_qos_v1.py`,
  `tests/runtime/test_oom_policy_v1.py`; serial markers
  `MEM: reclaim ok`, `QOS: io ok`.
- Hardware Tiers: Tier 1 multi-core + NVMe profiles; Tier 2 for large-memory
  hosts and mixed service/Desktop pressure campaigns.
- Implications: Stability under load improves, but no_std allocator behavior
  and reclaim heuristics can easily undermine determinism if budgets are not
  fixed and exposed.

#### Milestone ID: M80 - Observability Pipeline + Perf Telemetry v1

- Description: Add stable metrics, tracepoints, perf counters, and baseline
  diff tooling so runtime regressions can be measured automatically against both
  prior Rugo releases and the legacy C reference behavior.
- Dependencies: M29, M40, M78-M79.
- Contracts/Interfaces: `docs/runtime/telemetry_contract_v1.md`,
  `docs/runtime/perf_counter_abi_v1.md`,
  `docs/build/perf_baseline_policy_v1.md`.
- Tests/Gates: `make test-observability-v3`;
  `tests/runtime/test_metric_registry_v1.py`,
  `tests/runtime/test_perf_counter_abi_v1.py`,
  `tests/runtime/test_legacy_baseline_diff_v1.py`; serial markers
  `OBS: metric ok`, `OBS: perf diff ok`.
- Hardware Tiers: Cross-tier; multi-core Tier 1/Tier 2 machines are used for
  benchmark and regression budgets, while Tier 0 keeps the schema floor green.
- Implications: Regression triage becomes much faster, but metric sprawl and
  privacy/redaction rules need explicit governance.

#### Milestone ID: M81 - Reliability Fuzzing + Chaos Qualification v1

- Description: Add deterministic chaos campaigns, syscall/storage/network/UI
  fuzzing, long-run soak classification, and auto-bisect hooks so reliability
  becomes a measured property rather than an anecdotal one.
- Dependencies: M22, M28, M80.
- Contracts/Interfaces: `docs/runtime/chaos_qualification_policy_v1.md`,
  `docs/security/fuzz_coverage_contract_v1.md`,
  `docs/build/reliability_slo_v1.md`.
- Tests/Gates: `make test-kernel-reliability-v2`;
  `tests/runtime/test_chaos_campaign_v1.py`,
  `tests/security/test_fuzz_coverage_budget_v1.py`,
  `tests/runtime/test_reliability_slo_v1.py`; serial markers
  `CHAOS: pass seed=20260311`, `FUZZ: budget ok`.
- Hardware Tiers: Tier 1 becomes the release-blocking stress floor; audited
  Tier 2 stress profiles can promote once seed-stable campaigns stay green.
- Implications: Stability improves and hidden bugs surface earlier, but CI
  runtime and flaky-seed management will grow unless campaign sizes stay capped.

### Community, Documentation, and Release Engineering

#### Milestone ID: M82 - Contributor Portal + Governance v1

- Description: Establish a contributor portal, RFC process, ownership policy,
  moderation workflow, and newcomer path so roadmap execution can scale beyond
  a small core maintainer group.
- Dependencies: M31, M74.
- Contracts/Interfaces: `docs/community/contribution_policy_v1.md`,
  `docs/community/rfc_process_v1.md`,
  `docs/community/code_of_conduct_enforcement_v1.md`.
- Tests/Gates: `make test-docs-governance-v1`;
  `tests/docs/test_contribution_docs_v1.py`,
  `tests/docs/test_rfc_process_v1.py`.
- Hardware Tiers: N/A.
- Implications: Open-source collaboration becomes tractable, but governance and
  moderation become explicit maintenance costs rather than implicit side work.

#### Milestone ID: M83 - Localization + Docs Quality Pipeline v1

- Description: Add localization workflow for installer, desktop shell, and core
  docs, starting with an explicit French-Canadian baseline plus glossary,
  screenshot regeneration, and docs-lint enforcement.
- Dependencies: M52, M66-M70, M82.
- Contracts/Interfaces: `docs/community/localization_policy_v1.md`,
  `docs/build/docs_release_contract_v1.md`,
  `docs/desktop/string_catalog_contract_v1.md`.
- Tests/Gates: `make test-doc-quality-v1`;
  `tests/docs/test_localization_catalog_v1.py`,
  `tests/docs/test_docs_release_contract_v1.py`,
  `tests/desktop/test_string_catalog_consistency_v1.py`.
- Hardware Tiers: N/A.
- Implications: Broader adoption and clearer docs follow, especially for Quebec
  and bilingual users, but translation drift can easily block releases if
  catalog and screenshot schemas are not enforced automatically.

#### Milestone ID: M84 - Community Release Train + Support Channels v1

- Description: Add a public release calendar, support-channel SLAs, forum/
  discussion operations, public advisory workflow, and maintainer handoff
  runbooks so releases become predictable and sustainable.
- Dependencies: M31, M75-M76, M82-M83.
- Contracts/Interfaces: `docs/build/community_release_train_v1.md`,
  `docs/community/support_channel_sla_v1.md`,
  `docs/security/public_advisory_workflow_v1.md`.
- Tests/Gates: `make test-release-community-v1`;
  `tests/build/test_release_calendar_contract_v1.py`,
  `tests/docs/test_support_sla_docs_v1.py`,
  `tests/security/test_public_advisory_workflow_v1.py`.
- Hardware Tiers: N/A.
- Implications: Predictable releases and community trust improve, but support
  promises, advisory handling, and public maintenance windows become durable
  operational commitments.

## Research references (primary sources)

Compatibility and ABI:
- POSIX Issue 8 (Open Group online publications): https://pubs.opengroup.org/onlinepubs/9799919799/
- POSIX Issue 8 download index: https://pubs.opengroup.org/onlinepubs/9799919799/download/index.html
- FreeBSD Linux Binary Compatibility chapter: https://docs.freebsd.org/en/books/handbook/linuxemu/
- NetBSD binary emulation overview: https://www.netbsd.org/Documentation/compat.html
- QEMU user-mode emulation: https://www.qemu.org/docs/master/user/main.html

Hardware and platform:
- Virtio 1.2 spec: https://docs.oasis-open.org/virtio/virtio/v1.2/virtio-v1.2.html
- Linux PCI driver model guide: https://www.kernel.org/doc/html/next/PCI/pci.html
- UEFI 2.10 spec: https://uefi.org/specs/UEFI/2.10/
- ACPI 6.5 spec: https://uefi.org/specs/ACPI/6.5/index.html

Security:
- NIST SP 800-193 (platform firmware resiliency): https://csrc.nist.gov/pubs/sp/800/193/final
- NIST SP 800-218 (SSDF): https://csrc.nist.gov/publications/detail/sp/800-218/final
- secL4 capabilities tutorial: https://docs.sel4.systems/Tutorials/capabilities.html
- seL4 white paper: https://sel4.systems/About/whitepaper.html
- Linux seccomp filter docs: https://www.kernel.org/doc/html/latest/userspace-api/seccomp_filter.html
- Linux Landlock docs: https://www.kernel.org/doc/html/latest/userspace-api/landlock.html
- syzkaller project: https://github.com/google/syzkaller
- KCOV docs: https://docs.kernel.org/next/dev-tools/kcov.html

Runtime and toolchain:
- Go porting policy: https://go.dev/wiki/PortingPolicy
- Go minimum requirements: https://go.dev/wiki/MinimumRequirements
- musl project overview: https://musl.libc.org/

Networking:
- TCP (RFC 9293): https://www.rfc-editor.org/info/rfc9293
- Host requirements (RFC 1122): https://www.rfc-editor.org/info/rfc1122
- UDP (RFC 768): https://www.rfc-editor.org/info/rfc768
- IPv4 (RFC 791): https://www.rfc-editor.org/info/rfc791
- ARP (RFC 826): https://www.rfc-editor.org/info/rfc826
- IPv6 (RFC 8200): https://www.rfc-editor.org/info/rfc8200
- IPv6 Neighbor Discovery (RFC 4861): https://www.rfc-editor.org/info/rfc4861

Filesystem and storage:
- ext4 journal design (jbd2): https://docs.kernel.org/filesystems/ext4/journal.html
- ext4 admin behavior notes: https://www.kernel.org/doc/html/latest/admin-guide/ext4.html
- btrfs docs: https://docs.kernel.org/filesystems/btrfs.html
- Linux VFS overview: https://docs.kernel.org/filesystems/vfs.html

Productization and supply chain:
- TUF specification: https://theupdateframework.github.io/specification/
- SLSA spec v1.2: https://slsa.dev/spec/v1.2/
- SPDX specification 3.0.1: https://spdx.github.io/spdx-spec/v3.0.1/
- SOURCE_DATE_EPOCH specification: https://reproducible-builds.org/specs/source-date-epoch/
