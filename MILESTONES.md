# Rugo Milestones

Date: 2026-02-17
Target: x86-64, tiered hardware support model (QEMU Tier 0/Tier 1 release floor), Limine boot protocol

---

## How to read this document

Treat this file as the exhaustive completion ledger. The architecture-first
repo story now lives in `README.md`, `docs/architecture/README.md`, and
`docs/roadmap/README.md`.

The default product direction is:

- Rust `no_std` kernel for mechanisms
- Go user space for services and higher-level policy
- TinyGo-first integration path for the default demo lane
- stock-Go bring-up kept as experimental until the repo structure is migrated
- legacy C lane preserved as a reference baseline, not as an equal-status lane

Hardware support is still tracked through a tiered matrix. QEMU Tier 0/Tier 1
remain the release-blocking floor, while broader bare-metal classes advance
only through explicit promotion and audit policy.

| Lane | Language | Location | Purpose |
|------|----------|----------|---------|
| **Legacy** | C + Go (gccgo) | `legacy/` | Completed reference baseline (M0-M7 + G0). Preserved for comparison and regression context. |
| **Rugo** | Rust (no_std) + Go (TinyGo-first) | repo root | Default hybrid OS lane. The production rewrite and the active repo story. |
| **Experimental** | Rust + stock-Go bridge work | `services/go_std/`, `tools/gostd_stock_builder/` | Bring-up and contract experiments for the stock-Go port. |

**Legacy is not Rugo.** Legacy proved out the early milestone design. Rugo is
the production lane. The stock-Go lane is also not the default user-space path;
it remains an experiment until the repo structure is explicitly migrated around
it.

Milestones M0-M7 define kernel functionality (boot, memory, scheduling, user
mode, IPC, drivers, filesystem, networking). Go milestones define user-space
integration: G0 is legacy-only (gccgo kernel entry), G1 is the TinyGo-first
user-space lane, and G2 is the stock-Go contract path.

---

## Repo lanes and how to run

### Rugo (default)

```bash
# Recommended quick paths
make demo-go      # Rust kernel + TinyGo user-space demo
make run-kernel   # kernel-only boot path
make validate     # compatibility alias for make test-qemu

# Full native flow (requires nightly Rust, nasm, ld, xorriso, qemu, pytest)
make build        # Compile Rust kernel → out/kernel.elf
make image        # Build bootable ISO → out/os.iso
make image-panic  # Build panic-test ISO → out/os-panic.iso
make image-go     # Build TinyGo user-space ISO -> out/os-go.iso
make image-go-std # Build stock-Go experimental ISO -> out/os-go-std.iso
make test-qemu    # Build all Rugo test images, run pytest tests/

# Docker (cross-platform, no host toolchain needed)
make docker-all
```

Primary source map:
- kernel mechanisms: `arch/`, `boot/`, `kernel_rs/`
- default Go user space: `services/go/`
- experimental stock-Go lane: `services/go_std/`
- tooling and validation: `tools/`, `tests/`, `.github/`, `docs/`

Tests: `tests/` (boot, trap, sched, user, ipc, drivers, fs, pkg, net, go,
stress, quota, pressure)

### Legacy (reference only)

```bash
# From repo root (requires nasm, gcc, gccgo, ld, objcopy, xorriso, qemu, pytest)
make -C legacy build
make -C legacy image
make -C legacy test-qemu   # 16 tests
```

Tests: `legacy/tests/` (boot, trap, sched, user, ipc, drivers, fs, pkg, net)

**Note:** Both lanes output to `out/`. Building one lane overwrites the other's
`kernel.elf` and `os.iso`.

Current structure and migration plan:
- architecture overview: `docs/architecture/README.md`
- repo strategy and target layout: `docs/architecture/repo-strategy.md`
- milestone streams and roadmap summary: `docs/roadmap/README.md`

---

## Milestone status matrix

| # | Milestone | Legacy | Rugo | Evidence |
|---|-----------|--------|------|----------|
| M0 | Boot + serial | ✅ | ✅ | Legacy: `legacy/tests/boot/test_boot_banner.py` (KERNEL markers). Rugo: `tests/boot/test_boot_banner.py` + `tests/boot/test_panic_path.py` (RUGO markers). |
| M1 | Paging + traps | ✅ | ✅ | Legacy: `legacy/tests/boot/test_paging_enabled.py`, `legacy/tests/trap/test_page_fault_report.py`. Rugo: `tests/boot/test_paging_enabled.py`, `tests/trap/test_page_fault_report.py`, `tests/trap/test_idt_smoke.py`. |
| M2 | Scheduler + threads | ✅ | ✅ | Legacy: `legacy/tests/sched/test_timer_ticks.py`, `legacy/tests/sched/test_two_threads.py`. Rugo: `tests/sched/test_timer_ticks.py`, `tests/sched/test_two_threads.py`. |
| M3 | User mode + syscalls | ✅ | ✅ | Legacy: `legacy/tests/user/test_enter_user_mode.py`, `test_syscall_roundtrip.py`, `test_user_fault.py`. Rugo: `tests/user/test_enter_user_mode.py`, `tests/user/test_syscall_roundtrip.py`, `tests/user/test_thread_exit.py`, `tests/user/test_user_fault.py`. |
| M4 | IPC + shared memory | ✅ | ✅ | Legacy: `legacy/tests/ipc/test_ping_pong.py`, `legacy/tests/ipc/test_shm_bulk.py`. Rugo: `tests/ipc/test_ping_pong.py`, `tests/ipc/test_shm_bulk.py`. |
| M5 | VirtIO block + syscalls | ✅ | ✅ | Legacy: `legacy/tests/drivers/test_virtio_blk_identify.py`, `test_virtio_blk_rw.py`. Rugo: `tests/drivers/test_virtio_blk_identify.py`, `tests/drivers/test_virtio_blk_rw.py`. |
| M6 | Filesystem + pkg + shell | ✅ | ✅ | Legacy: `legacy/tests/fs/test_fsd_smoke.py`, `legacy/tests/pkg/test_pkg_install_run.py`. Rugo: `tests/fs/test_fsd_smoke.py`, `tests/pkg/test_pkg_install_run.py`. |
| M7 | VirtIO net + UDP echo | ✅ | ✅ | Legacy: `legacy/tests/net/test_udp_echo.py`. Rugo: `tests/net/test_udp_echo.py` (`NET: udp echo`). |
| G0 | Go kernel entry (gccgo) | ✅ | n/a | Legacy: `legacy/tests/boot/test_go_entry.py`. Legacy-only milestone. |
| G1 | Go services (TinyGo) | n/a | ✅ | Rugo: `tests/go/test_go_user_service.py` (`GOUSR: ok`). TinyGo bare-metal x86_64. |
| G2 | Full Go port | n/a | ✅ | Rugo-only milestone. Done (`tests/go/test_std_go_binary.py`, stock-Go artifact contract path). |
| M8 | Compatibility Profile v1 | n/a | ✅ | Rugo: `tests/compat/*`, `tests/pkg/test_pkg_external_apps.py`; docs in `docs/abi/*` and `docs/M8_EXECUTION_BACKLOG.md`. |
| M9 | Hardware enablement matrix v1 | n/a | ✅ | Rugo: `tests/hw/*`, CI hardware gate, and docs in `docs/hw/*`, `docs/M9_EXECUTION_BACKLOG.md`. |
| M10 | Security baseline v1 | n/a | ✅ | Rugo: `tests/security/*`, security CI gate, and docs in `docs/security/*`, `docs/M10_EXECUTION_BACKLOG.md`. |
| M11 | Runtime + toolchain maturity v1 | n/a | ✅ | Rugo: `tests/runtime/*`, `make test-runtime-maturity`, CI runtime gate, docs in `docs/runtime/*`, `docs/M11_EXECUTION_BACKLOG.md`. |
| M12 | Network stack v1 | n/a | ✅ | Rugo: `tests/net/*`, `make test-network-stack-v1`, CI network gate, docs in `docs/net/*`, `docs/M12_EXECUTION_BACKLOG.md`. |
| M13 | Storage reliability v1 | n/a | ✅ | Rugo: `tests/storage/*`, `make test-storage-reliability-v1`, CI storage gate, docs in `docs/storage/*`, `docs/M13_EXECUTION_BACKLOG.md`. |
| M14 | Productization + release engineering v1 | n/a | done | Rugo: `tests/build/*`, update attack/metadata tests in `tests/pkg/*`, `make test-release-engineering-v1`, CI release-engineering gate, docs in `docs/build/*`, `docs/pkg/*`, `docs/M14_EXECUTION_BACKLOG.md`. |
| M15 | Hardware Enablement Matrix v2 | n/a | done | Rugo: `tests/hw/*_v2`, `make test-hw-matrix-v2`, CI `Hardware matrix v2 gate`, docs in `docs/hw/*_v2`, `docs/M15_EXECUTION_BACKLOG.md`. |
| M16 | Process + Scheduler Model v2 | n/a | done | Rugo: `tests/sched/*_v2`, `tests/user/*_v2`, `make test-process-scheduler-v2`, CI `Process scheduler v2 gate`, docs in `docs/abi/*_v2`, `docs/M16_EXECUTION_BACKLOG.md`. |
| M17 | Compatibility Profile v2 | n/a | done | Rugo: `tests/compat/*_v2` + tier gate tests, `make test-compat-v2`, CI `Compatibility profile v2 gate`, docs in `docs/abi/*_v2`, `docs/runtime/syscall_coverage_matrix_v2.md`, `docs/M17_EXECUTION_BACKLOG.md`. |
| M18 | Storage Reliability v2 | n/a | done | Rugo: `tests/storage/*_v2` + storage gate tests, `make test-storage-reliability-v2`, CI `Storage reliability v2 gate`, docs in `docs/storage/*_v2`, `docs/M18_EXECUTION_BACKLOG.md`. |
| M19 | Network Stack v2 | n/a | done | Rugo: `tests/net/*_v2` + network gate tests, `make test-network-stack-v2`, CI `Network stack v2 gate`, docs in `docs/net/*_v2`, `docs/M19_EXECUTION_BACKLOG.md`. |
| M20 | Operability + Release UX v2 | n/a | done | Rugo: `tests/build/*_v2` + operability gate tests, `make test-release-ops-v2`, CI `Operability and release UX v2 gate`, docs in `docs/build/*_v2`, `docs/pkg/*_v2`, `docs/M20_EXECUTION_BACKLOG.md`. |
| M21 | ABI + API Stability Program v3 | n/a | done | Rugo: ABI/API stability docs and compatibility enforcement tests, `make test-abi-stability-v3`, CI `ABI stability v3 gate`, docs in `docs/abi/syscall_v3.md`, `docs/runtime/*`, and `docs/M21_EXECUTION_BACKLOG.md`. |
| M22 | Kernel Reliability + Soak v1 | n/a | done | Rugo: reliability model docs and deterministic soak/fault artifacts, `make test-kernel-reliability-v1`, CI `Kernel reliability v1 gate`, docs in `docs/runtime/kernel_reliability_model_v1.md`, and `docs/M22_EXECUTION_BACKLOG.md`. |
| M23 | Hardware Enablement Matrix v3 | n/a | done | Rugo: matrix v3 docs and deterministic diagnostics/attestation artifacts, `make test-hw-matrix-v3`, `make test-firmware-attestation-v1`, CI `Hardware matrix v3 gate` + `Firmware attestation v1 gate`, docs in `docs/hw/*_v3`, `docs/security/measured_boot_attestation_v1.md`, and `docs/M23_EXECUTION_BACKLOG.md`. |
| M24 | Performance Baseline + Regression Budgets v1 | n/a | done | Rugo: performance budget/policy docs and deterministic baseline/regression artifacts, `make test-perf-regression-v1`, CI `Performance regression v1 gate`, docs in `docs/runtime/performance_budget_v1.md`, `docs/runtime/benchmark_policy_v1.md`, and `docs/M24_EXECUTION_BACKLOG.md`. |
| M25 | Userspace Service Model + Init v2 | n/a | done | Rugo: service/init v2 contract docs and deterministic lifecycle/dependency/restart checks, `make test-userspace-model-v2`, CI `Userspace model v2 gate`, docs in `docs/runtime/service_model_v2.md`, `docs/runtime/init_contract_v2.md`, and `docs/M25_EXECUTION_BACKLOG.md`. |
| M26 | Package/Repo Ecosystem v3 | n/a | done | Rugo: package/repo v3 contracts and deterministic policy/rebuild/update-trust artifacts, `make test-pkg-ecosystem-v3`, `make test-update-trust-v1`, CI `Package ecosystem v3 gate` + `Update trust v1 gate`, docs in `docs/pkg/*_v3`, `docs/pkg/update_trust_model_v1.md`, and `docs/M26_EXECUTION_BACKLOG.md`. |
| M27 | External App Compatibility Program v3 | n/a | done | Rugo: compatibility profile/tier contracts + deterministic class matrix artifacts, `make test-app-compat-v3`, CI `App compatibility v3 gate`, docs in `docs/abi/compat_profile_v3.md`, `docs/abi/app_compat_tiers_v1.md`, and `docs/M27_EXECUTION_BACKLOG.md`. |
| M28 | Security Hardening Program v3 | n/a | done | Rugo: hardening profile/threat model contracts + deterministic attack/fuzz and vulnerability-response artifacts, `make test-security-hardening-v3`, `make test-vuln-response-v1`, CI `Security hardening v3 gate` + `Vulnerability response v1 gate`, docs in `docs/security/hardening_profile_v3.md`, `docs/security/threat_model_v2.md`, and `docs/M28_EXECUTION_BACKLOG.md`. |
| M29 | Observability + Diagnostics v2 | n/a | done | Rugo: observability/crash contracts + deterministic trace/diagnostic/crash artifacts, `make test-observability-v2`, `make test-crash-dump-v1`, CI `Observability v2 gate` + `Crash dump v1 gate`, docs in `docs/runtime/observability_contract_v2.md`, `docs/runtime/crash_dump_contract_v1.md`, and `docs/M29_EXECUTION_BACKLOG.md`. |
| M30 | Installer/Upgrade/Recovery UX v3 | n/a | done | Rugo: installer/recovery v3 contracts + deterministic upgrade/recovery rollback-safety artifacts, `make test-ops-ux-v3`, CI `Ops UX v3 gate`, docs in `docs/build/installer_ux_v3.md`, `docs/build/recovery_workflow_v3.md`, and `docs/M30_EXECUTION_BACKLOG.md`. |
| M31 | Release Engineering + Support Lifecycle v2 | n/a | done | Rugo: release/support/revalidation policy contracts + deterministic branch/support/supply-chain audits, `make test-release-lifecycle-v2`, `make test-supply-chain-revalidation-v1`, CI `Release lifecycle v2 gate` + `Supply-chain revalidation v1 gate`, docs in `docs/build/release_policy_v2.md`, `docs/build/support_lifecycle_policy_v1.md`, and `docs/M31_EXECUTION_BACKLOG.md`. |
| M32 | Conformance + Profile Qualification v1 | n/a | done | Rugo: profile conformance contract + deterministic profile qualification suite artifacts, `make test-conformance-v1`, CI `Conformance v1 gate`, docs in `docs/runtime/profile_conformance_v1.md` and `docs/M32_EXECUTION_BACKLOG.md`. |
| M33 | Fleet-Scale Operations Baseline v1 | n/a | done | Rugo: fleet update/health and rollout-safety policy contracts + deterministic fleet/canary/abort simulations, `make test-fleet-ops-v1`, `make test-fleet-rollout-safety-v1`, CI `Fleet ops v1 gate` + `Fleet rollout safety v1 gate`, docs in `docs/pkg/fleet_update_policy_v1.md`, `docs/runtime/fleet_health_policy_v1.md`, and `docs/M33_EXECUTION_BACKLOG.md`. |
| M34 | Maturity Qualification + LTS Declaration | n/a | done | Rugo: maturity qualification/LTS declaration contracts + deterministic cross-domain qualification bundle, `make test-maturity-qual-v1`, CI `Maturity qualification v1 gate`, docs in `docs/build/maturity_qualification_v1.md`, `docs/build/lts_declaration_policy_v1.md`, and `docs/M34_EXECUTION_BACKLOG.md`. |
| M35 | Desktop + Interactive UX Baseline v1 | n/a | done | Rugo: desktop/display/input contracts + deterministic desktop/gui artifacts, `make test-desktop-stack-v1`, `make test-gui-app-compat-v1`, CI `Desktop stack v1 gate` + `GUI app compatibility v1 gate`, docs in `docs/desktop/*` and `docs/M35_EXECUTION_BACKLOG.md`. |
| M36 | Compatibility Surface Expansion v1 | n/a | done | Rugo: compatibility/process/socket contracts + deterministic compatibility/POSIX artifacts, `make test-compat-surface-v1`, `make test-posix-gap-closure-v1`, CI `Compatibility surface v1 gate` + `POSIX gap closure v1 gate`, docs in `docs/abi/compat_profile_v4.md`, `docs/runtime/syscall_coverage_matrix_v3.md`, and `docs/M36_EXECUTION_BACKLOG.md`. |
| M37 | Hardware Breadth + Driver Matrix v4 | n/a | done | Rugo: matrix v4/driver-lifecycle/promotion contracts + deterministic hardware/promotion artifacts, `make test-hw-matrix-v4`, `make test-hw-baremetal-promotion-v1`, CI `Hardware matrix v4 gate` + `Hardware bare-metal promotion v1 gate`, docs in `docs/hw/support_matrix_v4.md`, `docs/hw/driver_lifecycle_contract_v4.md`, and `docs/M37_EXECUTION_BACKLOG.md`. |
| M38 | Storage + Platform Feature Expansion v1 | n/a | done | Rugo: storage/platform feature contracts + deterministic snapshot/resize/fs-ops and platform-conformance artifacts, `make test-storage-platform-v1`, `make test-storage-feature-contract-v1`, CI `Storage platform v1 gate` + `Storage feature contract v1 gate`, docs in `docs/storage/fs_feature_contract_v1.md`, `docs/runtime/platform_feature_profile_v1.md`, and `docs/M38_EXECUTION_BACKLOG.md`. |
| M39 | Ecosystem Scale + Distribution Workflow v1 | n/a | done | Rugo: ecosystem-scale/distribution contracts + deterministic catalog-scale/install/audit artifacts, `make test-ecosystem-scale-v1`, `make test-app-catalog-health-v1`, CI `Ecosystem scale v1 gate` + `App catalog health v1 gate`, docs in `docs/pkg/ecosystem_scale_policy_v1.md`, `docs/pkg/distribution_workflow_v1.md`, and `docs/M39_EXECUTION_BACKLOG.md`. |
| M40 | Runtime-Backed Evidence Integrity v1 | n/a | done | Rugo: evidence-integrity/provenance contracts + deterministic runtime-evidence/audit artifacts, `make test-evidence-integrity-v1`, `make test-synthetic-evidence-ban-v1`, CI `Evidence integrity v1 gate` + `Synthetic evidence ban v1 gate`, docs in `docs/runtime/evidence_integrity_policy_v1.md`, `docs/runtime/gate_provenance_policy_v1.md`, and `docs/M40_EXECUTION_BACKLOG.md`. |
| M41 | Process + Readiness Compatibility Closure v1 | n/a | done | Rugo: compatibility/process/readiness contracts + deterministic compatibility/POSIX artifacts, `make test-process-readiness-parity-v1`, `make test-posix-gap-closure-v2`, CI `Process readiness parity v1 gate` + `POSIX gap closure v2 gate`, docs in `docs/abi/compat_profile_v5.md`, `docs/runtime/syscall_coverage_matrix_v4.md`, and `docs/M41_EXECUTION_BACKLOG.md`. |
| M42 | Isolation + Namespace Baseline v1 | n/a | done | Rugo: isolation/namespace/resource-control contracts + deterministic containment artifacts, `make test-isolation-baseline-v1`, `make test-namespace-cgroup-v1`, CI `Isolation baseline v1 gate` + `Namespace cgroup v1 gate`, docs in `docs/abi/namespace_cgroup_contract_v1.md`, `docs/runtime/resource_control_policy_v1.md`, and `docs/M42_EXECUTION_BACKLOG.md`. |
| M43 | Hardware/Firmware Breadth + SMP v1 | n/a | done | Rugo: matrix-v5/driver-lifecycle-v5/firmware-hardening-v3 + SMP interrupt model contracts with deterministic hardware/firmware/SMP artifacts, `make test-hw-firmware-smp-v1`, `make test-native-driver-matrix-v1`, CI `Hardware firmware smp v1 gate` + `Native driver matrix v1 gate`, docs in `docs/hw/support_matrix_v5.md`, `docs/hw/driver_lifecycle_contract_v5.md`, `docs/hw/acpi_uefi_hardening_v3.md`, `docs/runtime/smp_interrupt_model_v1.md`, and `docs/M43_EXECUTION_BACKLOG.md`. |
| M44 | Real Desktop + Ecosystem Qualification v2 | n/a | done | Rugo: desktop/app-tier/ecosystem v2 contracts + deterministic runtime qualification artifacts, `make test-real-ecosystem-desktop-v2`, `make test-real-app-catalog-v2`, CI `Real ecosystem desktop v2 gate` + `Real app catalog v2 gate`, docs in `docs/desktop/desktop_profile_v2.md`, `docs/abi/app_compat_tiers_v2.md`, `docs/pkg/ecosystem_scale_policy_v2.md`, `docs/pkg/distribution_workflow_v2.md`, and `docs/M44_EXECUTION_BACKLOG.md`. |
| M45 | Modern Virtual Platform Parity v1 | n/a | done | Rugo: matrix v6 / driver-lifecycle v6 / virtio-platform-profile v1 contracts plus deterministic modern VirtIO shadow artifacts, `make test-hw-matrix-v6`, `make test-virtio-platform-v1`, CI `Hardware matrix v6 shadow gate` + `Virtio platform v1 shadow gate`, docs in `docs/hw/support_matrix_v6.md`, `docs/hw/driver_lifecycle_contract_v6.md`, `docs/hw/virtio_platform_profile_v1.md`, `docs/desktop/display_stack_contract_v1.md`, and `docs/M45_EXECUTION_BACKLOG.md`. |
| M46 | Bare-Metal I/O Baseline v1 | n/a | done | Rugo: baremetal-io-profile-v1 and usb-input-removable-contract-v1 contracts plus deterministic wired-NIC, USB input, and removable-media qualification artifacts, `make test-baremetal-io-baseline-v1`, `make test-usb-input-removable-v1`, CI `Bare-metal io baseline v1 gate` + `USB input removable v1 gate`, docs in `docs/hw/baremetal_io_profile_v1.md`, `docs/hw/usb_input_removable_contract_v1.md`, `docs/desktop/input_stack_contract_v1.md`, and `docs/M46_EXECUTION_BACKLOG.md`. |
| M47 | Hardware Claim Promotion Program v1 | n/a | done | Rugo: support-claim-policy-v1 / bare-metal-promotion-policy-v2 / support-tier-audit-v1 contracts plus deterministic claim-promotion and support-tier-audit artifacts, `make test-hw-claim-promotion-v1`, `make test-hw-support-tier-audit-v1`, CI `Hardware claim promotion v1 gate` + `Hardware support tier audit v1 gate`, docs in `docs/hw/support_claim_policy_v1.md`, `docs/hw/bare_metal_promotion_policy_v2.md`, `docs/hw/support_tier_audit_v1.md`, and `docs/M47_EXECUTION_BACKLOG.md`. |
| M48 | Display Runtime + Scanout v1 | n/a | done | Rugo: display-runtime-contract-v1 / scanout-buffer-contract-v1 / gpu-fallback-policy-v1 contracts plus deterministic scanout, timing, and frame-capture artifacts, `make test-display-runtime-v1`, `make test-scanout-path-v1`, CI `Display runtime v1 gate` + `Scanout path v1 gate`, docs in `docs/desktop/display_runtime_contract_v1.md`, `docs/desktop/scanout_buffer_contract_v1.md`, `docs/desktop/gpu_fallback_policy_v1.md`, and `docs/M48_EXECUTION_BACKLOG.md`. |
| M49 | Input + Seat Management v1 | n/a | done | Rugo: seat-input-contract-v1 / input-event-contract-v1 / focus-routing-policy-v1 contracts plus deterministic seat runtime and HID event-path artifacts, `make test-input-seat-v1`, `make test-hid-event-path-v1`, CI `Input seat v1 gate` + `HID event path v1 gate`, docs in `docs/desktop/seat_input_contract_v1.md`, `docs/desktop/input_event_contract_v1.md`, `docs/desktop/focus_routing_policy_v1.md`, and `docs/M49_EXECUTION_BACKLOG.md`. |
| M50 | Window System + Composition v1 | n/a | done | Rugo: surface-lifecycle-contract-v1 / compositor-damage-policy-v1 / window-manager-contract-v2 contracts plus deterministic window runtime and compositor damage artifacts, `make test-window-system-v1`, `make test-compositor-damage-v1`, CI `Window system v1 gate` + `Compositor damage v1 gate`, docs in `docs/desktop/surface_lifecycle_contract_v1.md`, `docs/desktop/compositor_damage_policy_v1.md`, `docs/desktop/window_manager_contract_v2.md`, and `docs/M50_EXECUTION_BACKLOG.md`. |
| M51 | GUI Runtime + Toolkit Bridge v1 | n/a | done | Rugo: gui-runtime-contract-v1 / toolkit-profile-v1 / font-text-rendering-policy-v1 contracts plus deterministic GUI runtime and toolkit compatibility artifacts, `make test-gui-runtime-v1`, `make test-toolkit-compat-v1`, CI `GUI runtime v1 gate` + `Toolkit compatibility v1 gate`, docs in `docs/desktop/gui_runtime_contract_v1.md`, `docs/desktop/toolkit_profile_v1.md`, `docs/desktop/font_text_rendering_policy_v1.md`, and `docs/M51_EXECUTION_BACKLOG.md`. |
| M52 | Desktop Shell + Workflow Baseline v1 | n/a | done | Rugo: desktop-shell-contract-v1 / session-workflow-profile-v1 / graphical-installer-ux-v1 contracts plus deterministic desktop shell and graphical installer workflow artifacts, `make test-desktop-shell-v1`, `make test-desktop-workflows-v1`, CI `Desktop shell v1 gate` + `Desktop workflows v1 gate`, docs in `docs/desktop/desktop_shell_contract_v1.md`, `docs/desktop/session_workflow_profile_v1.md`, `docs/build/graphical_installer_ux_v1.md`, and `docs/M52_EXECUTION_BACKLOG.md`. |
Legend: ✅ = done with passing tests, ◐ = in progress (prep), — = not started, n/a = not applicable to this lane.

---

## Intent and architecture boundary

- Rust owns kernel mechanisms (correctness-critical, no_std).
- Go owns high-level user space services and the ecosystem.
- Go Track A (TinyGo-first) is the short-term path to get Go services running.
- Go Track B (full Go port) is explicitly long-term and should start only after
  the kernel ABI/process model stabilizes.

### Kernel (Rust) provides mechanisms

- Memory management: physical allocator, paging, per-process page tables
- Traps: IDT, exception handling, syscall entry dispatch
- Scheduling: threads, runnable queues, timers
- IPC and shared memory primitives
- Security boundary: handle table or capabilities
- Minimal drivers required to boot and test under QEMU (VirtIO preferred early)

### User space (Go services) provides policy and ecosystem

- Service manager: svcman
- Filesystem service: fsd
- Package manager: pkg and pkgd
- Network stack service: netd
- Shell and core utilities

---

## Key constraints and design choices

### Rust kernel target and no_std stance

- Kernel code is Rust `no_std` compiled for `x86_64-unknown-none`.
- A thin assembly layer exists for CPU entry (`arch/x86_64/entry.asm`).
- Build uses `build-std = ["core"]` with `compiler-builtins-mem`.

### Go in user space, not kernel

- Go is used for high-level services in user space (fsd, pkgd, netd, svcman).
- Running standard Go binaries on a new OS is a porting effort (a new
  GOOS/GOARCH combination). This is not the short-term path.

### Go track decision (explicit)

Track A: TinyGo-first (short-term)
- Goal: get 1-3 Go services running early with a constrained subset of Go/stdlib.
- Expectation: avoid heavy reflection patterns; keep deps minimal.

Track B: Full Go port (G2 baseline complete)
- Goal: run standard Go binaries produced by the stock toolchain.
- Start condition: syscall ABI and process model have stopped changing.

---

## M0: Boot + serial logging

### Definition of done

- Limine boots the kernel in QEMU.
- Serial output prints deterministic markers and exits cleanly via debug-exit.
- Panic path prints a deterministic marker and exits.

### Acceptance tests (marker-based)

| Test | Markers | ISO |
|------|---------|-----|
| `test_boot_banner` | `RUGO: boot ok`, `RUGO: halt ok` | Normal (`out/os.iso`) |
| `test_panic_path` | `RUGO: panic code=...` | Panic (`out/os-panic.iso`, built with `panic_test` feature) |

### Legacy evidence

- `legacy/tests/boot/test_boot_banner.py` asserts `KERNEL: boot ok`, `KERNEL: halt ok`
- `legacy/kernel/main.c`, `legacy/arch/x86_64/entry.asm`

### Rugo evidence

- `tests/boot/test_boot_banner.py` asserts `RUGO: boot ok`, `RUGO: halt ok`
- `tests/boot/test_panic_path.py` asserts `RUGO: panic code=`
- `kernel_rs/src/lib.rs` (kmain + panic handler)
- `arch/x86_64/entry.asm` (sets stack, calls kmain)

---

## M1: Paging + traps + page fault reporting

### Definition of done

- Paging enabled with known mapping strategy.
- IDT installed.
- Page fault handler logs fault address and error bits.

### Acceptance tests

| Test | Markers |
|------|---------|
| `test_paging_enabled` | `MM: paging=on` |
| `test_page_fault_report` | `PF: addr=0x... err=0x...` |
| `test_idt_smoke` | Deterministic trap marker for a forced exception |

### Legacy evidence

- `legacy/tests/boot/test_paging_enabled.py`, `legacy/tests/trap/test_page_fault_report.py`
- `legacy/kernel/vmm.c`, `legacy/kernel/pmm.c`, `legacy/arch/x86_64/idt.c`, `legacy/arch/x86_64/trap.c`

### Rugo evidence

- `tests/boot/test_paging_enabled.py`, `tests/trap/test_page_fault_report.py`, `tests/trap/test_idt_smoke.py`
- `kernel_rs/src/lib.rs` (paging enable marker, IDT install, fault/trap handlers)
- `arch/x86_64/isr.asm` (ISR stubs)

---

## M2: Scheduler + kernel threads

### Definition of done

- Timer interrupts tick.
- At least two kernel threads can run and context switch.
- A minimal lock primitive exists and is safe with interrupts.

### Acceptance tests

| Test | Markers |
|------|---------|
| `test_timer_ticks` | `TICK: 100` |
| `test_two_threads` | Interleaved `A` and `B` |

### Legacy evidence

- `legacy/tests/sched/test_timer_ticks.py`, `legacy/tests/sched/test_two_threads.py`
- `legacy/kernel/sched.c`, `legacy/arch/x86_64/pit.c`, `legacy/arch/x86_64/context.asm`

### Rugo evidence

- `tests/sched/test_timer_ticks.py`, `tests/sched/test_two_threads.py`
- `kernel_rs/src/lib.rs` (timer tick path and scheduler test flow)
- `arch/x86_64/context.asm` (context switch assembly)

---

## M3: User mode + syscall ABI v0

### Definition of done

- Kernel can enter ring 3.
- Syscalls round-trip safely.
- User faults are contained and do not crash the kernel.

### Syscall ABI v0 (minimum)

- debug: `sys_debug_write`
- tasking: `sys_thread_spawn`, `sys_thread_exit`, `sys_yield`
- memory: `sys_vm_map`, `sys_vm_unmap`
- shm: `sys_shm_create`, `sys_shm_map`
- ipc: `sys_ipc_send`, `sys_ipc_recv`
- time: `sys_time_now`

### Acceptance tests

| Test | Markers |
|------|---------|
| `test_enter_user_mode` | `USER: hello` |
| `test_syscall_roundtrip` | `SYSCALL: ok` |
| `test_thread_exit` | `THREAD_EXIT: ok` and clean halt |
| `test_user_fault` | `USER: killed` and kernel continues |

### Legacy evidence

- `legacy/tests/user/test_enter_user_mode.py`, `test_syscall_roundtrip.py`, `test_user_fault.py`
- `legacy/kernel/syscall.c`, `legacy/kernel/process.c`, `legacy/user/init.c`, `legacy/user/fault.c`

### Rugo evidence

- `tests/user/test_enter_user_mode.py`, `tests/user/test_syscall_roundtrip.py`, `tests/user/test_thread_exit.py`, `tests/user/test_user_fault.py`
- `docs/abi/syscall_v0.md` (ABI contract)
- `kernel_rs/src/lib.rs` (ring 3 entry, syscall dispatch, user-fault containment)

---

## M4: IPC + shared memory + service registry v0

### Definition of done

- Two user processes can exchange messages through kernel IPC endpoints.
- Shared memory can be created and mapped by two processes.
- A service registry exists for name-to-endpoint resolution.
- Service registration rejects inactive or out-of-range endpoint IDs.

### Acceptance tests

| Test | Markers |
|------|---------|
| `test_ping_pong` | `PING: ok`, `PONG: ok` |
| `test_shm_bulk` | `SHM: checksum ok` |

### Legacy evidence

- `legacy/tests/ipc/test_ping_pong.py`, `legacy/tests/ipc/test_shm_bulk.py`
- `legacy/kernel/ipc.c`, `legacy/kernel/shm.c`, `legacy/kernel/service_registry.c`
- `legacy/user/ping.c`, `legacy/user/pong.c`, `legacy/user/shm_writer.c`, `legacy/user/shm_reader.c`

### Rugo evidence

- `tests/ipc/test_ping_pong.py`, `tests/ipc/test_shm_bulk.py`
- `tests/ipc/test_svc_overwrite.py`, `tests/ipc/test_svc_full.py`, `tests/ipc/test_svc_bad_endpoint.py`
- `kernel_rs/src/lib.rs` (IPC, SHM, and service-registry syscall paths)

---

## M5: VirtIO block + block syscalls

### Definition of done

- VirtIO block works in QEMU.
- Kernel exposes block read/write syscalls.
- Read/write passes deterministic data integrity tests.

### Acceptance tests

| Test | Markers |
|------|---------|
| `test_virtio_blk_identify` | `BLK: found virtio-blk` |
| `test_virtio_blk_rw` | `BLK: rw ok` |
| `test_virtio_blk_init_invariants` | `BLK: invariants ok` |
| `test_virtio_blk_init_failure` | `BLK: init failed` |

### Legacy evidence

- `legacy/tests/drivers/test_virtio_blk_identify.py`, `legacy/tests/drivers/test_virtio_blk_rw.py`
- `legacy/kernel/virtio_blk.c`, `legacy/kernel/pci.c`

### Rugo evidence

- `tests/drivers/test_virtio_blk_identify.py`, `tests/drivers/test_virtio_blk_rw.py`
- `tests/drivers/test_virtio_blk_init_invariants.py`, `tests/drivers/test_virtio_blk_init_failure.py`
- `kernel_rs/src/lib.rs` (VirtIO block init + `sys_blk_read`/`sys_blk_write`)
- `docs/abi/syscall_v0.md` (`sys_blk_read` and `sys_blk_write`)

---

## M6: Filesystem + package manager + shell

### Definition of done

- A filesystem service exists that can open, read, write, list.
- A package tool can install a package and run an app.
- A shell can run a packaged hello app.

### Acceptance tests

| Test | Markers |
|------|---------|
| `test_fsd_smoke` | `FSD: mount ok` |
| `test_pkg_install_run` | `APP: hello world` |

### Legacy evidence

- `legacy/tests/fs/test_fsd_smoke.py`, `legacy/tests/pkg/test_pkg_install_run.py`
- `legacy/user/fsd.c`, `legacy/user/pkg.c`, `legacy/user/sh.c`, `legacy/user/hello.c`

### Rugo evidence

- `tests/fs/test_fsd_smoke.py` asserts `FSD: mount ok`
- `tests/pkg/test_pkg_install_run.py` asserts `APP: hello world`
- `kernel_rs/src/lib.rs` (fs_test feature: SimpleFS v0 mount + PKG v0 parse + hello user-mode)
- `tools/mkfs.py` (deterministic disk image builder)
- `docs/storage/fs_v0.md` (on-disk format documentation)

---

## M7: VirtIO net + net syscalls + UDP echo (optional)

### Definition of done

- VirtIO net works in QEMU.
- Kernel exposes send/recv syscalls.
- A UDP echo demo passes.

### Acceptance tests

| Test | Markers |
|------|---------|
| `test_udp_echo` | `NET: udp echo` |

### Legacy evidence

- `legacy/tests/net/test_udp_echo.py`
- `legacy/kernel/virtio_net.c`, `legacy/user/netd.c`

### Rugo evidence

- `tests/net/test_udp_echo.py` asserts `NET: udp echo`
- `kernel_rs/src/lib.rs` (net_test feature: VirtIO net driver + ARP/UDP echo)
- `docs/net/udp_echo_v0.md` (protocol documentation)
- `docs/abi/syscall_v0.md` (sys_net_send/sys_net_recv at syscalls 15–16)

---

## G0: Go kernel entry (gccgo) — Legacy only

This milestone exists only in the legacy lane. It demonstrated Go code running
in kernel context via gccgo. The Rugo lane does not use gccgo; Go services will
run in user space (see G1, G2).

### Acceptance tests

| Test | Markers |
|------|---------|
| `test_go_entry` | `GO: kmain ok` |

### Legacy evidence

- `legacy/tests/boot/test_go_entry.py`
- `legacy/kernelgo/entry.go`, `legacy/rtshim/bridge.c`, `legacy/rtshim/runtime_stubs.c`

---

## G1: Go services bringup (Track A: TinyGo-first) — Rugo only

### Definition of done

- At least one user space service is written in Go and runs on the Rugo kernel.
- The service uses the existing syscall ABI and IPC primitives.
- A test asserts `GOUSR: ok`.

### Acceptance tests

| Test | Markers |
|------|---------|
| `test_go_user_service` | `GOUSR: ok` |

### Rugo evidence

- `tests/go/test_go_user_service.py` asserts `GOUSR: ok`
- `services/go/main.go` (TinyGo user program, prints "GOUSR: ok" via sys_debug_write)
- `services/go/start.asm` (NASM: entry point, syscall wrappers, runtime stubs, bump allocator)
- `services/go/linker.ld` (user-space linker script, code at VA 0x400000)
- `tools/build_go.sh` (TinyGo build pipeline: NASM + TinyGo + LLD + objcopy)
- `kernel_rs/src/lib.rs` (go_test feature: embeds gousr.bin, enters ring 3)
- `Dockerfile` (updated with Go 1.25 + TinyGo 0.40.1 for Docker builds)

---

## G2: Full Go port (Track B) — Rugo only

Milestone status: done (2026-03-04).

### Start conditions (gate)

- Syscall ABI has been stable across multiple releases.
- Process/thread model is stable and documented.
- Filesystem and networking contracts are stable enough that the Go runtime can
  rely on them.

### Definition of done

- Standard Go toolchain can produce binaries for your GOOS/GOARCH.
- Go runtime syscalls and thread management work on the OS.
- A standard Go hello binary runs and prints `GOSTD: ok`.

### Acceptance tests

| Test | Markers |
|------|---------|
| `test_std_go_binary` | `GOSTD: ok` |

### Rugo evidence

- `tests/go/test_std_go_binary.py` asserts `GOSTD: ok` marker flow plus
  stock-Go artifact contract metadata from `out/gostd-contract.env`.
- `tools/build_go_std_spike.sh` now uses stock Go (`go run ./tools/gostd_stock_builder/main.go`)
  to produce `out/gostd.bin` and `out/gostd-contract.env`.
- `make image-go-std` builds the G2 image path (`os-go-std.iso`) with kernel
  `go_std_test` embedding `out/gostd.bin`.
- Go-port contract and bridge map remains documented in
  `docs/abi/go_port_spike_v0.md` (`GOOS=rugo`, `GOARCH=amd64`).
- Execution history and PR sequencing are recorded in
  `docs/G2_EXECUTION_BACKLOG.md`.

---

## M8: Compatibility Profile v1

Milestone status: done (2026-03-04).

### PR-1 deliverables completed

- Versioned ABI contract: `docs/abi/syscall_v1.md`.
- Compatibility profile contract: `docs/abi/compat_profile_v1.md`.
- Initial compatibility suite scaffolding under `tests/compat/`.

### PR-2 deliverables completed

- Loader/process/fd contract documentation:
  - `docs/abi/process_thread_model_v1.md`
  - updated `docs/abi/syscall_v1.md`
- Kernel v1 compatibility primitives in `kernel_rs/src/lib.rs`:
  - ELF64 validation helper policy (`elf_v1_validate_image`)
  - core compatibility syscalls: open/read/write/close/wait/poll
  - deterministic M3 fd-table baseline (`/dev/console`, `/compat/hello.txt`)
- Compatibility tests promoted from TODO skeletons to executable checks:
  - `tests/compat/test_loader_contract.py`
  - `tests/compat/test_process_lifecycle.py`
  - `tests/compat/test_process_wait.py`
  - `tests/compat/test_fd_table.py`
  - updated `tests/compat/test_file_io_subset.py`
- Backlog execution status updated: `docs/M8_EXECUTION_BACKLOG.md`.

### PR-3 deliverables completed

- Remaining POSIX subset closure (time/signal/socket):
  - `tests/compat/test_time_signal_subset.py`
  - `tests/compat/test_socket_api_subset.py`
  - `tests/compat/test_posix_subset.py`
- Package/repository v1 bootstrap:
  - `docs/pkg/package_format_v1.md`
  - `tools/pkg_bootstrap_v1.py`
  - `tests/pkg/test_pkg_external_apps.py`
- Compatibility/profile gating promoted to CI:
  - `.github/workflows/ci.yml` explicit compatibility profile v1 gate step

---

## M9: Hardware Enablement Matrix v1

Milestone status: done (2026-03-04).

### Definition of done

- Published hardware matrix v1 with Tier 0/Tier 1 target classes.
- Deterministic storage + network smoke checks run on both matrix tiers.
- PCI probe/init path is shared across in-tree virtio drivers.
- DMA invalid-input rejection checks are included in matrix acceptance.
- ACPI/UEFI hardening policy and bare-metal bring-up runbook are published.

### Acceptance tests

| Test | Markers |
|------|---------|
| `tests/hw/test_hardware_matrix_v1.py` | `BLK: found virtio-blk`, `BLK: rw ok`, `NET: virtio-net ready`, `NET: udp echo` |
| `tests/hw/test_probe_negative_paths_v1.py` | `BLK: not found`, `NET: not found` |
| `tests/hw/test_dma_safety_v1.py` | `BLK: badlen ok`, `BLK: badptr ok` |

### Rugo evidence

- PCI helper cleanup + arbitration baseline in `kernel_rs/src/lib.rs`:
  - shared device lookup and BAR parsing helpers,
  - shared bus-master enable helper,
  - PCI function claim guard.
- Hardware matrix docs:
  - `docs/hw/support_matrix_v1.md`
  - `docs/hw/dma_iommu_strategy_v1.md`
  - `docs/hw/acpi_uefi_hardening_v1.md`
  - `docs/hw/bare_metal_bringup_v1.md`
- Execution and sequencing history:
  - `docs/M9_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` target `test-hw-matrix`
- `.github/workflows/ci.yml` step `Hardware matrix v1 gate`

---

## M10: Security Baseline v1

Milestone status: done (2026-03-04).

### Definition of done

- Per-handle rights model is documented and enforced in kernel paths.
- Restricted syscall/resource profile exists with deterministic deny behavior.
- Boot artifact manifest signing and verification rejects tampered artifacts.
- Continuous security fuzz gate runs and produces machine-readable reports.
- Incident response and advisory workflow is documented.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/security/test_rights_enforcement.py` | `SEC: rights ok` |
| `tests/security/test_syscall_filter.py` | `SEC: filter ok` |
| `tests/security/test_secure_boot_manifest_v1.py` | tamper rejection + key rotation pass |
| `tests/security/test_security_fuzz_harness_v1.py` | fuzz report schema, `total_violations == 0` |

### Rugo evidence

- Kernel enforcement in `kernel_rs/src/lib.rs`:
  - fd rights fields and checks in `sys_open_v1/sys_read_v1/sys_write_v1/sys_poll_v1`,
  - new security syscalls:
    - `sys_fd_rights_get_v1` (24),
    - `sys_fd_rights_reduce_v1` (25),
    - `sys_fd_rights_transfer_v1` (26),
    - `sys_sec_profile_set_v1` (27),
  - restricted profile allowlist in `m10_syscall_allowed`.
- Security user-mode acceptance payloads:
  - `services/security/sec_rights.asm`
  - `services/security/sec_filter.asm`
- Security tooling:
  - `tools/secure_boot_manifest_v1.py`
  - `tools/run_security_fuzz_v1.py`
- Security contracts/process docs:
  - `docs/security/rights_capability_model_v1.md`
  - `docs/security/syscall_filtering_v1.md`
  - `docs/security/secure_boot_policy_v1.md`
  - `docs/security/fuzzing_v1.md`
  - `docs/security/incident_response_v1.md`
- Execution and sequencing history:
  - `docs/M10_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` target `test-security-baseline`
  - `.github/workflows/ci.yml` step `Security baseline v1 gate`

---

## M11: Runtime + Toolchain Maturity v1

Milestone status: done (2026-03-04).

### Definition of done

- Runtime-port contract is documented for `GOOS=rugo` and `GOARCH=amd64`.
- Runtime syscall coverage matrix exists with explicit owner and target rows.
- Toolchain bootstrap and reproducibility artifacts are generated by scripts.
- Runtime maturity is release-gated in local and CI lanes.
- Maintainership and ABI-window policy are documented.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/runtime/test_runtime_stress_v1.py` | `GOSTD:*` runtime markers + stress/pressure baselines |
| `tests/runtime/test_runtime_abi_window_v1.py` | ABI policy + syscall docs alignment |
| `tests/runtime/test_runtime_contract_docs_v1.py` | runtime docs/tools/gate wiring present |

### Rugo evidence

- Runtime contract docs:
  - `docs/runtime/port_contract_v1.md`
  - `docs/runtime/syscall_coverage_matrix_v1.md`
  - `docs/runtime/abi_stability_policy_v1.md`
  - `docs/runtime/toolchain_bootstrap_v1.md`
  - `docs/runtime/maintainers_v1.md`
- Runtime tooling:
  - `tools/bootstrap_go_port_v1.sh`
  - `tools/runtime_toolchain_contract_v1.py`
- Runtime test gate:
  - `tests/runtime/test_runtime_contract_docs_v1.py`
  - `tests/runtime/test_runtime_abi_window_v1.py`
  - `tests/runtime/test_runtime_stress_v1.py`
- Execution and sequencing history:
  - `docs/M11_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` target `test-runtime-maturity`
  - `.github/workflows/ci.yml` step `Runtime + toolchain maturity v1 gate`

---

## M12: Network Stack v1

Milestone status: done (2026-03-04).

### Definition of done

- Network stack v1 contract docs are published and versioned.
- IPv4/UDP baseline behavior is deterministic and test-backed.
- TCP state-machine and retransmission policy baselines are documented and
  tested.
- IPv6 ND + ICMPv6 baseline is documented and covered by model checks.
- Interop and soak artifact lanes are release-gated locally and in CI.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/net/test_udp_echo.py` | `NET: virtio-net ready`, `NET: udp echo`, no timeout marker |
| `tests/net/test_ipv4_udp_contract_v1.py` | IPv4/UDP contract + tiered net fixture checks |
| `tests/net/test_tcp_state_machine_v1.py` | TCP lifecycle state-transition baseline |
| `tests/net/test_tcp_retransmission_v1.py` | deterministic retransmission/backoff/timeout policy |
| `tests/net/test_ipv6_nd_icmpv6_v1.py` | IPv6 ND + ICMPv6 baseline model checks |
| `tests/net/test_net_interop_matrix_v1.py` | interop report schema + target threshold |
| `tests/net/test_net_soak_v1.py` | soak/fault-injection report schema + failure threshold |

### Rugo evidence

- Network contract docs:
  - `docs/net/network_stack_contract_v1.md`
  - `docs/net/socket_contract_v1.md`
  - `docs/net/ipv4_udp_profile_v1.md`
  - `docs/net/tcp_state_machine_v1.md`
  - `docs/net/retransmission_timer_policy_v1.md`
  - `docs/net/ipv6_baseline_v1.md`
- Network tooling:
  - `tools/net_trace_capture_v1.py`
  - `tools/run_net_interop_matrix_v1.py`
  - `tools/run_net_soak_v1.py`
- Network test gate:
  - `tests/net/test_socket_contract_docs_v1.py`
  - `tests/net/test_socket_poll_semantics_v1.py`
  - `tests/net/test_net_trace_capture_v1.py`
- Execution and sequencing history:
  - `docs/M12_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` target `test-network-stack-v1`
  - `.github/workflows/ci.yml` step `Network stack v1 gate`

---

## M13: Storage Reliability v1

Milestone status: done (2026-03-04).

### Definition of done

- Storage v1 contracts are documented for crash model and durability classes.
- Write-ordering/barrier semantics are explicitly defined and test-backed.
- Recovery and fault-campaign tooling produce machine-readable reports.
- Storage reliability is release-gated in local and CI lanes.
- M13 evidence is linked in backlog/status docs.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/storage/test_storage_contract_docs_v1.py` | storage docs/tools/gate wiring present |
| `tests/storage/test_fsync_semantics_v1.py` | `fdatasync` vs `fsync` durability semantics |
| `tests/storage/test_write_ordering_contract_v1.py` | ordered commit baseline and invalid ordering rejection |
| `tests/storage/test_storage_recovery_v1.py` | recovery report schema + mountability checks |
| `tests/storage/test_storage_fault_campaign_v1.py` | campaign report schema + threshold checks |
| `tests/storage/test_storage_integrity_checker_v1.py` | checksum/corruption detection baseline |
| `tests/storage/test_storage_reliability_gate_v1.py` | make/ci/status wiring for M13 gate |

### Rugo evidence

- Storage contract docs:
  - `docs/storage/fs_v1.md`
  - `docs/storage/durability_model_v1.md`
  - `docs/storage/write_ordering_policy_v1.md`
  - `docs/storage/recovery_playbook_v1.md`
  - `docs/storage/fault_campaign_v1.md`
- Storage tooling:
  - `tools/storage_recover_v1.py`
  - `tools/run_storage_fault_campaign_v1.py`
- Storage test gate:
  - `tests/storage/test_storage_recovery_v1.py`
  - `tests/storage/test_storage_fault_campaign_v1.py`
  - `tests/storage/test_storage_reliability_gate_v1.py`
- Execution and sequencing history:
  - `docs/M13_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` target `test-storage-reliability-v1`
  - `.github/workflows/ci.yml` step `Storage reliability v1 gate`

---

## M14: Productization + Release Engineering v1

Milestone status: done (2026-03-04).

### Definition of done

- Release/channel/versioning policy is documented and executable.
- Signed update metadata and client verification enforce rollback/replay/freeze
  protection.
- Supply-chain artifacts (SBOM + provenance) are generated in local/CI gates.
- Release engineering is release-gated in local and CI lanes.
- Installer/recovery/support-bundle baseline is documented and test-backed.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/build/test_release_contract_docs_v1.py` | release policy/versioning/checklist docs and tool presence |
| `tests/build/test_release_contract_report_v1.py` | release contract report schema + channel metadata |
| `tests/pkg/test_update_metadata_v1.py` | signed update metadata + verify roundtrip |
| `tests/pkg/test_update_rollback_protection_v1.py` | replay/rollback rejection by monotonic sequence |
| `tests/pkg/test_update_attack_suite_v1.py` | attack-suite report schema + zero failures target |
| `tests/build/test_release_engineering_gate_v1.py` | make/ci/docs wiring + supply-chain/support artifacts |

### Rugo evidence

- Release and productization docs:
  - `docs/build/release_policy_v1.md`
  - `docs/build/versioning_scheme_v1.md`
  - `docs/build/release_checklist_v1.md`
  - `docs/build/supply_chain_policy_v1.md`
  - `docs/build/installer_recovery_baseline_v1.md`
  - `docs/pkg/update_protocol_v1.md`
  - `docs/pkg/update_repo_layout_v1.md`
  - `docs/security/update_signing_policy_v1.md`
- Tooling:
  - `tools/release_contract_v1.py`
  - `tools/update_repo_sign_v1.py`
  - `tools/update_client_verify_v1.py`
  - `tools/run_update_attack_suite_v1.py`
  - `tools/generate_sbom_v1.py`
  - `tools/generate_provenance_v1.py`
  - `tools/collect_support_bundle_v1.py`
- Release test gate:
  - `tests/build/test_release_engineering_gate_v1.py`
  - `tests/pkg/test_update_metadata_v1.py`
  - `tests/pkg/test_update_rollback_protection_v1.py`
  - `tests/pkg/test_update_attack_suite_v1.py`
- Execution and sequencing history:
  - `docs/M14_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` target `test-release-engineering-v1`
  - `.github/workflows/ci.yml` step `Release engineering v1 gate`

---

## M15: Hardware Enablement Matrix v2

Milestone status: done (2026-03-06).

### Definition of done

- Tiered hardware matrix v2 criteria are versioned and executable.
- Tier 0/Tier 1 checks are release-gated in local and CI lanes.
- Bare-metal promotion policy is evidence-bound and deterministic.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/hw/test_hardware_matrix_v2.py` | Tier 0/Tier 1 storage/network marker parity |
| `tests/hw/test_probe_negative_paths_v2.py` | deterministic probe-missing baseline |
| `tests/hw/test_dma_iommu_policy_v2.py` | DMA invalid input rejection baseline |
| `tests/hw/test_acpi_boot_paths_v2.py` | deterministic firmware boot/halt markers |
| `tests/hw/test_bare_metal_smoke_v2.py` | runbook evidence bundle + smoke markers |
| `tests/hw/test_hw_gate_v2.py` | make/ci/backlog gate wiring |

### Rugo evidence

- Contract docs:
  - `docs/hw/support_matrix_v2.md`
  - `docs/hw/device_profile_contract_v2.md`
  - `docs/hw/dma_iommu_strategy_v2.md`
  - `docs/hw/acpi_uefi_hardening_v2.md`
  - `docs/hw/bare_metal_bringup_v2.md`
- Test gate:
  - `tests/hw/test_hardware_matrix_v2.py`
  - `tests/hw/test_probe_negative_paths_v2.py`
  - `tests/hw/test_dma_iommu_policy_v2.py`
  - `tests/hw/test_acpi_boot_paths_v2.py`
  - `tests/hw/test_bare_metal_smoke_v2.py`
  - `tests/hw/test_hw_gate_v2.py`
- Execution history:
  - `docs/M15_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` target `test-hw-matrix-v2`
  - `.github/workflows/ci.yml` step `Hardware matrix v2 gate`

---

## M16: Process + Scheduler Model v2

Milestone status: done (2026-03-06).

### Definition of done

- Process/thread lifecycle semantics are versioned in v2 ABI docs.
- Preemption and fairness regressions are deterministic and gate-blocking.
- Faulted user tasks are contained without scheduler collapse.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/user/test_process_wait_kill_v2.py` | wait/kill lifecycle contract + fault/exit markers |
| `tests/user/test_signal_delivery_v2.py` | deterministic signal delivery and queue bounds |
| `tests/sched/test_preempt_timer_quantum_v2.py` | timer quantum preemption determinism |
| `tests/sched/test_priority_fairness_v2.py` | equal-priority and weighted fairness checks |
| `tests/sched/test_scheduler_soak_v2.py` | seeded soak determinism + machine-readable failures |
| `tests/sched/test_scheduler_gate_v2.py` | make/ci/docs gate wiring and closure |

### Rugo evidence

- Contract docs:
  - `docs/abi/process_thread_model_v2.md`
  - `docs/abi/scheduling_policy_v2.md`
- Scheduler/process model:
  - `tests/sched/v2_model.py`
- Test gate:
  - `tests/user/test_process_wait_kill_v2.py`
  - `tests/user/test_signal_delivery_v2.py`
  - `tests/sched/test_preempt_timer_quantum_v2.py`
  - `tests/sched/test_priority_fairness_v2.py`
  - `tests/sched/test_scheduler_soak_v2.py`
  - `tests/sched/test_scheduler_gate_v2.py`
- Execution history:
  - `docs/M16_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` target `test-process-scheduler-v2`
  - `.github/workflows/ci.yml` step `Process scheduler v2 gate`

---

## M17: Compatibility Profile v2

Milestone status: done (2026-03-06).

### Definition of done

- ABI/loader v2 contracts are versioned and executable-check referenced.
- Supported vs unsupported compatibility surfaces are explicit.
- External app tier thresholds are deterministic and release-gated.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/compat/test_abi_profile_v2_docs.py` | v2 ABI/profile/loader doc contracts + kernel dispatch wiring |
| `tests/compat/test_elf_loader_dynamic_v2.py` | deterministic static/dynamic loader acceptance and rejection rules |
| `tests/compat/test_posix_profile_v2.py` | required/optional/unsupported profile surface model checks |
| `tests/compat/test_external_apps_tier_v2.py` | signed artifact tier thresholds + Tier B runtime marker checks |
| `tests/compat/test_compat_gate_v2.py` | make/ci/docs gate wiring and closure |

### Rugo evidence

- Contract docs:
  - `docs/abi/syscall_v2.md`
  - `docs/abi/compat_profile_v2.md`
  - `docs/abi/elf_loader_contract_v2.md`
  - `docs/runtime/syscall_coverage_matrix_v2.md`
- Compatibility model:
  - `tests/compat/v2_model.py`
- Test gate:
  - `tests/compat/test_abi_profile_v2_docs.py`
  - `tests/compat/test_elf_loader_dynamic_v2.py`
  - `tests/compat/test_posix_profile_v2.py`
  - `tests/compat/test_external_apps_tier_v2.py`
  - `tests/compat/test_compat_gate_v2.py`
- Execution history:
  - `docs/M17_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` target `test-compat-v2`
  - `.github/workflows/ci.yml` step `Compatibility profile v2 gate`

---

## M18: Storage Reliability v2

Milestone status: done (2026-03-06).

### Definition of done

- Storage v2 contract docs freeze journaling, durability, and recovery semantics.
- Recovery and power-fail artifacts are deterministic and machine-readable.
- Storage v2 gate is release-blocking in local and CI lanes.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/storage/test_journal_recovery_v2.py` | recovery report schema + mountability and journal state invariants |
| `tests/storage/test_metadata_integrity_v2.py` | metadata corruption detection, bounds checks, and doc contract tokens |
| `tests/storage/test_powerfail_campaign_v2.py` | deterministic power-fail campaign thresholds and report schema |
| `tests/storage/test_storage_gate_v2.py` | make/ci/docs gate wiring and closure |

### Rugo evidence

- Contract docs:
  - `docs/storage/fs_v2.md`
  - `docs/storage/durability_model_v2.md`
  - `docs/storage/write_ordering_policy_v2.md`
  - `docs/storage/recovery_playbook_v2.md`
  - `docs/storage/fault_campaign_v2.md`
- Tooling:
  - `tools/storage_recover_v2.py`
  - `tools/run_storage_powerfail_campaign_v2.py`
- Test gate:
  - `tests/storage/test_journal_recovery_v2.py`
  - `tests/storage/test_metadata_integrity_v2.py`
  - `tests/storage/test_powerfail_campaign_v2.py`
  - `tests/storage/test_storage_gate_v2.py`
- Execution history:
  - `docs/M18_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` target `test-storage-reliability-v2`
  - `.github/workflows/ci.yml` step `Storage reliability v2 gate`

---

## M19: Network Stack v2

Milestone status: done (2026-03-08).

### Definition of done

- Network v2 contract docs freeze TCP/IPv6/DNS baseline behavior.
- Interop and soak evidence is deterministic and machine-readable.
- Network v2 gate is release-blocking in local and CI lanes.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/net/test_tcp_interop_v2.py` | deterministic TCP peer interop profile checks |
| `tests/net/test_ipv6_interop_v2.py` | deterministic IPv6 ND/ICMPv6 + dual-stack fallback checks |
| `tests/net/test_dns_stub_v2.py` | DNS-stub TTL/NXDOMAIN semantics + v2 interop/soak schema checks |
| `tests/net/test_network_gate_v2.py` | make/ci/docs gate wiring and closure |

### Rugo evidence

- Contract docs:
  - `docs/net/network_stack_contract_v2.md`
  - `docs/net/socket_contract_v2.md`
  - `docs/net/tcp_profile_v2.md`
  - `docs/net/interop_matrix_v2.md`
- Tooling:
  - `tools/run_net_interop_matrix_v2.py`
  - `tools/run_net_soak_v2.py`
- Test gate:
  - `tests/net/test_tcp_interop_v2.py`
  - `tests/net/test_ipv6_interop_v2.py`
  - `tests/net/test_dns_stub_v2.py`
  - `tests/net/test_network_gate_v2.py`
- Execution history:
  - `docs/M19_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` target `test-network-stack-v2`
  - `.github/workflows/ci.yml` step `Network stack v2 gate`

---

## M20: Operability + Release UX v2

Milestone status: done (2026-03-09).

### Definition of done

- Installer/recovery and operations runbook contracts are versioned and
  executable-check referenced.
- Upgrade/rollback/recovery and support-bundle artifacts are deterministic and
  machine-readable.
- Operability v2 gate is release-blocking in local and CI lanes.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/build/test_installer_recovery_v2.py` | installer/recovery contract docs and installer artifact schema checks |
| `tests/build/test_upgrade_rollback_v2.py` | deterministic upgrade/rollback stage contract and drill report schema |
| `tests/build/test_support_bundle_v2.py` | support-bundle v2 schema and evidence collection checks |
| `tests/build/test_operability_gate_v2.py` | make/ci/docs gate wiring and closure |

### Rugo evidence

- Contract docs:
  - `docs/build/installer_recovery_baseline_v2.md`
  - `docs/build/operations_runbook_v2.md`
  - `docs/pkg/update_protocol_v2.md`
  - `docs/pkg/rollback_policy_v2.md`
- Tooling:
  - `tools/build_installer_v2.py`
  - `tools/run_upgrade_recovery_drill_v2.py`
  - `tools/collect_support_bundle_v2.py`
- Test gate:
  - `tests/build/test_installer_recovery_v2.py`
  - `tests/build/test_upgrade_rollback_v2.py`
  - `tests/build/test_support_bundle_v2.py`
  - `tests/build/test_operability_gate_v2.py`
- Execution history:
  - `docs/M20_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` target `test-release-ops-v2`
  - `.github/workflows/ci.yml` step `Operability and release UX v2 gate`

---

## M21: ABI + API Stability Program v3

Milestone status: done (2026-03-09).

### Definition of done

- ABI/API stability v3 contracts are versioned and test-backed.
- ABI diffs and compatibility policy checks are machine-enforced.
- ABI stability v3 gate is release-blocking in local and CI lanes.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/runtime/test_abi_docs_v3.py` | syscall v3 and policy docs include required contract tokens |
| `tests/runtime/test_abi_window_v3.py` | deprecation-window policy values and registry alignment checks |
| `tests/runtime/test_abi_diff_gate_v3.py` | deterministic ABI diff report schema and breaking-change detection |
| `tests/compat/test_abi_compat_matrix_v3.py` | compatibility matrix schema and explicit migration-action policy checks |
| `tests/runtime/test_abi_stability_gate_v3.py` | make/ci/docs gate wiring and closure |

### Rugo evidence

- Contract docs:
  - `docs/abi/syscall_v3.md`
  - `docs/runtime/abi_stability_policy_v2.md`
  - `docs/runtime/deprecation_window_policy_v1.md`
- Enforcement tooling:
  - `tools/check_abi_diff_v3.py`
  - `tools/check_syscall_compat_v3.py`
- Test gate:
  - `tests/runtime/test_abi_docs_v3.py`
  - `tests/runtime/test_abi_window_v3.py`
  - `tests/runtime/test_abi_diff_gate_v3.py`
  - `tests/compat/test_abi_compat_matrix_v3.py`
  - `tests/runtime/test_abi_stability_gate_v3.py`
- Execution history:
  - `docs/M21_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` target `test-abi-stability-v3`
  - `.github/workflows/ci.yml` step `ABI stability v3 gate`

---

## M22: Kernel Reliability + Soak v1

Milestone status: done (2026-03-09).

### Definition of done

- Reliability thresholds are explicitly versioned in runtime docs.
- Soak and fault campaign artifacts are deterministic and machine-readable.
- Kernel reliability v1 gate is release-blocking in local and CI lanes.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/stress/test_kernel_soak_24h_v1.py` | reliability-model tokens and 24h soak threshold schema checks |
| `tests/stress/test_fault_injection_matrix_v1.py` | fault matrix token checks and deterministic recovery thresholds |
| `tests/stress/test_reliability_artifact_schema_v1.py` | seeded determinism and artifact-schema compatibility checks |
| `tests/stress/test_kernel_reliability_gate_v1.py` | make/ci/docs gate wiring and closure |

### Rugo evidence

- Contract docs:
  - `docs/runtime/kernel_reliability_model_v1.md`
- Tooling:
  - `tools/run_kernel_soak_v1.py`
  - `tools/run_fault_campaign_kernel_v1.py`
- Test gate:
  - `tests/stress/test_kernel_soak_24h_v1.py`
  - `tests/stress/test_fault_injection_matrix_v1.py`
  - `tests/stress/test_reliability_artifact_schema_v1.py`
  - `tests/stress/test_kernel_reliability_gate_v1.py`
- Execution history:
  - `docs/M22_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` target `test-kernel-reliability-v1`
  - `.github/workflows/ci.yml` step `Kernel reliability v1 gate`

---

## M23: Hardware Enablement Matrix v3

Milestone status: done (2026-03-09).

### Definition of done

- Hardware matrix v3 contracts are versioned and test-backed.
- Driver lifecycle, suspend/resume, and hotplug baselines are deterministic and
  machine-auditable.
- Firmware attestation v1 is integrated as a required sub-gate.
- Hardware matrix v3 gate and firmware sub-gate are release-blocking in local
  and CI lanes.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/hw/test_hardware_matrix_v3.py` | Tier 0/Tier 1 storage and network deterministic marker checks + v3 contract tokens |
| `tests/hw/test_driver_lifecycle_v3.py` | v3 lifecycle state contract tokens + seeded diagnostics determinism |
| `tests/hw/test_suspend_resume_v1.py` | suspend/resume cycle counters and latency-budget schema checks |
| `tests/hw/test_hotplug_baseline_v1.py` | hotplug event counters and settle-budget schema checks |
| `tests/hw/test_measured_boot_attestation_v1.py` | measured-boot schema/policy verdict checks including missing-PCR rejection |
| `tests/hw/test_tpm_eventlog_schema_v1.py` | TPM event log required-field schema checks |
| `tests/hw/test_hw_gate_v3.py` | matrix v3 make/ci/docs gate wiring and closure |
| `tests/hw/test_firmware_attestation_gate_v1.py` | firmware sub-gate make/ci/docs wiring and attestation evidence checks |

### Rugo evidence

- Contract docs:
  - `docs/hw/support_matrix_v3.md`
  - `docs/hw/driver_lifecycle_contract_v3.md`
  - `docs/hw/firmware_resiliency_policy_v1.md`
  - `docs/security/measured_boot_attestation_v1.md`
- Tooling:
  - `tools/collect_hw_diagnostics_v3.py`
  - `tools/collect_measured_boot_report_v1.py`
- Test gate:
  - `tests/hw/test_hardware_matrix_v3.py`
  - `tests/hw/test_driver_lifecycle_v3.py`
  - `tests/hw/test_suspend_resume_v1.py`
  - `tests/hw/test_hotplug_baseline_v1.py`
  - `tests/hw/test_measured_boot_attestation_v1.py`
  - `tests/hw/test_tpm_eventlog_schema_v1.py`
  - `tests/hw/test_hw_gate_v3.py`
  - `tests/hw/test_firmware_attestation_gate_v1.py`
- Execution history:
  - `docs/M23_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` target `test-hw-matrix-v3`
  - `Makefile` target `test-firmware-attestation-v1`
  - `.github/workflows/ci.yml` step `Hardware matrix v3 gate`
  - `.github/workflows/ci.yml` step `Firmware attestation v1 gate`

---

## M24: Performance Baseline + Regression Budgets v1

Milestone status: done (2026-03-09).

### Definition of done

- Performance budget and benchmark policy contracts are versioned and test-backed.
- Baseline and regression artifacts are deterministic and machine-readable.
- Performance regression v1 gate is release-blocking in local and CI lanes.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/runtime/test_perf_budget_docs_v1.py` | performance budget and benchmark policy contract tokens and ownership checks |
| `tests/runtime/test_perf_regression_v1.py` | seeded baseline/regression determinism and threshold-breach detection checks |
| `tests/runtime/test_perf_gate_v1.py` | make/ci/docs gate wiring and closure checks |

### Rugo evidence

- Contract docs:
  - `docs/runtime/performance_budget_v1.md`
  - `docs/runtime/benchmark_policy_v1.md`
- Tooling:
  - `tools/run_perf_baseline_v1.py`
  - `tools/check_perf_regression_v1.py`
- Test gate:
  - `tests/runtime/test_perf_budget_docs_v1.py`
  - `tests/runtime/test_perf_regression_v1.py`
  - `tests/runtime/test_perf_gate_v1.py`
- Execution history:
  - `docs/M24_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` target `test-perf-regression-v1`
  - `.github/workflows/ci.yml` step `Performance regression v1 gate`

---

## M25: Userspace Service Model + Init v2

Milestone status: done (2026-03-09).

### Definition of done

- Service/init contracts are versioned and test-backed.
- Startup/shutdown/failure/restart semantics are deterministic and bounded.
- Userspace model v2 gate is release-blocking in local and CI lanes.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/runtime/test_service_model_docs_v2.py` | service/init v2 doc contract tokens and gate references |
| `tests/runtime/test_service_lifecycle_v2.py` | deterministic boot-to-operational lifecycle and failure propagation checks |
| `tests/runtime/test_service_dependency_order_v2.py` | topological+lexical ordering and cycle/missing-dependency rejection checks |
| `tests/runtime/test_restart_policy_v2.py` | bounded restart policy cap/backoff semantics checks |
| `tests/runtime/test_userspace_model_gate_v2.py` | make/ci/docs gate wiring and closure checks |

### Rugo evidence

- Contract docs:
  - `docs/runtime/service_model_v2.md`
  - `docs/runtime/init_contract_v2.md`
- Test gate:
  - `tests/runtime/test_service_model_docs_v2.py`
  - `tests/runtime/test_service_lifecycle_v2.py`
  - `tests/runtime/test_service_dependency_order_v2.py`
  - `tests/runtime/test_restart_policy_v2.py`
  - `tests/runtime/test_userspace_model_gate_v2.py`
- Execution history:
  - `docs/M25_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` target `test-userspace-model-v2`
  - `.github/workflows/ci.yml` step `Userspace model v2 gate`

---

## M26: Package/Repo Ecosystem v3

Milestone status: done (2026-03-09).

### Definition of done

- Package/repository v3 policy contracts are versioned and test-backed.
- Rebuild and repository-policy artifacts are deterministic and machine-readable.
- Update trust v1 is integrated as a required sub-gate in local and CI lanes.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/pkg/test_pkg_contract_docs_v3.py` | package/repository v3 contract docs include required policy and gate tokens |
| `tests/pkg/test_pkg_rebuild_repro_v3.py` | seeded deterministic rebuild report generation and mismatch detection checks |
| `tests/pkg/test_repo_policy_v3.py` | repository policy report schema and failure-path enforcement checks |
| `tests/pkg/test_update_trust_docs_v1.py` | update trust and key-rotation policy v1 contract token checks |
| `tests/pkg/test_update_metadata_expiry_v1.py` | metadata-expiry attack remains blocked by trust checker |
| `tests/pkg/test_update_freeze_attack_v1.py` | freeze/replay attack remains blocked by trust checker |
| `tests/pkg/test_update_mix_and_match_v1.py` | mix-and-match metadata attack remains blocked by trust checker |
| `tests/pkg/test_update_key_rotation_v1.py` | deterministic key-rotation drill stages and success checks |
| `tests/pkg/test_update_trust_gate_v1.py` | trust sub-gate make/ci wiring and artifact checks |
| `tests/pkg/test_pkg_ecosystem_gate_v3.py` | package ecosystem gate wiring, sub-gate integration, and closure checks |

### Rugo evidence

- Contract docs:
  - `docs/pkg/package_format_v3.md`
  - `docs/pkg/repository_policy_v3.md`
  - `docs/pkg/update_trust_model_v1.md`
  - `docs/security/update_key_rotation_policy_v1.md`
- Tooling:
  - `tools/repo_policy_check_v3.py`
  - `tools/pkg_rebuild_verify_v3.py`
  - `tools/check_update_trust_v1.py`
  - `tools/run_update_key_rotation_drill_v1.py`
- Test gate:
  - `tests/pkg/test_pkg_contract_docs_v3.py`
  - `tests/pkg/test_pkg_rebuild_repro_v3.py`
  - `tests/pkg/test_repo_policy_v3.py`
  - `tests/pkg/test_update_trust_docs_v1.py`
  - `tests/pkg/test_update_metadata_expiry_v1.py`
  - `tests/pkg/test_update_freeze_attack_v1.py`
  - `tests/pkg/test_update_mix_and_match_v1.py`
  - `tests/pkg/test_update_key_rotation_v1.py`
  - `tests/pkg/test_update_trust_gate_v1.py`
  - `tests/pkg/test_pkg_ecosystem_gate_v3.py`
- Execution history:
  - `docs/M26_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` targets `test-pkg-ecosystem-v3`, `test-update-trust-v1`
  - `.github/workflows/ci.yml` steps `Package ecosystem v3 gate`, `Update trust v1 gate`

---

## M27: External App Compatibility Program v3

Milestone status: done (2026-03-09).

### Definition of done

- Compatibility profile v3 and app-tier taxonomy are versioned and test-backed.
- App-class compatibility reports are deterministic and machine-readable.
- App compatibility v3 gate is release-blocking in local and CI lanes.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/compat/test_app_tier_docs_v1.py` | profile v3 and tier v1 docs expose required contract identifiers and gate hooks |
| `tests/compat/test_cli_suite_v3.py` | deterministic CLI suite thresholds and schema checks |
| `tests/compat/test_runtime_suite_v3.py` | deterministic runtime suite thresholds and ABI mismatch rejection checks |
| `tests/compat/test_service_suite_v3.py` | deterministic service suite thresholds and unsigned/regression rejection checks |
| `tests/compat/test_app_compat_gate_v3.py` | make/ci/docs gate wiring, closure checks, and matrix report schema pass |

### Rugo evidence

- Contract docs:
  - `docs/abi/compat_profile_v3.md`
  - `docs/abi/app_compat_tiers_v1.md`
- Tooling:
  - `tools/run_app_compat_matrix_v3.py`
- Test gate:
  - `tests/compat/test_app_tier_docs_v1.py`
  - `tests/compat/test_cli_suite_v3.py`
  - `tests/compat/test_runtime_suite_v3.py`
  - `tests/compat/test_service_suite_v3.py`
  - `tests/compat/test_app_compat_gate_v3.py`
- Execution history:
  - `docs/M27_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` target `test-app-compat-v3`
  - `.github/workflows/ci.yml` step `App compatibility v3 gate`

---

## M28: Security Hardening Program v3

Milestone status: done (2026-03-09).

### Definition of done

- Hardening profile v3 and threat model v2 are versioned and test-backed.
- Attack-suite and fuzz v2 artifacts are deterministic and machine-readable.
- Vulnerability response v1 is integrated as a required sub-gate in local and
  CI release lanes.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/security/test_hardening_docs_v3.py` | hardening profile and threat model contracts include required gate and schema tokens |
| `tests/security/test_attack_suite_v3.py` | deterministic attack-suite report schema and injected-failure detection checks |
| `tests/security/test_fuzz_gate_v2.py` | deterministic fuzz v2 campaign schema and injected-violation rejection checks |
| `tests/security/test_policy_enforcement_v3.py` | policy identifier alignment and advisory/embargo negative-path enforcement checks |
| `tests/security/test_vuln_response_docs_v1.py` | response/advisory policy contracts expose required IDs and gate wiring |
| `tests/security/test_vuln_triage_sla_v1.py` | triage SLA remains enforced in embargo drill output |
| `tests/security/test_embargo_workflow_v1.py` | embargo workflow stages complete with required success markers |
| `tests/security/test_advisory_schema_v1.py` | advisory lint schema remains valid with zero errors for baseline advisory |
| `tests/security/test_security_hardening_gate_v3.py` | hardening gate make/ci/docs wiring, closure checks, and tool artifact pass |
| `tests/security/test_vuln_response_gate_v1.py` | sub-gate make/ci/docs wiring and advisory/embargo artifact checks |

### Rugo evidence

- Contract docs:
  - `docs/security/hardening_profile_v3.md`
  - `docs/security/threat_model_v2.md`
  - `docs/security/vulnerability_response_policy_v1.md`
  - `docs/security/security_advisory_policy_v1.md`
- Tooling:
  - `tools/run_security_attack_suite_v3.py`
  - `tools/run_security_fuzz_v2.py`
  - `tools/security_advisory_lint_v1.py`
  - `tools/security_embargo_drill_v1.py`
- Test gate:
  - `tests/security/test_hardening_docs_v3.py`
  - `tests/security/test_attack_suite_v3.py`
  - `tests/security/test_fuzz_gate_v2.py`
  - `tests/security/test_policy_enforcement_v3.py`
  - `tests/security/test_vuln_response_docs_v1.py`
  - `tests/security/test_vuln_triage_sla_v1.py`
  - `tests/security/test_embargo_workflow_v1.py`
  - `tests/security/test_advisory_schema_v1.py`
  - `tests/security/test_security_hardening_gate_v3.py`
  - `tests/security/test_vuln_response_gate_v1.py`
- Execution history:
  - `docs/M28_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` targets `test-security-hardening-v3`, `test-vuln-response-v1`
  - `.github/workflows/ci.yml` steps `Security hardening v3 gate`,
    `Vulnerability response v1 gate`

---

## M29: Observability + Diagnostics v2

Milestone status: done (2026-03-09).

### Definition of done

- Observability contract v2 and crash/postmortem contracts are versioned and
  test-backed.
- Trace bundle, diagnostic snapshot, and crash symbolization artifacts are
  deterministic and machine-readable.
- Observability v2 gate and crash-dump v1 sub-gate are required in local and CI
  release lanes.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/runtime/test_observability_docs_v2.py` | observability v2 contract tokens and gate anchors are present |
| `tests/runtime/test_crash_dump_docs_v1.py` | crash dump and triage playbook contracts include required IDs and policy tokens |
| `tests/runtime/test_trace_bundle_v2.py` | deterministic trace bundle schema and injected error-path rejection checks |
| `tests/runtime/test_diag_snapshot_v2.py` | deterministic diagnostic snapshot schema and injected unhealthy-check rejection checks |
| `tests/runtime/test_crash_dump_capture_v1.py` | crash dump schema includes required register/frame/build identifiers |
| `tests/runtime/test_crash_dump_symbolization_v1.py` | symbolization schema and unresolved-frame failure path checks |
| `tests/runtime/test_observability_gate_v2.py` | observability gate make/ci/docs wiring, closure checks, and tool artifact pass |
| `tests/runtime/test_crash_dump_gate_v1.py` | crash-dump sub-gate make/ci/docs wiring and symbolized artifact checks |

### Rugo evidence

- Contract docs:
  - `docs/runtime/observability_contract_v2.md`
  - `docs/runtime/crash_dump_contract_v1.md`
  - `docs/runtime/postmortem_triage_playbook_v1.md`
- Tooling:
  - `tools/collect_trace_bundle_v2.py`
  - `tools/collect_diagnostic_snapshot_v2.py`
  - `tools/collect_crash_dump_v1.py`
  - `tools/symbolize_crash_dump_v1.py`
- Test gate:
  - `tests/runtime/test_observability_docs_v2.py`
  - `tests/runtime/test_trace_bundle_v2.py`
  - `tests/runtime/test_diag_snapshot_v2.py`
  - `tests/runtime/test_observability_gate_v2.py`
  - `tests/runtime/test_crash_dump_docs_v1.py`
  - `tests/runtime/test_crash_dump_capture_v1.py`
  - `tests/runtime/test_crash_dump_symbolization_v1.py`
  - `tests/runtime/test_crash_dump_gate_v1.py`
- Execution history:
  - `docs/M29_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` targets `test-observability-v2`, `test-crash-dump-v1`
  - `.github/workflows/ci.yml` steps `Observability v2 gate`,
    `Crash dump v1 gate`

---

## M30: Installer/Upgrade/Recovery UX v3

Milestone status: done (2026-03-09).

### Definition of done

- Installer and recovery workflow contracts are versioned and test-backed.
- Upgrade and recovery drills emit deterministic, machine-readable artifacts.
- Rollback safety checks are executable, auditable, and release-blocking.
- Ops UX v3 gate is required in local and CI release lanes.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/build/test_installer_ux_v3.py` | installer/recovery v3 docs include required contract IDs and gate anchors |
| `tests/build/test_upgrade_recovery_v3.py` | deterministic upgrade/recovery drill schema and stage sequencing checks |
| `tests/build/test_rollback_safety_v3.py` | rollback-floor and operator-checklist failure-path enforcement checks |
| `tests/build/test_ops_ux_gate_v3.py` | make/ci/docs gate wiring, closure checks, and drill artifact pass |

### Rugo evidence

- Contract docs:
  - `docs/build/installer_ux_v3.md`
  - `docs/build/recovery_workflow_v3.md`
- Tooling:
  - `tools/run_upgrade_drill_v3.py`
  - `tools/run_recovery_drill_v3.py`
- Test gate:
  - `tests/build/test_installer_ux_v3.py`
  - `tests/build/test_upgrade_recovery_v3.py`
  - `tests/build/test_rollback_safety_v3.py`
  - `tests/build/test_ops_ux_gate_v3.py`
- Execution history:
  - `docs/M30_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` target `test-ops-ux-v3`
  - `.github/workflows/ci.yml` step `Ops UX v3 gate`

---

## M31: Release Engineering + Support Lifecycle v2

Milestone status: done (2026-03-09).

### Definition of done

- Release lifecycle and support-window policies are versioned and test-backed.
- Branch and support-window audits are deterministic and machine-readable.
- Supply-chain revalidation and attestation drift checks are required for every
  release candidate.
- Release lifecycle v2 gate and supply-chain revalidation sub-gate are required
  in local and CI release lanes.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/build/test_release_policy_v2_docs.py` | release/support policy docs include required contract IDs, thresholds, and gate anchors |
| `tests/build/test_supply_chain_revalidation_docs_v1.py` | supply-chain/attestation docs include required schema IDs, drift thresholds, and gate anchors |
| `tests/build/test_release_branch_policy_v2.py` | deterministic branch audit schema and missing-required-branch enforcement checks |
| `tests/build/test_support_window_policy_v1.py` | deterministic support window schema and out-of-window rejection checks |
| `tests/build/test_sbom_revalidation_v1.py` | SBOM/provenance revalidation artifact schema and zero-failure baseline checks |
| `tests/build/test_provenance_verification_v1.py` | provenance subject drift detection checks |
| `tests/build/test_attestation_drift_v1.py` | attestation policy mismatch/drift detection checks |
| `tests/build/test_supply_chain_revalidation_gate_v1.py` | supply-chain sub-gate make/ci/docs wiring and artifact checks |
| `tests/build/test_release_lifecycle_gate_v2.py` | lifecycle gate wiring, sub-gate integration, closure checks, and audit artifact pass |

### Rugo evidence

- Contract docs:
  - `docs/build/release_policy_v2.md`
  - `docs/build/support_lifecycle_policy_v1.md`
  - `docs/build/supply_chain_revalidation_policy_v1.md`
  - `docs/build/release_attestation_policy_v1.md`
- Tooling:
  - `tools/release_branch_audit_v2.py`
  - `tools/support_window_audit_v1.py`
  - `tools/verify_sbom_provenance_v2.py`
  - `tools/verify_release_attestations_v1.py`
- Test gate:
  - `tests/build/test_release_policy_v2_docs.py`
  - `tests/build/test_supply_chain_revalidation_docs_v1.py`
  - `tests/build/test_release_branch_policy_v2.py`
  - `tests/build/test_support_window_policy_v1.py`
  - `tests/build/test_sbom_revalidation_v1.py`
  - `tests/build/test_provenance_verification_v1.py`
  - `tests/build/test_attestation_drift_v1.py`
  - `tests/build/test_supply_chain_revalidation_gate_v1.py`
  - `tests/build/test_release_lifecycle_gate_v2.py`
- Execution history:
  - `docs/M31_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` targets `test-release-lifecycle-v2`,
    `test-supply-chain-revalidation-v1`
  - `.github/workflows/ci.yml` steps `Release lifecycle v2 gate`,
    `Supply-chain revalidation v1 gate`

---

## M32: Conformance + Profile Qualification v1

Milestone status: done (2026-03-09).

### Definition of done

- Profile conformance requirements are explicit, versioned, and test-backed.
- Profile qualification artifacts are deterministic and machine-readable.
- Conformance v1 gate is required in local and CI release lanes.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/runtime/test_profile_conformance_docs_v1.py` | profile conformance docs include required policy IDs, profile IDs, thresholds, and gate anchors |
| `tests/runtime/test_server_profile_v1.py` | deterministic server profile qualification and targeted failure-path enforcement |
| `tests/runtime/test_dev_profile_v1.py` | deterministic developer profile qualification and regression-failure enforcement |
| `tests/runtime/test_conformance_gate_v1.py` | make/ci/docs gate wiring, closure checks, and conformance artifact pass |

### Rugo evidence

- Contract docs:
  - `docs/runtime/profile_conformance_v1.md`
- Tooling:
  - `tools/run_conformance_suite_v1.py`
- Test gate:
  - `tests/runtime/test_profile_conformance_docs_v1.py`
  - `tests/runtime/test_server_profile_v1.py`
  - `tests/runtime/test_dev_profile_v1.py`
  - `tests/runtime/test_conformance_gate_v1.py`
- Execution history:
  - `docs/M32_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` target `test-conformance-v1`
  - `.github/workflows/ci.yml` step `Conformance v1 gate`

---

## M33: Fleet-Scale Operations Baseline v1

Milestone status: done (2026-03-09).

### Definition of done

- Fleet update/health and rollout-safety requirements are explicit, versioned,
  and test-backed.
- Fleet simulation artifacts are deterministic and machine-readable.
- Fleet ops gate and rollout-safety sub-gate are required in local and CI lanes.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/pkg/test_fleet_policy_docs_v1.py` | fleet update/health docs include required policy IDs, schemas, thresholds, and gate anchors |
| `tests/pkg/test_rollout_policy_docs_v1.py` | rollout/canary SLO docs include stage budgets, SLO fields, and halt/rollback requirements |
| `tests/pkg/test_fleet_update_sim_v1.py` | deterministic fleet update simulation pass/fail behavior and rollback coordination evidence |
| `tests/runtime/test_fleet_health_sim_v1.py` | deterministic fleet health simulation pass/fail behavior and degraded-state detection |
| `tests/pkg/test_canary_rollout_sim_v1.py` | deterministic staged canary rollout behavior and automatic halt on injected stage failure |
| `tests/runtime/test_rollout_abort_policy_v1.py` | deterministic SLO-triggered abort drill enforcement and missing-halt failure detection |
| `tests/runtime/test_fleet_rollout_safety_gate_v1.py` | rollout-safety make/ci/docs wiring, closure checks, and artifact pass |
| `tests/runtime/test_fleet_ops_gate_v1.py` | fleet ops gate wiring, sub-gate integration, closure checks, and artifact pass |

### Rugo evidence

- Contract docs:
  - `docs/pkg/fleet_update_policy_v1.md`
  - `docs/runtime/fleet_health_policy_v1.md`
  - `docs/pkg/staged_rollout_policy_v1.md`
  - `docs/runtime/canary_slo_policy_v1.md`
- Tooling:
  - `tools/run_fleet_update_sim_v1.py`
  - `tools/run_fleet_health_sim_v1.py`
  - `tools/run_canary_rollout_sim_v1.py`
  - `tools/run_rollout_abort_drill_v1.py`
- Test gate:
  - `tests/pkg/test_fleet_policy_docs_v1.py`
  - `tests/pkg/test_rollout_policy_docs_v1.py`
  - `tests/pkg/test_fleet_update_sim_v1.py`
  - `tests/runtime/test_fleet_health_sim_v1.py`
  - `tests/pkg/test_canary_rollout_sim_v1.py`
  - `tests/runtime/test_rollout_abort_policy_v1.py`
  - `tests/runtime/test_fleet_rollout_safety_gate_v1.py`
  - `tests/runtime/test_fleet_ops_gate_v1.py`
- Execution history:
  - `docs/M33_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` targets `test-fleet-ops-v1`,
    `test-fleet-rollout-safety-v1`
  - `.github/workflows/ci.yml` steps `Fleet ops v1 gate`,
    `Fleet rollout safety v1 gate`

---

## M34: Maturity Qualification + LTS Declaration

Milestone status: done (2026-03-09).

### Definition of done

- Maturity qualification and LTS declaration requirements are explicit,
  versioned, and test-backed.
- Final qualification bundle is deterministic and machine-readable.
- Maturity qualification v1 gate is required in local and CI release lanes.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/build/test_maturity_docs_v1.py` | maturity/LTS docs include required policy IDs, schemas, thresholds, and gate anchors |
| `tests/build/test_maturity_qualification_v1.py` | deterministic maturity qualification bundle pass/fail behavior and LTS eligibility checks |
| `tests/build/test_lts_policy_v1.py` | LTS declaration policy enforcement checks and short-history rejection behavior |
| `tests/build/test_maturity_security_response_drill_v1.py` | security advisory/embargo drill evidence is present in maturity lane |
| `tests/build/test_maturity_supply_chain_continuity_v1.py` | supply-chain continuity evidence remains deterministic and auditable |
| `tests/build/test_maturity_rollout_safety_v1.py` | rollout safety evidence remains deterministic and policy-enforced |
| `tests/build/test_maturity_gate_v1.py` | make/ci/docs gate wiring, closure checks, and qualification artifact pass |

### Rugo evidence

- Contract docs:
  - `docs/build/maturity_qualification_v1.md`
  - `docs/build/lts_declaration_policy_v1.md`
- Tooling:
  - `tools/run_maturity_qualification_v1.py`
- Test gate:
  - `tests/build/test_maturity_docs_v1.py`
  - `tests/build/test_maturity_qualification_v1.py`
  - `tests/build/test_lts_policy_v1.py`
  - `tests/build/test_maturity_security_response_drill_v1.py`
  - `tests/build/test_maturity_supply_chain_continuity_v1.py`
  - `tests/build/test_maturity_rollout_safety_v1.py`
  - `tests/build/test_maturity_gate_v1.py`
- Execution history:
  - `docs/M34_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` target `test-maturity-qual-v1`
  - `.github/workflows/ci.yml` step `Maturity qualification v1 gate`

---

## M35: Desktop + Interactive UX Baseline v1

Milestone status: done (2026-03-09).

### Definition of done

- Desktop/session/input baseline contracts are explicit, versioned, and
  test-backed.
- Desktop smoke and GUI compatibility artifacts are deterministic and
  machine-readable.
- Desktop gate and GUI compatibility sub-gate are required in local and CI
  release lanes.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/desktop/test_desktop_docs_v1.py` | desktop/display/window/input contract docs include required IDs, thresholds, schemas, and gate anchors |
| `tests/desktop/test_display_session_v1.py` | deterministic display/session smoke behavior and display failure-path enforcement |
| `tests/desktop/test_input_baseline_v1.py` | deterministic keyboard/pointer baseline checks and injected regression rejection |
| `tests/desktop/test_window_lifecycle_v1.py` | deterministic create/map/focus/close lifecycle checks and failure-path enforcement |
| `tests/desktop/test_gui_app_compat_v1.py` | deterministic GUI app matrix schema and class threshold enforcement checks |
| `tests/desktop/test_gui_app_compat_gate_v1.py` | GUI sub-gate make/ci/docs wiring and matrix artifact pass |
| `tests/desktop/test_desktop_gate_v1.py` | desktop gate wiring, sub-gate integration, closure checks, and deterministic artifact pass |

### Rugo evidence

- Contract docs:
  - `docs/desktop/display_stack_contract_v1.md`
  - `docs/desktop/window_manager_contract_v1.md`
  - `docs/desktop/input_stack_contract_v1.md`
  - `docs/desktop/desktop_profile_v1.md`
- Tooling:
  - `tools/run_desktop_smoke_v1.py`
  - `tools/run_gui_app_matrix_v1.py`
- Test gate:
  - `tests/desktop/test_desktop_docs_v1.py`
  - `tests/desktop/test_display_session_v1.py`
  - `tests/desktop/test_input_baseline_v1.py`
  - `tests/desktop/test_window_lifecycle_v1.py`
  - `tests/desktop/test_gui_app_compat_v1.py`
  - `tests/desktop/test_gui_app_compat_gate_v1.py`
  - `tests/desktop/test_desktop_gate_v1.py`
- Execution history:
  - `docs/M35_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` targets `test-desktop-stack-v1`, `test-gui-app-compat-v1`
  - `.github/workflows/ci.yml` steps `Desktop stack v1 gate`,
    `GUI app compatibility v1 gate`

---

## M36: Compatibility Surface Expansion v1

Milestone status: done (2026-03-09).

### Definition of done

- Compatibility profile v4 and related process/socket contracts are explicit,
  versioned, and test-backed.
- Compatibility surface and POSIX gap artifacts are deterministic and
  machine-readable.
- Compatibility surface gate and POSIX sub-gate are required in local and CI
  release lanes.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/compat/test_compat_docs_v4.py` | compatibility/process/socket docs include required IDs, schemas, and gate anchors |
| `tests/compat/test_posix_gap_closure_v1.py` | deterministic POSIX gap report schema and required/deferred pass-fail behavior |
| `tests/compat/test_process_model_v3.py` | deterministic process model v3 campaign semantics and regression rejection |
| `tests/compat/test_socket_family_expansion_v1.py` | deterministic socket-family expansion checks and required API coverage |
| `tests/compat/test_deferred_surface_behavior_v1.py` | deterministic deferred-surface unsupported behavior and violation rejection |
| `tests/compat/test_posix_gap_closure_gate_v1.py` | POSIX sub-gate make/ci/docs wiring and artifact checks |
| `tests/compat/test_compat_surface_gate_v1.py` | compatibility gate wiring, sub-gate integration, closure checks, and deterministic artifacts |

### Rugo evidence

- Contract docs:
  - `docs/abi/compat_profile_v4.md`
  - `docs/runtime/syscall_coverage_matrix_v3.md`
  - `docs/abi/process_model_v3.md`
  - `docs/abi/socket_family_expansion_v1.md`
- Tooling:
  - `tools/run_compat_surface_campaign_v1.py`
  - `tools/run_posix_gap_report_v1.py`
- Test gate:
  - `tests/compat/test_compat_docs_v4.py`
  - `tests/compat/test_posix_gap_closure_v1.py`
  - `tests/compat/test_process_model_v3.py`
  - `tests/compat/test_socket_family_expansion_v1.py`
  - `tests/compat/test_deferred_surface_behavior_v1.py`
  - `tests/compat/test_posix_gap_closure_gate_v1.py`
  - `tests/compat/test_compat_surface_gate_v1.py`
- Execution history:
  - `docs/M36_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` targets `test-compat-surface-v1`, `test-posix-gap-closure-v1`
  - `.github/workflows/ci.yml` steps `Compatibility surface v1 gate`,
    `POSIX gap closure v1 gate`

---

## M37: Hardware Breadth + Driver Matrix v4

Milestone status: done (2026-03-09).

### Definition of done

- Matrix v4, driver lifecycle v4, and bare-metal promotion policy contracts are
  explicit, versioned, and test-backed.
- Matrix and promotion artifacts are deterministic and machine-readable.
- Hardware matrix v4 gate and bare-metal promotion sub-gate are required in
  local and CI release lanes.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/hw/test_hw_matrix_docs_v4.py` | matrix/lifecycle/promotion docs include required IDs, tiers, thresholds, and gate anchors |
| `tests/hw/test_hw_matrix_v4.py` | deterministic matrix v4 schema and device-class coverage checks with tier smoke markers |
| `tests/hw/test_driver_lifecycle_v4.py` | deterministic lifecycle v4 state coverage and injected runtime regression rejection |
| `tests/hw/test_baremetal_promotion_v1.py` | deterministic promotion report schema and artifact/threshold policy enforcement |
| `tests/hw/test_hw_negative_paths_v4.py` | deterministic `BLK: not found` / `NET: not found` negative-path checks and regression rejection |
| `tests/hw/test_hw_baremetal_promotion_gate_v1.py` | promotion sub-gate make/ci/docs wiring and promotion artifact pass |
| `tests/hw/test_hw_gate_v4.py` | hardware gate wiring, sub-gate integration, closure checks, and deterministic artifacts |

### Rugo evidence

- Contract docs:
  - `docs/hw/support_matrix_v4.md`
  - `docs/hw/driver_lifecycle_contract_v4.md`
  - `docs/hw/bare_metal_promotion_policy_v1.md`
- Tooling:
  - `tools/run_hw_matrix_v4.py`
  - `tools/collect_hw_promotion_evidence_v1.py`
- Test gate:
  - `tests/hw/test_hw_matrix_docs_v4.py`
  - `tests/hw/test_hw_matrix_v4.py`
  - `tests/hw/test_driver_lifecycle_v4.py`
  - `tests/hw/test_baremetal_promotion_v1.py`
  - `tests/hw/test_hw_negative_paths_v4.py`
  - `tests/hw/test_hw_baremetal_promotion_gate_v1.py`
  - `tests/hw/test_hw_gate_v4.py`
- Execution history:
  - `docs/M37_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` targets `test-hw-matrix-v4`, `test-hw-baremetal-promotion-v1`
  - `.github/workflows/ci.yml` steps `Hardware matrix v4 gate`,
    `Hardware bare-metal promotion v1 gate`

---

## M38: Storage + Platform Feature Expansion v1

Milestone status: done (2026-03-09).

### Definition of done

- Storage/platform feature contracts for snapshots, online resize, advanced
  filesystem operations, and platform profiles are explicit, versioned, and
  test-backed.
- Feature campaign and platform conformance artifacts are deterministic and
  machine-readable.
- Storage platform gate and storage feature contract sub-gate are required in
  local and CI release lanes.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/storage/test_storage_feature_docs_v1.py` | storage/platform feature contract docs include required IDs, thresholds, schemas, and gate anchors |
| `tests/storage/test_snapshot_semantics_v1.py` | deterministic snapshot campaign schema and restore-integrity failure-path enforcement |
| `tests/storage/test_online_resize_v1.py` | deterministic online resize campaign semantics and shrink-guard regression rejection |
| `tests/storage/test_advanced_fs_ops_v1.py` | deterministic advanced fs operation semantics and xattr/reflink policy enforcement |
| `tests/runtime/test_platform_feature_profile_v1.py` | deterministic platform profile schema and profile requirement enforcement |
| `tests/storage/test_storage_feature_contract_gate_v1.py` | storage feature contract sub-gate make/ci/docs wiring and conformance artifact pass |
| `tests/storage/test_storage_platform_gate_v1.py` | storage platform gate wiring, sub-gate integration, closure checks, and deterministic artifacts |

### Rugo evidence

- Contract docs:
  - `docs/storage/fs_feature_contract_v1.md`
  - `docs/storage/snapshot_policy_v1.md`
  - `docs/storage/online_resize_policy_v1.md`
  - `docs/runtime/platform_feature_profile_v1.md`
- Tooling:
  - `tools/run_storage_feature_campaign_v1.py`
  - `tools/run_platform_feature_conformance_v1.py`
- Test gate:
  - `tests/storage/test_storage_feature_docs_v1.py`
  - `tests/storage/test_snapshot_semantics_v1.py`
  - `tests/storage/test_online_resize_v1.py`
  - `tests/storage/test_advanced_fs_ops_v1.py`
  - `tests/runtime/test_platform_feature_profile_v1.py`
  - `tests/storage/test_storage_feature_contract_gate_v1.py`
  - `tests/storage/test_storage_platform_gate_v1.py`
- Execution history:
  - `docs/M38_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` targets `test-storage-platform-v1`,
    `test-storage-feature-contract-v1`
  - `.github/workflows/ci.yml` steps `Storage platform v1 gate`,
    `Storage feature contract v1 gate`

---

## M39: Ecosystem Scale + Distribution Workflow v1

Milestone status: done (2026-03-09).

### Definition of done

- Ecosystem scale, catalog quality, and distribution workflow contracts are
  explicit, versioned, and test-backed.
- App catalog simulation, install-success campaign, and reproducibility-audit
  artifacts are deterministic and machine-readable.
- Ecosystem scale gate and app-catalog-health sub-gate are required in local
  and CI release lanes.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/pkg/test_ecosystem_scale_docs_v1.py` | ecosystem-scale/catalog-quality/distribution docs include required IDs, thresholds, schemas, and gate anchors |
| `tests/pkg/test_app_catalog_sim_v1.py` | deterministic catalog simulation schema and class/quality threshold enforcement |
| `tests/pkg/test_pkg_install_success_rate_v1.py` | deterministic install-success campaign schema and success-rate regression rejection |
| `tests/pkg/test_catalog_reproducibility_v1.py` | deterministic reproducibility audit schema and rebuild/provenance failure-path enforcement |
| `tests/pkg/test_distribution_workflow_v1.py` | deterministic workflow-stage, signoff, rollback, and replication-lag checks |
| `tests/pkg/test_app_catalog_health_gate_v1.py` | app-catalog-health sub-gate make/ci/docs wiring and install/audit artifact pass |
| `tests/pkg/test_ecosystem_scale_gate_v1.py` | ecosystem-scale gate wiring, sub-gate integration, closure checks, and deterministic artifacts |

### Rugo evidence

- Contract docs:
  - `docs/pkg/ecosystem_scale_policy_v1.md`
  - `docs/pkg/catalog_quality_contract_v1.md`
  - `docs/pkg/distribution_workflow_v1.md`
- Tooling:
  - `tools/run_app_catalog_sim_v1.py`
  - `tools/run_pkg_install_success_campaign_v1.py`
  - `tools/run_reproducible_catalog_audit_v1.py`
- Test gate:
  - `tests/pkg/test_ecosystem_scale_docs_v1.py`
  - `tests/pkg/test_app_catalog_sim_v1.py`
  - `tests/pkg/test_pkg_install_success_rate_v1.py`
  - `tests/pkg/test_catalog_reproducibility_v1.py`
  - `tests/pkg/test_distribution_workflow_v1.py`
  - `tests/pkg/test_app_catalog_health_gate_v1.py`
  - `tests/pkg/test_ecosystem_scale_gate_v1.py`
- Execution history:
  - `docs/M39_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` targets `test-ecosystem-scale-v1`,
    `test-app-catalog-health-v1`
  - `.github/workflows/ci.yml` steps `Ecosystem scale v1 gate`,
    `App catalog health v1 gate`

---

## M40: Runtime-Backed Evidence Integrity v1

Milestone status: done (2026-03-10).

### Definition of done

- Evidence integrity, runtime evidence schema, and provenance audit policy
  contracts are explicit, versioned, and test-backed.
- Runtime evidence collection and gate audit artifacts are deterministic,
  machine-readable, and trace-linked.
- Evidence integrity gate and synthetic-evidence-ban sub-gate are required in
  local and CI release lanes.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/runtime/test_evidence_integrity_docs_v1.py` | evidence-integrity/runtime-schema/gate-provenance docs include required IDs, thresholds, schemas, and gate anchors |
| `tests/runtime/test_runtime_evidence_collection_v1.py` | deterministic runtime evidence schema, provenance metrics, and synthetic failure-path rejection |
| `tests/runtime/test_gate_evidence_audit_v1.py` | deterministic audit schema and synthetic-only evidence rejection path |
| `tests/runtime/test_evidence_trace_linkage_v1.py` | runtime evidence items link to traces with matching lane and digest |
| `tests/runtime/test_synthetic_evidence_ban_v1.py` | synthetic-ban sub-gate make/ci/docs wiring and enforced synthetic-only rejection |
| `tests/runtime/test_evidence_integrity_gate_v1.py` | evidence-integrity gate wiring, sub-gate integration, closure checks, and deterministic artifacts |

### Rugo evidence

- Contract docs:
  - `docs/runtime/evidence_integrity_policy_v1.md`
  - `docs/runtime/runtime_evidence_schema_v1.md`
  - `docs/runtime/gate_provenance_policy_v1.md`
- Tooling:
  - `tools/collect_runtime_evidence_v1.py`
  - `tools/audit_gate_evidence_v1.py`
- Test gate:
  - `tests/runtime/test_evidence_integrity_docs_v1.py`
  - `tests/runtime/test_runtime_evidence_collection_v1.py`
  - `tests/runtime/test_gate_evidence_audit_v1.py`
  - `tests/runtime/test_evidence_trace_linkage_v1.py`
  - `tests/runtime/test_synthetic_evidence_ban_v1.py`
  - `tests/runtime/test_evidence_integrity_gate_v1.py`
- Execution history:
  - `docs/M40_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` targets `test-evidence-integrity-v1`,
    `test-synthetic-evidence-ban-v1`
  - `.github/workflows/ci.yml` steps `Evidence integrity v1 gate`,
    `Synthetic evidence ban v1 gate`

---

## M41: Process + Readiness Compatibility Closure v1

Milestone status: done (2026-03-10).

### Definition of done

- Compatibility profile v5 plus process/readiness contracts are explicit,
  versioned, and test-backed.
- Process/readiness compatibility campaign and POSIX gap artifacts are
  deterministic and machine-readable.
- Process/readiness parity gate and POSIX closure sub-gate are required in
  local and CI release lanes.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/compat/test_compat_docs_v5.py` | compatibility/process/readiness docs include required IDs, schemas, thresholds, and gate anchors |
| `tests/compat/test_fork_clone_surface_v1.py` | deterministic fork/clone deferred-surface behavior and violation rejection |
| `tests/compat/test_epoll_surface_v1.py` | deterministic readiness/epoll deferred behavior and readiness regression rejection |
| `tests/compat/test_process_model_v4.py` | deterministic process model v4 campaign semantics and regression rejection |
| `tests/compat/test_deferred_surface_behavior_v2.py` | deterministic deferred-surface unsupported behavior and violation rejection |
| `tests/compat/test_posix_gap_closure_gate_v2.py` | POSIX sub-gate make/ci/docs wiring and artifact checks |
| `tests/compat/test_process_readiness_gate_v1.py` | process/readiness gate wiring, sub-gate integration, closure checks, and deterministic artifacts |

### Rugo evidence

- Contract docs:
  - `docs/abi/compat_profile_v5.md`
  - `docs/runtime/syscall_coverage_matrix_v4.md`
  - `docs/abi/process_model_v4.md`
  - `docs/abi/readiness_io_model_v1.md`
- Tooling:
  - `tools/run_compat_surface_campaign_v2.py`
  - `tools/run_posix_gap_report_v2.py`
- Test gate:
  - `tests/compat/test_compat_docs_v5.py`
  - `tests/compat/test_fork_clone_surface_v1.py`
  - `tests/compat/test_epoll_surface_v1.py`
  - `tests/compat/test_process_model_v4.py`
  - `tests/compat/test_deferred_surface_behavior_v2.py`
  - `tests/compat/test_posix_gap_closure_gate_v2.py`
  - `tests/compat/test_process_readiness_gate_v1.py`
- Execution history:
  - `docs/M41_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` targets `test-process-readiness-parity-v1`,
    `test-posix-gap-closure-v2`
  - `.github/workflows/ci.yml` steps `Process readiness parity v1 gate`,
    `POSIX gap closure v2 gate`

---

## M42: Isolation + Namespace Baseline v1

Milestone status: done (2026-03-10).

### Definition of done

- Namespace/cgroup baseline and isolation/resource-control contracts are
  explicit, versioned, and test-backed.
- Isolation and resource-control campaign artifacts are deterministic and
  machine-readable.
- Isolation baseline gate and namespace/cgroup sub-gate are required in local
  and CI release lanes.

### Acceptance tests

| Test | Markers/Outcome |
|------|------------------|
| `tests/security/test_isolation_docs_v1.py` | namespace/cgroup, isolation profile, and resource-control docs include required IDs, schemas, thresholds, and gate anchors |
| `tests/security/test_namespace_baseline_v1.py` | deterministic namespace baseline schema and regression rejection |
| `tests/security/test_cgroup_baseline_v1.py` | deterministic cgroup baseline checks and memory-limit regression rejection |
| `tests/security/test_isolation_escape_negative_v1.py` | deterministic escape negative-path checks and denial-regression rejection |
| `tests/runtime/test_resource_control_policy_v1.py` | deterministic resource-control policy schema and controller-drift rejection |
| `tests/security/test_namespace_cgroup_gate_v1.py` | namespace/cgroup sub-gate make/ci/docs wiring and resource-control artifact checks |
| `tests/security/test_isolation_gate_v1.py` | isolation gate wiring, sub-gate integration, closure checks, and deterministic artifacts |

### Rugo evidence

- Contract docs:
  - `docs/abi/namespace_cgroup_contract_v1.md`
  - `docs/security/isolation_profile_v1.md`
  - `docs/runtime/resource_control_policy_v1.md`
- Tooling:
  - `tools/run_isolation_campaign_v1.py`
  - `tools/run_resource_control_campaign_v1.py`
- Test gate:
  - `tests/security/test_isolation_docs_v1.py`
  - `tests/security/test_namespace_baseline_v1.py`
  - `tests/security/test_cgroup_baseline_v1.py`
  - `tests/security/test_isolation_escape_negative_v1.py`
  - `tests/runtime/test_resource_control_policy_v1.py`
  - `tests/security/test_namespace_cgroup_gate_v1.py`
  - `tests/security/test_isolation_gate_v1.py`
- Execution history:
  - `docs/M42_EXECUTION_BACKLOG.md`
- Release gating:
  - `Makefile` targets `test-isolation-baseline-v1`,
    `test-namespace-cgroup-v1`
  - `.github/workflows/ci.yml` steps `Isolation baseline v1 gate`,
    `Namespace cgroup v1 gate`

---

## References

- Rust target `x86_64-unknown-none` platform support notes (no std; allocator
  required for alloc).
- Go porting policy (ports are OS/arch; avoid broken/incomplete ports).
- TinyGo documentation (targets; language and stdlib support notes).
- Build toolchain details: `docs/build/rust_toolchain.md`
- Legacy Go toolchain: `docs/build/go_toolchain.md`


