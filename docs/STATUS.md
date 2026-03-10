# Rugo — Status

## What this repo contains

| Track | Language | Location | Role |
|-------|----------|----------|------|
| **Rugo** | Rust (no_std) | repo root | Default build. Production rewrite target. |
| **Legacy** | C + Go (gccgo) | `legacy/` | Completed reference baseline (read-only). |

Full milestone definitions and acceptance criteria live in
[MILESTONES.md](../MILESTONES.md).

## How to run

```bash
# Rugo (build + full QEMU smoke-test suite)
make test-qemu
make test-runtime-maturity
make test-process-scheduler-v2
make test-compat-v2
make test-network-stack-v1
make test-network-stack-v2
make test-storage-reliability-v1
make test-storage-reliability-v2
make test-release-engineering-v1
make test-release-ops-v2
make test-abi-stability-v3
make test-kernel-reliability-v1
make test-hw-matrix-v3
make test-firmware-attestation-v1
make test-perf-regression-v1
make test-userspace-model-v2
make test-pkg-ecosystem-v3
make test-update-trust-v1
make test-app-compat-v3
make test-security-hardening-v3
make test-vuln-response-v1
make test-observability-v2
make test-crash-dump-v1
make test-ops-ux-v3
make test-desktop-stack-v1
make test-gui-app-compat-v1
make test-compat-surface-v1
make test-posix-gap-closure-v1
make test-hw-matrix-v4
make test-hw-baremetal-promotion-v1
make test-ecosystem-scale-v1
make test-app-catalog-health-v1
make test-evidence-integrity-v1
make test-synthetic-evidence-ban-v1
make test-process-readiness-parity-v1
make test-posix-gap-closure-v2

# Legacy (build + QEMU smoke tests, 16 tests)
make -C legacy build && make -C legacy image && make -C legacy test-qemu

# Cross-platform via Docker
make docker-all          # Rugo only
make docker-legacy       # Legacy only (requires gccgo in Docker image)
```

## Status matrix

| Milestone | Legacy | Rugo | Evidence |
|-----------|--------|------|----------|
| **M0** Boot + serial | ✅ | ✅ | `tests/boot/test_boot_banner.py` (`RUGO: boot ok`, `RUGO: halt ok`), `tests/boot/test_panic_path.py` (`RUGO: panic code=`). Legacy: `legacy/tests/boot/test_boot_banner.py` |
| **M1** Paging + traps | ✅ | ✅ | Rugo: `tests/boot/test_paging_enabled.py` (`MM: paging=on`), `tests/trap/test_page_fault_report.py` (`PF: addr=0x...`), `tests/trap/test_idt_smoke.py` (`TRAP: ok`). Legacy: `legacy/tests/boot/test_paging_enabled.py`, `legacy/tests/trap/test_page_fault_report.py` |
| **M2** Scheduler + threads | ✅ | ✅ | Rugo: `tests/sched/test_timer_ticks.py` (`TICK: 100`), `tests/sched/test_two_threads.py` (interleaved `A`/`B`). Legacy: `legacy/tests/sched/test_timer_ticks.py`, `legacy/tests/sched/test_two_threads.py` |
| **M3** User mode + syscalls | ✅ | ✅ | Rugo: `tests/user/test_enter_user_mode.py` (`USER: hello`), `tests/user/test_syscall_roundtrip.py` (`SYSCALL: ok`), `tests/user/test_thread_exit.py` (`THREAD_EXIT: ok`), `tests/user/test_user_fault.py` (`USER: killed`). Legacy: `legacy/tests/user/` |
| **M4** IPC + shared memory | ✅ | ✅ | Rugo: `tests/ipc/test_ping_pong.py` (`PING: ok`, `PONG: ok`), `tests/ipc/test_shm_bulk.py` (`SHM: checksum ok`). Legacy: `legacy/tests/ipc/` |
| **M5** VirtIO block | ✅ | ✅ | Rugo: `tests/drivers/test_virtio_blk_identify.py` (`BLK: found virtio-blk`), `tests/drivers/test_virtio_blk_rw.py` (`BLK: rw ok`), `tests/drivers/test_virtio_blk_init_invariants.py` (`BLK: invariants ok`), `tests/drivers/test_virtio_blk_init_failure.py` (`BLK: init failed`). Legacy: `legacy/tests/drivers/` |
| **M6** Filesystem + pkg + shell | ✅ | ✅ | Rugo: `tests/fs/test_fsd_smoke.py` (`FSD: mount ok`), `tests/pkg/test_pkg_install_run.py` (`APP: hello world`). Legacy: `legacy/tests/fs/test_fsd_smoke.py`, `legacy/tests/pkg/test_pkg_install_run.py` |
| **M7** VirtIO net + UDP | ✅ | ✅ | Rugo: `tests/net/test_udp_echo.py` (`NET: udp echo`). Legacy: `legacy/tests/net/test_udp_echo.py` |
| **G0** Go kernel entry | ✅ | n/a | `legacy/tests/boot/test_go_entry.py` (`GO: kmain ok`). Legacy-only. Re-verified 2026-02-18 via Docker. |
| **G1** Go services (TinyGo) | n/a | ✅ | Rugo: `tests/go/test_go_user_service.py` (`GOUSR: ok`). TinyGo bare-metal x86_64. |
| **G2** Full Go port | n/a | ✅ | Rugo-only. Done: `tests/go/test_std_go_binary.py` on `go_std_test` stock-Go artifact path (`os-go-std.iso`, `GOSTD: ok`). |
| **M8** Compatibility Profile v1 | n/a | ✅ | Rugo: `tests/compat/*`, `tests/pkg/test_pkg_external_apps.py`; docs in `docs/abi/*` and `docs/M8_EXECUTION_BACKLOG.md`. |
| **M9** Hardware enablement matrix v1 | n/a | ✅ | Rugo: `tests/hw/*`, `make test-hw-matrix`, CI `Hardware matrix v1 gate`, docs in `docs/hw/*` and `docs/M9_EXECUTION_BACKLOG.md`. |
| **M10** Security baseline v1 | n/a | ✅ | Rugo: `tests/security/*`, `make test-security-baseline`, CI `Security baseline v1 gate`, docs in `docs/security/*` and `docs/M10_EXECUTION_BACKLOG.md`. |
| **M11** Runtime + toolchain maturity v1 | n/a | ✅ | Rugo: `tests/runtime/*`, `make test-runtime-maturity`, CI `Runtime + toolchain maturity v1 gate`, docs in `docs/runtime/*` and `docs/M11_EXECUTION_BACKLOG.md`. |
| **M12** Network stack v1 | n/a | ✅ | Rugo: `tests/net/*`, `make test-network-stack-v1`, CI `Network stack v1 gate`, docs in `docs/net/*` and `docs/M12_EXECUTION_BACKLOG.md`. |
| **M13** Storage reliability v1 | n/a | ✅ | Rugo: `tests/storage/*`, `make test-storage-reliability-v1`, CI `Storage reliability v1 gate`, docs in `docs/storage/*` and `docs/M13_EXECUTION_BACKLOG.md`. |
| **M14** Productization + release engineering v1 | n/a | done | Rugo: `tests/build/*`, update tests in `tests/pkg/*`, `make test-release-engineering-v1`, CI `Release engineering v1 gate`, docs in `docs/build/*`, `docs/pkg/*`, and `docs/M14_EXECUTION_BACKLOG.md`. |
| **M15** Hardware Enablement Matrix v2 | n/a | done | Rugo: `tests/hw/*_v2`, `make test-hw-matrix-v2`, CI `Hardware matrix v2 gate`, docs in `docs/hw/*_v2` and `docs/M15_EXECUTION_BACKLOG.md`. |
| **M16** Process + Scheduler Model v2 | n/a | done | Rugo: `tests/sched/*_v2`, `tests/user/*_v2`, `make test-process-scheduler-v2`, CI `Process scheduler v2 gate`, docs in `docs/abi/*_v2` and `docs/M16_EXECUTION_BACKLOG.md`. |
| **M17** Compatibility Profile v2 | n/a | done | Rugo: `tests/compat/*_v2` + tier gate tests, `make test-compat-v2`, CI `Compatibility profile v2 gate`, docs in `docs/abi/*_v2`, `docs/runtime/syscall_coverage_matrix_v2.md`, and `docs/M17_EXECUTION_BACKLOG.md`. |
| **M18** Storage Reliability v2 | n/a | done | Rugo: `tests/storage/*_v2` + storage gate tests, `make test-storage-reliability-v2`, CI `Storage reliability v2 gate`, docs in `docs/storage/*_v2`, and `docs/M18_EXECUTION_BACKLOG.md`. |
| **M19** Network Stack v2 | n/a | done | Rugo: `tests/net/*_v2` + network gate tests, `make test-network-stack-v2`, CI `Network stack v2 gate`, docs in `docs/net/*_v2`, and `docs/M19_EXECUTION_BACKLOG.md`. |
| **M20** Operability + Release UX v2 | n/a | done | Rugo: `tests/build/*_v2` + operability gate tests, `make test-release-ops-v2`, CI `Operability and release UX v2 gate`, docs in `docs/build/*_v2`, `docs/pkg/*_v2`, and `docs/M20_EXECUTION_BACKLOG.md`. |
| **M21** ABI + API Stability Program v3 | n/a | done | Rugo: ABI/API stability docs + compatibility enforcement tests, `make test-abi-stability-v3`, CI `ABI stability v3 gate`, docs in `docs/abi/syscall_v3.md`, `docs/runtime/*`, and `docs/M21_EXECUTION_BACKLOG.md`. |
| **M22** Kernel Reliability + Soak v1 | n/a | done | Rugo: reliability model docs + deterministic soak/fault artifacts, `make test-kernel-reliability-v1`, CI `Kernel reliability v1 gate`, docs in `docs/runtime/kernel_reliability_model_v1.md`, and `docs/M22_EXECUTION_BACKLOG.md`. |
| **M23** Hardware Enablement Matrix v3 | n/a | done | Rugo: matrix v3 docs + deterministic diagnostics and firmware attestation artifacts, `make test-hw-matrix-v3`, `make test-firmware-attestation-v1`, CI `Hardware matrix v3 gate` + `Firmware attestation v1 gate`, docs in `docs/hw/*_v3`, `docs/security/measured_boot_attestation_v1.md`, and `docs/M23_EXECUTION_BACKLOG.md`. |
| **M24** Performance Baseline + Regression Budgets v1 | n/a | done | Rugo: performance budget/policy docs + deterministic baseline/regression artifacts, `make test-perf-regression-v1`, CI `Performance regression v1 gate`, docs in `docs/runtime/performance_budget_v1.md`, `docs/runtime/benchmark_policy_v1.md`, and `docs/M24_EXECUTION_BACKLOG.md`. |
| **M25** Userspace Service Model + Init v2 | n/a | done | Rugo: service/init v2 contract docs + deterministic lifecycle/dependency/restart checks, `make test-userspace-model-v2`, CI `Userspace model v2 gate`, docs in `docs/runtime/service_model_v2.md`, `docs/runtime/init_contract_v2.md`, and `docs/M25_EXECUTION_BACKLOG.md`. |
| **M26** Package/Repo Ecosystem v3 | n/a | done | Rugo: package/repo v3 contracts + deterministic policy/rebuild/update-trust artifacts, `make test-pkg-ecosystem-v3`, `make test-update-trust-v1`, CI `Package ecosystem v3 gate` + `Update trust v1 gate`, docs in `docs/pkg/*_v3`, `docs/pkg/update_trust_model_v1.md`, and `docs/M26_EXECUTION_BACKLOG.md`. |
| **M27** External App Compatibility Program v3 | n/a | done | Rugo: compatibility profile/tier contracts + deterministic class matrix artifacts, `make test-app-compat-v3`, CI `App compatibility v3 gate`, docs in `docs/abi/compat_profile_v3.md`, `docs/abi/app_compat_tiers_v1.md`, and `docs/M27_EXECUTION_BACKLOG.md`. |
| **M28** Security Hardening Program v3 | n/a | done | Rugo: hardening profile/threat model contracts + deterministic attack/fuzz and vulnerability-response artifacts, `make test-security-hardening-v3`, `make test-vuln-response-v1`, CI `Security hardening v3 gate` + `Vulnerability response v1 gate`, docs in `docs/security/hardening_profile_v3.md`, `docs/security/threat_model_v2.md`, and `docs/M28_EXECUTION_BACKLOG.md`. |
| **M29** Observability + Diagnostics v2 | n/a | done | Rugo: observability/crash contracts + deterministic trace/diagnostic/crash artifacts, `make test-observability-v2`, `make test-crash-dump-v1`, CI `Observability v2 gate` + `Crash dump v1 gate`, docs in `docs/runtime/observability_contract_v2.md`, `docs/runtime/crash_dump_contract_v1.md`, and `docs/M29_EXECUTION_BACKLOG.md`. |
| **M30** Installer/Upgrade/Recovery UX v3 | n/a | done | Rugo: installer/recovery v3 contracts + deterministic upgrade/recovery rollback-safety artifacts, `make test-ops-ux-v3`, CI `Ops UX v3 gate`, docs in `docs/build/installer_ux_v3.md`, `docs/build/recovery_workflow_v3.md`, and `docs/M30_EXECUTION_BACKLOG.md`. |
| **M31** Release Engineering + Support Lifecycle v2 | n/a | done | Rugo: release/support/revalidation policy contracts + deterministic branch/support/supply-chain audits, `make test-release-lifecycle-v2`, `make test-supply-chain-revalidation-v1`, CI `Release lifecycle v2 gate` + `Supply-chain revalidation v1 gate`, docs in `docs/build/release_policy_v2.md`, `docs/build/support_lifecycle_policy_v1.md`, and `docs/M31_EXECUTION_BACKLOG.md`. |
| **M32** Conformance + Profile Qualification v1 | n/a | done | Rugo: profile conformance contract + deterministic profile qualification suite artifacts, `make test-conformance-v1`, CI `Conformance v1 gate`, docs in `docs/runtime/profile_conformance_v1.md`, and `docs/M32_EXECUTION_BACKLOG.md`. |
| **M33** Fleet-Scale Operations Baseline v1 | n/a | done | Rugo: fleet update/health and rollout-safety policy contracts + deterministic fleet/canary/abort simulations, `make test-fleet-ops-v1`, `make test-fleet-rollout-safety-v1`, CI `Fleet ops v1 gate` + `Fleet rollout safety v1 gate`, docs in `docs/pkg/fleet_update_policy_v1.md`, `docs/runtime/fleet_health_policy_v1.md`, and `docs/M33_EXECUTION_BACKLOG.md`. |
| **M34** Maturity Qualification + LTS Declaration | n/a | done | Rugo: maturity qualification/LTS declaration contracts + deterministic cross-domain qualification bundle, `make test-maturity-qual-v1`, CI `Maturity qualification v1 gate`, docs in `docs/build/maturity_qualification_v1.md`, `docs/build/lts_declaration_policy_v1.md`, and `docs/M34_EXECUTION_BACKLOG.md`. |
| **M35** Desktop + Interactive UX Baseline v1 | n/a | done | Rugo: desktop/display/input contracts + deterministic desktop/gui artifacts, `make test-desktop-stack-v1`, `make test-gui-app-compat-v1`, CI `Desktop stack v1 gate` + `GUI app compatibility v1 gate`, docs in `docs/desktop/*`, and `docs/M35_EXECUTION_BACKLOG.md`. |
| **M36** Compatibility Surface Expansion v1 | n/a | done | Rugo: compatibility/process/socket contracts + deterministic compatibility/POSIX artifacts, `make test-compat-surface-v1`, `make test-posix-gap-closure-v1`, CI `Compatibility surface v1 gate` + `POSIX gap closure v1 gate`, docs in `docs/abi/compat_profile_v4.md`, `docs/runtime/syscall_coverage_matrix_v3.md`, and `docs/M36_EXECUTION_BACKLOG.md`. |
| **M37** Hardware Breadth + Driver Matrix v4 | n/a | done | Rugo: matrix v4/driver-lifecycle/promotion contracts + deterministic hardware/promotion artifacts, `make test-hw-matrix-v4`, `make test-hw-baremetal-promotion-v1`, CI `Hardware matrix v4 gate` + `Hardware bare-metal promotion v1 gate`, docs in `docs/hw/support_matrix_v4.md`, `docs/hw/driver_lifecycle_contract_v4.md`, and `docs/M37_EXECUTION_BACKLOG.md`. |
| **M38** Storage + Platform Feature Expansion v1 | n/a | done | Rugo: storage/platform feature contracts + deterministic snapshot/resize/fs-ops and platform-conformance artifacts, `make test-storage-platform-v1`, `make test-storage-feature-contract-v1`, CI `Storage platform v1 gate` + `Storage feature contract v1 gate`, docs in `docs/storage/fs_feature_contract_v1.md`, `docs/runtime/platform_feature_profile_v1.md`, and `docs/M38_EXECUTION_BACKLOG.md`. |
| **M39** Ecosystem Scale + Distribution Workflow v1 | n/a | done | Rugo: ecosystem-scale/distribution contracts + deterministic catalog-scale/install/audit artifacts, `make test-ecosystem-scale-v1`, `make test-app-catalog-health-v1`, CI `Ecosystem scale v1 gate` + `App catalog health v1 gate`, docs in `docs/pkg/ecosystem_scale_policy_v1.md`, `docs/pkg/distribution_workflow_v1.md`, and `docs/M39_EXECUTION_BACKLOG.md`. |
| **M40** Runtime-Backed Evidence Integrity v1 | n/a | done | Rugo: evidence-integrity/provenance contracts + deterministic runtime-evidence/audit artifacts, `make test-evidence-integrity-v1`, `make test-synthetic-evidence-ban-v1`, CI `Evidence integrity v1 gate` + `Synthetic evidence ban v1 gate`, docs in `docs/runtime/evidence_integrity_policy_v1.md`, `docs/runtime/gate_provenance_policy_v1.md`, and `docs/M40_EXECUTION_BACKLOG.md`. |
| **M41** Process + Readiness Compatibility Closure v1 | n/a | done | Rugo: compatibility/process/readiness contracts + deterministic compatibility/POSIX artifacts, `make test-process-readiness-parity-v1`, `make test-posix-gap-closure-v2`, CI `Process readiness parity v1 gate` + `POSIX gap closure v2 gate`, docs in `docs/abi/compat_profile_v5.md`, `docs/runtime/syscall_coverage_matrix_v4.md`, and `docs/M41_EXECUTION_BACKLOG.md`. |

✅ done &ensp; ◐ in progress (prep) &ensp; ⬜ not started &ensp; n/a not applicable

## Legacy verification

### G0 (Go kernel entry) — verified 2026-02-18

**Status: PASS.** All 16 legacy tests pass, including G0 (`test_go_entry`).
Verified via `make docker-legacy` (Docker, Ubuntu 24.04, gccgo).

**Test file:** `legacy/tests/boot/test_go_entry.py`

**Expected serial markers:**
- `GO: kmain ok` — Go kernel entry executed
- `KERNEL: boot ok` — M0 boot marker
- `KERNEL: halt ok` — M0 clean halt marker

### How to run Legacy G0

```bash
# Via Docker (recommended — no host toolchain needed)
make docker-legacy

# Native (requires nasm, gcc, gccgo, ld, objcopy, xorriso, qemu, pytest)
make -C legacy build && make -C legacy image && make -C legacy test-qemu

# G0 only (native)
make -C legacy build && make -C legacy image \
  && python3 -m pytest legacy/tests/boot/test_go_entry.py -v
```

### Fixes applied (2026-02-18)

1. **conftest REPO_ROOT path:** `legacy/tests/conftest.py` and
   `legacy/tests/net/test_udp_echo.py` computed `REPO_ROOT` as `legacy/`
   instead of the repo root. Since the legacy `Makefile` sets `OUT = ../out`,
   all build artifacts (ISO, disk images) live in the repo-root `out/`
   directory. Fixed by adding one extra `os.path.dirname()` call in each file.

2. **`docker-legacy` Makefile target:** Added to the root `Makefile` so legacy
   tests can run via Docker on any platform (Windows, macOS, CI) without
   installing gccgo, QEMU, or xorriso on the host.

## Current focus

M0–M7 and G1 are complete: boot, paging, traps, scheduler, user mode, syscalls,
IPC, shared memory, service registry, VirtIO block, filesystem, package manager,
shell, VirtIO net, UDP echo, and TinyGo user-space services are all functional
with passing QEMU integration tests.

G2 is complete as of 2026-03-04:
- `tools/build_go_std_spike.sh` now executes a stock-Go path
  (`go run ./tools/gostd_stock_builder/main.go`) and writes
  `out/gostd.bin` plus `out/gostd-contract.env`.
- `tests/go/test_std_go_binary.py` validates serial marker flow and stock-Go
  contract metadata (`GOOS=rugo`, `GOARCH=amd64`, `STOCK_GO_*` keys).
- `make image-go-std` remains the G2 artifact gate for `os-go-std.iso`.
- Execution and sequencing history remains in `docs/G2_EXECUTION_BACKLOG.md`.

M8 execution update (2026-03-04):
- PR-1 complete (contract-first deliverables):
  - `docs/abi/syscall_v1.md`
  - `docs/abi/compat_profile_v1.md`
  - `tests/compat/` initial suite scaffolding
- PR-2 complete (loader/process/fd core compatibility primitives):
  - `docs/abi/process_thread_model_v1.md`
  - `kernel_rs/src/lib.rs` v1 syscall additions (`sys_open/sys_read/sys_write/sys_close/sys_wait/sys_poll`)
  - executable compat checks:
    - `tests/compat/test_loader_contract.py`
    - `tests/compat/test_process_lifecycle.py`
    - `tests/compat/test_process_wait.py`
    - `tests/compat/test_fd_table.py`
- PR-3 complete (subset closure + package/bootstrap lane + CI gate):
  - executable time/signal/socket closure:
    - `tests/compat/test_time_signal_subset.py`
    - `tests/compat/test_socket_api_subset.py`
    - `tests/compat/test_posix_subset.py`
  - package/repository v1 bootstrap:
    - `docs/pkg/package_format_v1.md`
    - `tools/pkg_bootstrap_v1.py`
    - `tests/pkg/test_pkg_external_apps.py`
  - release-gating CI lane:
    - `.github/workflows/ci.yml` compatibility profile v1 gate step
- M8 is done.

M9 execution update (2026-03-04):
- PR-1 complete (hardware matrix contract + harness):
  - `docs/hw/support_matrix_v1.md`
  - `tests/hw/test_hardware_matrix_v1.py`
  - `tests/hw/test_probe_negative_paths_v1.py`
  - tiered QEMU fixtures in `tests/conftest.py` (`q35`, `pc`/i440fx)
- PR-2 complete (PCI cleanup + DMA safety gate):
  - shared PCI helpers + claim path in `kernel_rs/src/lib.rs`
  - DMA rejection checks in `tests/hw/test_dma_safety_v1.py`
- PR-3 complete (CI gate + hardening docs + milestone closure):
  - `Makefile` target `test-hw-matrix`
  - `.github/workflows/ci.yml` step `Hardware matrix v1 gate`
  - `docs/hw/dma_iommu_strategy_v1.md`
  - `docs/hw/acpi_uefi_hardening_v1.md`
  - `docs/hw/bare_metal_bringup_v1.md`
- M9 is done.

M10 execution update (2026-03-04):
- PR-1 complete (rights model + kernel enforcement):
  - fd rights enforcement in `kernel_rs/src/lib.rs`
  - new syscalls `24..27` (`sys_fd_rights_*`, `sys_sec_profile_set`)
  - `services/security/sec_rights.asm`
  - `tests/security/test_rights_enforcement.py`
  - `docs/security/rights_capability_model_v1.md`
- PR-2 complete (syscall filtering + secure boot manifest):
  - restricted profile allowlist/path policy in `kernel_rs/src/lib.rs`
  - `services/security/sec_filter.asm`
  - `tests/security/test_syscall_filter.py`
  - `tools/secure_boot_manifest_v1.py`
  - `tests/security/test_secure_boot_manifest_v1.py`
  - `docs/security/syscall_filtering_v1.md`
  - `docs/security/secure_boot_policy_v1.md`
- PR-3 complete (fuzz gate + incident process + CI gate):
  - `tools/run_security_fuzz_v1.py`
  - `tests/security/test_security_fuzz_harness_v1.py`
  - `docs/security/fuzzing_v1.md`
  - `docs/security/incident_response_v1.md`
  - `Makefile` target `test-security-baseline`
- `.github/workflows/ci.yml` step `Security baseline v1 gate`
- M10 is done.

M11 execution update (2026-03-04):
- PR-1 complete (runtime contract + coverage matrix):
  - `docs/runtime/port_contract_v1.md`
  - `docs/runtime/syscall_coverage_matrix_v1.md`
  - `docs/runtime/abi_stability_policy_v1.md`
  - `tests/runtime/test_runtime_contract_docs_v1.py`
  - `tests/runtime/test_runtime_abi_window_v1.py`
- PR-2 complete (bootstrap + reproducibility tooling):
  - `tools/bootstrap_go_port_v1.sh`
  - `tools/runtime_toolchain_contract_v1.py`
  - `docs/runtime/toolchain_bootstrap_v1.md`
- PR-3 complete (runtime gate + maintainers + milestone closure):
  - `tests/runtime/test_runtime_stress_v1.py`
  - `docs/runtime/maintainers_v1.md`
  - `Makefile` target `test-runtime-maturity`
  - `.github/workflows/ci.yml` step `Runtime + toolchain maturity v1 gate`
- M11 is done.

M12 execution update (2026-03-04):
- PR-1 complete (network/socket contract + IPv4/UDP baseline):
  - `docs/net/network_stack_contract_v1.md`
  - `docs/net/socket_contract_v1.md`
  - `docs/net/ipv4_udp_profile_v1.md`
  - `tests/net/test_udp_echo.py`
  - `tests/net/test_ipv4_udp_contract_v1.py`
  - `tests/net/test_socket_contract_docs_v1.py`
- PR-2 complete (TCP + socket semantics baseline):
  - `docs/net/tcp_state_machine_v1.md`
  - `docs/net/retransmission_timer_policy_v1.md`
  - `tests/net/v1_model.py`
  - `tests/net/test_tcp_state_machine_v1.py`
  - `tests/net/test_tcp_retransmission_v1.py`
  - `tests/net/test_socket_poll_semantics_v1.py`
- PR-3 complete (IPv6 baseline + interop/soak gate + milestone closure):
  - `docs/net/ipv6_baseline_v1.md`
  - `tools/net_trace_capture_v1.py`
  - `tools/run_net_interop_matrix_v1.py`
  - `tools/run_net_soak_v1.py`
  - `tests/net/test_ipv6_nd_icmpv6_v1.py`
  - `tests/net/test_net_trace_capture_v1.py`
  - `tests/net/test_net_interop_matrix_v1.py`
  - `tests/net/test_net_soak_v1.py`
  - `Makefile` target `test-network-stack-v1`
  - `.github/workflows/ci.yml` step `Network stack v1 gate`
- M12 is done.

M13 execution update (2026-03-04):
- PR-1 complete (storage contracts + durability baseline):
  - `docs/storage/fs_v1.md`
  - `docs/storage/durability_model_v1.md`
  - `docs/storage/write_ordering_policy_v1.md`
  - `tests/storage/test_storage_contract_docs_v1.py`
  - `tests/storage/test_fsync_semantics_v1.py`
  - `tests/storage/test_write_ordering_contract_v1.py`
- PR-2 complete (recovery tooling + fault campaign):
  - `tools/storage_recover_v1.py`
  - `tools/run_storage_fault_campaign_v1.py`
  - `docs/storage/recovery_playbook_v1.md`
  - `docs/storage/fault_campaign_v1.md`
  - `tests/storage/test_storage_recovery_v1.py`
  - `tests/storage/test_storage_fault_campaign_v1.py`
  - `tests/storage/test_storage_integrity_checker_v1.py`
- PR-3 complete (storage reliability gate + milestone closure):
  - `tests/storage/test_storage_reliability_gate_v1.py`
  - `Makefile` target `test-storage-reliability-v1`
  - `.github/workflows/ci.yml` step `Storage reliability v1 gate`
- M13 is done.

M14 execution update (2026-03-04):
- PR-1 complete (release contracts and governance docs):
  - `docs/build/release_policy_v1.md`
  - `docs/build/versioning_scheme_v1.md`
  - `docs/build/release_checklist_v1.md`
  - `tools/release_contract_v1.py`
  - `tests/build/test_release_contract_docs_v1.py`
  - `tests/build/test_release_contract_report_v1.py`
- PR-2 complete (signed update metadata + rollback defenses):
  - `docs/pkg/update_protocol_v1.md`
  - `docs/pkg/update_repo_layout_v1.md`
  - `docs/security/update_signing_policy_v1.md`
  - `tools/update_repo_sign_v1.py`
  - `tools/update_client_verify_v1.py`
  - `tools/run_update_attack_suite_v1.py`
  - `tests/pkg/test_update_metadata_v1.py`
  - `tests/pkg/test_update_rollback_protection_v1.py`
  - `tests/pkg/test_update_attack_suite_v1.py`
- PR-3 complete (supply-chain + release gate + milestone closure):
  - `docs/build/supply_chain_policy_v1.md`
  - `docs/build/installer_recovery_baseline_v1.md`
  - `tools/generate_sbom_v1.py`
  - `tools/generate_provenance_v1.py`
  - `tools/collect_support_bundle_v1.py`
  - `tests/build/test_release_engineering_gate_v1.py`
  - `Makefile` target `test-release-engineering-v1`
  - `.github/workflows/ci.yml` step `Release engineering v1 gate`
- M14 is done.

M15 execution update (2026-03-06):
- PR-1 complete (matrix v2 contract + target classes):
  - `docs/hw/support_matrix_v2.md`
  - `docs/hw/device_profile_contract_v2.md`
  - `tests/hw/test_hardware_matrix_v2.py`
  - `tests/hw/test_probe_negative_paths_v2.py`
- PR-2 complete (driver lifecycle + firmware hardening docs/tests):
  - `docs/hw/dma_iommu_strategy_v2.md`
  - `docs/hw/acpi_uefi_hardening_v2.md`
  - `tests/hw/test_dma_iommu_policy_v2.py`
  - `tests/hw/test_acpi_boot_paths_v2.py`
- PR-3 complete (bare-metal lane + gate promotion):
  - `docs/hw/bare_metal_bringup_v2.md`
  - `tests/hw/test_bare_metal_smoke_v2.py`
  - `tests/hw/test_hw_gate_v2.py`
  - `Makefile` target `test-hw-matrix-v2`
  - `.github/workflows/ci.yml` step `Hardware matrix v2 gate`
- M15 is done.

M16 execution update (2026-03-06):
- PR-1 complete (process/thread contract v2):
  - `docs/abi/process_thread_model_v2.md`
  - `docs/abi/scheduling_policy_v2.md`
  - `tests/user/test_process_wait_kill_v2.py`
  - `tests/user/test_signal_delivery_v2.py`
- PR-2 complete (scheduler behavior/fairness + soak model):
  - `tests/sched/v2_model.py`
  - `tests/sched/test_preempt_timer_quantum_v2.py`
  - `tests/sched/test_priority_fairness_v2.py`
  - `tests/sched/test_scheduler_soak_v2.py`
- PR-3 complete (scheduler v2 gate + closure wiring):
  - `tests/sched/test_scheduler_gate_v2.py`
  - `Makefile` target `test-process-scheduler-v2`
  - `.github/workflows/ci.yml` step `Process scheduler v2 gate`
- M16 is done.

M17 execution update (2026-03-06):
- PR-1 complete (ABI/profile/loader contract v2):
  - `docs/abi/syscall_v2.md`
  - `docs/abi/compat_profile_v2.md`
  - `docs/abi/elf_loader_contract_v2.md`
  - `tests/compat/test_abi_profile_v2_docs.py`
  - `tests/compat/test_elf_loader_dynamic_v2.py`
- PR-2 complete (POSIX subset expansion + external app tier model):
  - `docs/runtime/syscall_coverage_matrix_v2.md`
  - `tests/compat/v2_model.py`
  - `tests/compat/test_posix_profile_v2.py`
  - `tests/compat/test_external_apps_tier_v2.py`
- PR-3 complete (compatibility v2 gate + closure wiring):
  - `tests/compat/test_compat_gate_v2.py`
  - `Makefile` target `test-compat-v2`
  - `.github/workflows/ci.yml` step `Compatibility profile v2 gate`
- M17 is done.

M18 execution update (2026-03-06):
- PR-1 complete (storage contract + journaling policy v2):
  - `docs/storage/fs_v2.md`
  - `docs/storage/durability_model_v2.md`
  - `docs/storage/write_ordering_policy_v2.md`
  - `tests/storage/test_journal_recovery_v2.py`
  - `tests/storage/test_metadata_integrity_v2.py`
- PR-2 complete (recovery + power-fail tooling and campaign):
  - `tools/storage_recover_v2.py`
  - `tools/run_storage_powerfail_campaign_v2.py`
  - `docs/storage/recovery_playbook_v2.md`
  - `docs/storage/fault_campaign_v2.md`
  - `tests/storage/test_powerfail_campaign_v2.py`
- PR-3 complete (storage reliability v2 gate + closure wiring):
  - `tests/storage/test_storage_gate_v2.py`
  - `Makefile` target `test-storage-reliability-v2`
  - `.github/workflows/ci.yml` step `Storage reliability v2 gate`
- M18 is done.

M19 execution update (2026-03-08):
- PR-1 complete (protocol + socket contract v2):
  - `docs/net/network_stack_contract_v2.md`
  - `docs/net/socket_contract_v2.md`
  - `docs/net/tcp_profile_v2.md`
  - `tests/net/test_tcp_interop_v2.py`
  - `tests/net/test_ipv6_interop_v2.py`
- PR-2 complete (service behavior + diagnostics artifacts):
  - `tools/run_net_interop_matrix_v2.py`
  - `tools/run_net_soak_v2.py`
  - `docs/net/interop_matrix_v2.md`
  - `tests/net/test_dns_stub_v2.py`
- PR-3 complete (network stack v2 gate + closure wiring):
  - `tests/net/test_network_gate_v2.py`
  - `Makefile` target `test-network-stack-v2`
  - `.github/workflows/ci.yml` step `Network stack v2 gate`
- M19 is done.

M20 execution update (2026-03-09):
- PR-1 complete (installer + operational contract v2):
  - `docs/build/installer_recovery_baseline_v2.md`
  - `docs/build/operations_runbook_v2.md`
  - `tools/build_installer_v2.py`
  - `tests/build/test_installer_recovery_v2.py`
- PR-2 complete (upgrade/rollback drill + support bundle v2):
  - `docs/pkg/update_protocol_v2.md`
  - `docs/pkg/rollback_policy_v2.md`
  - `tools/run_upgrade_recovery_drill_v2.py`
  - `tools/collect_support_bundle_v2.py`
  - `tests/build/test_upgrade_rollback_v2.py`
  - `tests/build/test_support_bundle_v2.py`
- PR-3 complete (operability v2 gate + closure wiring):
  - `tests/build/test_operability_gate_v2.py`
  - `Makefile` target `test-release-ops-v2`
  - `.github/workflows/ci.yml` step `Operability and release UX v2 gate`
- M20 is done.

M21 execution update (2026-03-09):
- PR-1 complete (ABI v3 contract freeze):
  - `docs/abi/syscall_v3.md`
  - `docs/runtime/abi_stability_policy_v2.md`
  - `docs/runtime/deprecation_window_policy_v1.md`
  - `tests/runtime/test_abi_docs_v3.py`
  - `tests/runtime/test_abi_window_v3.py`
- PR-2 complete (compatibility enforcement tooling):
  - `tools/check_abi_diff_v3.py`
  - `tools/check_syscall_compat_v3.py`
  - `tests/runtime/test_abi_diff_gate_v3.py`
  - `tests/compat/test_abi_compat_matrix_v3.py`
- PR-3 complete (ABI stability v3 gate + closure wiring):
  - `tests/runtime/test_abi_stability_gate_v3.py`
  - `Makefile` target `test-abi-stability-v3`
  - `.github/workflows/ci.yml` step `ABI stability v3 gate`
- M21 is done.

M22 execution update (2026-03-09):
- PR-1 complete (reliability model + soak baseline):
  - `docs/runtime/kernel_reliability_model_v1.md`
  - `tests/stress/test_kernel_soak_24h_v1.py`
  - `tests/stress/test_fault_injection_matrix_v1.py`
- PR-2 complete (campaign tooling + artifact schema):
  - `tools/run_kernel_soak_v1.py`
  - `tools/run_fault_campaign_kernel_v1.py`
  - `tests/stress/test_reliability_artifact_schema_v1.py`
- PR-3 complete (kernel reliability v1 gate + closure wiring):
  - `tests/stress/test_kernel_reliability_gate_v1.py`
  - `Makefile` target `test-kernel-reliability-v1`
  - `.github/workflows/ci.yml` step `Kernel reliability v1 gate`
- M22 is done.

M23 execution update (2026-03-09):
- PR-1 complete (matrix v3 + firmware contracts):
  - `docs/hw/support_matrix_v3.md`
  - `docs/hw/driver_lifecycle_contract_v3.md`
  - `docs/hw/firmware_resiliency_policy_v1.md`
  - `docs/security/measured_boot_attestation_v1.md`
  - `tests/hw/test_hardware_matrix_v3.py`
  - `tests/hw/test_driver_lifecycle_v3.py`
- PR-2 complete (suspend/hotplug + measured-boot evidence):
  - `tools/collect_hw_diagnostics_v3.py`
  - `tools/collect_measured_boot_report_v1.py`
  - `tests/hw/test_suspend_resume_v1.py`
  - `tests/hw/test_hotplug_baseline_v1.py`
  - `tests/hw/test_measured_boot_attestation_v1.py`
  - `tests/hw/test_tpm_eventlog_schema_v1.py`
- PR-3 complete (hardware v3 gate + firmware sub-gate):
  - `tests/hw/test_hw_gate_v3.py`
  - `tests/hw/test_firmware_attestation_gate_v1.py`
  - `Makefile` targets `test-hw-matrix-v3`, `test-firmware-attestation-v1`
  - `.github/workflows/ci.yml` steps `Hardware matrix v3 gate`, `Firmware attestation v1 gate`
- M23 is done.

M24 execution update (2026-03-09):
- PR-1 complete (budget and benchmark policy contracts):
  - `docs/runtime/performance_budget_v1.md`
  - `docs/runtime/benchmark_policy_v1.md`
  - `tests/runtime/test_perf_budget_docs_v1.py`
- PR-2 complete (deterministic baseline + regression tooling):
  - `tools/run_perf_baseline_v1.py`
  - `tools/check_perf_regression_v1.py`
  - `tests/runtime/test_perf_regression_v1.py`
- PR-3 complete (performance regression v1 gate + closure wiring):
  - `tests/runtime/test_perf_gate_v1.py`
  - `Makefile` target `test-perf-regression-v1`
  - `.github/workflows/ci.yml` step `Performance regression v1 gate`
- M24 is done.

M25 execution update (2026-03-09):
- PR-1 complete (service/init v2 contract freeze):
  - `docs/runtime/service_model_v2.md`
  - `docs/runtime/init_contract_v2.md`
  - `tests/runtime/test_service_model_docs_v2.py`
- PR-2 complete (deterministic lifecycle/dependency/restart semantics):
  - `tests/runtime/test_service_lifecycle_v2.py`
  - `tests/runtime/test_service_dependency_order_v2.py`
  - `tests/runtime/test_restart_policy_v2.py`
- PR-3 complete (userspace model v2 gate + closure wiring):
  - `tests/runtime/test_userspace_model_gate_v2.py`
  - `Makefile` target `test-userspace-model-v2`
  - `.github/workflows/ci.yml` step `Userspace model v2 gate`
- M25 is done.

M26 execution update (2026-03-09):
- PR-1 complete (package/repo v3 + update trust contract freeze):
  - `docs/pkg/package_format_v3.md`
  - `docs/pkg/repository_policy_v3.md`
  - `docs/pkg/update_trust_model_v1.md`
  - `docs/security/update_key_rotation_policy_v1.md`
  - `tests/pkg/test_pkg_contract_docs_v3.py`
  - `tests/pkg/test_update_trust_docs_v1.py`
- PR-2 complete (policy/rebuild enforcement tooling + trust hardening):
  - `tools/repo_policy_check_v3.py`
  - `tools/pkg_rebuild_verify_v3.py`
  - `tools/check_update_trust_v1.py`
  - `tools/run_update_key_rotation_drill_v1.py`
  - `tests/pkg/test_pkg_rebuild_repro_v3.py`
  - `tests/pkg/test_repo_policy_v3.py`
  - `tests/pkg/test_update_metadata_expiry_v1.py`
  - `tests/pkg/test_update_freeze_attack_v1.py`
  - `tests/pkg/test_update_mix_and_match_v1.py`
  - `tests/pkg/test_update_key_rotation_v1.py`
- PR-3 complete (package ecosystem v3 gate + update trust sub-gate):
  - `tests/pkg/test_pkg_ecosystem_gate_v3.py`
  - `tests/pkg/test_update_trust_gate_v1.py`
  - `Makefile` targets `test-pkg-ecosystem-v3`, `test-update-trust-v1`
  - `.github/workflows/ci.yml` steps `Package ecosystem v3 gate`, `Update trust v1 gate`
- M26 is done.

M27 execution update (2026-03-09):
- PR-1 complete (compatibility profile v3 + tier taxonomy freeze):
  - `docs/abi/compat_profile_v3.md`
  - `docs/abi/app_compat_tiers_v1.md`
  - `tests/compat/test_app_tier_docs_v1.py`
- PR-2 complete (deterministic app-class suite expansion):
  - `tools/run_app_compat_matrix_v3.py`
  - `tests/compat/test_cli_suite_v3.py`
  - `tests/compat/test_runtime_suite_v3.py`
  - `tests/compat/test_service_suite_v3.py`
- PR-3 complete (app compatibility v3 gate + closure wiring):
  - `tests/compat/test_app_compat_gate_v3.py`
  - `Makefile` target `test-app-compat-v3`
  - `.github/workflows/ci.yml` step `App compatibility v3 gate`
- M27 is done.

M28 execution update (2026-03-09):
- PR-1 complete (hardening v3 + vulnerability-response contracts):
  - `docs/security/hardening_profile_v3.md`
  - `docs/security/threat_model_v2.md`
  - `docs/security/vulnerability_response_policy_v1.md`
  - `docs/security/security_advisory_policy_v1.md`
  - `tests/security/test_hardening_docs_v3.py`
  - `tests/security/test_vuln_response_docs_v1.py`
- PR-2 complete (deterministic hardening enforcement + response tooling):
  - `tools/run_security_attack_suite_v3.py`
  - `tools/run_security_fuzz_v2.py`
  - `tools/security_advisory_lint_v1.py`
  - `tools/security_embargo_drill_v1.py`
  - `tests/security/test_attack_suite_v3.py`
  - `tests/security/test_fuzz_gate_v2.py`
  - `tests/security/test_policy_enforcement_v3.py`
  - `tests/security/test_vuln_triage_sla_v1.py`
  - `tests/security/test_embargo_workflow_v1.py`
  - `tests/security/test_advisory_schema_v1.py`
- PR-3 complete (security hardening v3 gate + vuln-response sub-gate):
  - `tests/security/test_security_hardening_gate_v3.py`
  - `tests/security/test_vuln_response_gate_v1.py`
  - `Makefile` targets `test-security-hardening-v3`, `test-vuln-response-v1`
  - `.github/workflows/ci.yml` steps `Security hardening v3 gate`, `Vulnerability response v1 gate`
- M28 is done.

M29 execution update (2026-03-09):
- PR-1 complete (observability + crash/postmortem contracts):
  - `docs/runtime/observability_contract_v2.md`
  - `docs/runtime/crash_dump_contract_v1.md`
  - `docs/runtime/postmortem_triage_playbook_v1.md`
  - `tests/runtime/test_observability_docs_v2.py`
  - `tests/runtime/test_crash_dump_docs_v1.py`
- PR-2 complete (deterministic trace/diagnostic + crash tooling):
  - `tools/collect_trace_bundle_v2.py`
  - `tools/collect_diagnostic_snapshot_v2.py`
  - `tools/collect_crash_dump_v1.py`
  - `tools/symbolize_crash_dump_v1.py`
  - `tests/runtime/test_trace_bundle_v2.py`
  - `tests/runtime/test_diag_snapshot_v2.py`
  - `tests/runtime/test_crash_dump_capture_v1.py`
  - `tests/runtime/test_crash_dump_symbolization_v1.py`
- PR-3 complete (observability v2 gate + crash-dump sub-gate):
  - `tests/runtime/test_observability_gate_v2.py`
  - `tests/runtime/test_crash_dump_gate_v1.py`
  - `Makefile` targets `test-observability-v2`, `test-crash-dump-v1`
  - `.github/workflows/ci.yml` steps `Observability v2 gate`, `Crash dump v1 gate`
- M29 is done.

M30 execution update (2026-03-09):
- PR-1 complete (installer + recovery workflow contracts):
  - `docs/build/installer_ux_v3.md`
  - `docs/build/recovery_workflow_v3.md`
  - `tests/build/test_installer_ux_v3.py`
- PR-2 complete (deterministic upgrade/recovery drills + rollback safety):
  - `tools/run_upgrade_drill_v3.py`
  - `tools/run_recovery_drill_v3.py`
  - `tests/build/test_upgrade_recovery_v3.py`
  - `tests/build/test_rollback_safety_v3.py`
- PR-3 complete (ops UX v3 gate + closure wiring):
  - `tests/build/test_ops_ux_gate_v3.py`
  - `Makefile` target `test-ops-ux-v3`
  - `.github/workflows/ci.yml` step `Ops UX v3 gate`
- M30 is done.

M31 execution update (2026-03-09):
- PR-1 complete (release lifecycle/support/revalidation contract freeze):
  - `docs/build/release_policy_v2.md`
  - `docs/build/support_lifecycle_policy_v1.md`
  - `docs/build/supply_chain_revalidation_policy_v1.md`
  - `docs/build/release_attestation_policy_v1.md`
  - `tests/build/test_release_policy_v2_docs.py`
  - `tests/build/test_supply_chain_revalidation_docs_v1.py`
- PR-2 complete (deterministic lifecycle and revalidation audits):
  - `tools/release_branch_audit_v2.py`
  - `tools/support_window_audit_v1.py`
  - `tools/verify_sbom_provenance_v2.py`
  - `tools/verify_release_attestations_v1.py`
  - `tests/build/test_release_branch_policy_v2.py`
  - `tests/build/test_support_window_policy_v1.py`
  - `tests/build/test_sbom_revalidation_v1.py`
  - `tests/build/test_provenance_verification_v1.py`
  - `tests/build/test_attestation_drift_v1.py`
- PR-3 complete (lifecycle v2 gate + supply-chain sub-gate wiring):
  - `tests/build/test_release_lifecycle_gate_v2.py`
  - `tests/build/test_supply_chain_revalidation_gate_v1.py`
  - `Makefile` targets `test-release-lifecycle-v2`, `test-supply-chain-revalidation-v1`
  - `.github/workflows/ci.yml` steps `Release lifecycle v2 gate`, `Supply-chain revalidation v1 gate`
- M31 is done.

M32 execution update (2026-03-09):
- PR-1 complete (profile conformance contract freeze):
  - `docs/runtime/profile_conformance_v1.md`
  - `tests/runtime/test_profile_conformance_docs_v1.py`
- PR-2 complete (deterministic profile qualification tooling + checks):
  - `tools/run_conformance_suite_v1.py`
  - `tests/runtime/test_server_profile_v1.py`
  - `tests/runtime/test_dev_profile_v1.py`
- PR-3 complete (conformance v1 gate + closure wiring):
  - `tests/runtime/test_conformance_gate_v1.py`
  - `Makefile` target `test-conformance-v1`
  - `.github/workflows/ci.yml` step `Conformance v1 gate`
- M32 is done.

M33 execution update (2026-03-09):
- PR-1 complete (fleet and rollout policy contract freeze):
  - `docs/pkg/fleet_update_policy_v1.md`
  - `docs/runtime/fleet_health_policy_v1.md`
  - `docs/pkg/staged_rollout_policy_v1.md`
  - `docs/runtime/canary_slo_policy_v1.md`
  - `tests/pkg/test_fleet_policy_docs_v1.py`
  - `tests/pkg/test_rollout_policy_docs_v1.py`
- PR-2 complete (deterministic fleet/rollout simulation tooling + checks):
  - `tools/run_fleet_update_sim_v1.py`
  - `tools/run_fleet_health_sim_v1.py`
  - `tools/run_canary_rollout_sim_v1.py`
  - `tools/run_rollout_abort_drill_v1.py`
  - `tests/pkg/test_fleet_update_sim_v1.py`
  - `tests/runtime/test_fleet_health_sim_v1.py`
  - `tests/pkg/test_canary_rollout_sim_v1.py`
  - `tests/runtime/test_rollout_abort_policy_v1.py`
- PR-3 complete (fleet ops gate + rollout-safety sub-gate wiring):
  - `tests/runtime/test_fleet_ops_gate_v1.py`
  - `tests/runtime/test_fleet_rollout_safety_gate_v1.py`
  - `Makefile` targets `test-fleet-ops-v1`, `test-fleet-rollout-safety-v1`
  - `.github/workflows/ci.yml` steps `Fleet ops v1 gate`, `Fleet rollout safety v1 gate`
- M33 is done.

M34 execution update (2026-03-09):
- PR-1 complete (maturity qualification + LTS declaration contract freeze):
  - `docs/build/maturity_qualification_v1.md`
  - `docs/build/lts_declaration_policy_v1.md`
  - `tests/build/test_maturity_docs_v1.py`
- PR-2 complete (deterministic maturity qualification tooling + checks):
  - `tools/run_maturity_qualification_v1.py`
  - `tests/build/test_maturity_qualification_v1.py`
  - `tests/build/test_lts_policy_v1.py`
  - `tests/build/test_maturity_security_response_drill_v1.py`
  - `tests/build/test_maturity_supply_chain_continuity_v1.py`
  - `tests/build/test_maturity_rollout_safety_v1.py`
- PR-3 complete (maturity qualification v1 gate + closure wiring):
  - `tests/build/test_maturity_gate_v1.py`
  - `Makefile` target `test-maturity-qual-v1`
  - `.github/workflows/ci.yml` step `Maturity qualification v1 gate`
- M34 is done.

M35 execution update (2026-03-09):
- PR-1 complete (desktop/display/window/input contract freeze):
  - `docs/desktop/display_stack_contract_v1.md`
  - `docs/desktop/window_manager_contract_v1.md`
  - `docs/desktop/input_stack_contract_v1.md`
  - `docs/desktop/desktop_profile_v1.md`
  - `tests/desktop/test_desktop_docs_v1.py`
- PR-2 complete (deterministic desktop and GUI baseline tooling + checks):
  - `tools/run_desktop_smoke_v1.py`
  - `tools/run_gui_app_matrix_v1.py`
  - `tests/desktop/test_display_session_v1.py`
  - `tests/desktop/test_input_baseline_v1.py`
  - `tests/desktop/test_window_lifecycle_v1.py`
  - `tests/desktop/test_gui_app_compat_v1.py`
- PR-3 complete (desktop gate + GUI sub-gate wiring):
  - `tests/desktop/test_desktop_gate_v1.py`
  - `tests/desktop/test_gui_app_compat_gate_v1.py`
  - `Makefile` targets `test-desktop-stack-v1`, `test-gui-app-compat-v1`
  - `.github/workflows/ci.yml` steps `Desktop stack v1 gate`, `GUI app compatibility v1 gate`
- M35 is done.

M36 execution update (2026-03-09):
- PR-1 complete (compatibility/process/socket contract freeze):
  - `docs/abi/compat_profile_v4.md`
  - `docs/runtime/syscall_coverage_matrix_v3.md`
  - `docs/abi/process_model_v3.md`
  - `docs/abi/socket_family_expansion_v1.md`
  - `tests/compat/test_compat_docs_v4.py`
- PR-2 complete (deterministic compatibility campaign tooling + checks):
  - `tools/run_compat_surface_campaign_v1.py`
  - `tools/run_posix_gap_report_v1.py`
  - `tests/compat/test_posix_gap_closure_v1.py`
  - `tests/compat/test_process_model_v3.py`
  - `tests/compat/test_socket_family_expansion_v1.py`
  - `tests/compat/test_deferred_surface_behavior_v1.py`
- PR-3 complete (compatibility gate + POSIX sub-gate wiring):
  - `tests/compat/test_compat_surface_gate_v1.py`
  - `tests/compat/test_posix_gap_closure_gate_v1.py`
  - `Makefile` targets `test-compat-surface-v1`, `test-posix-gap-closure-v1`
  - `.github/workflows/ci.yml` steps `Compatibility surface v1 gate`, `POSIX gap closure v1 gate`
- M36 is done.

M37 execution update (2026-03-09):
- PR-1 complete (matrix/lifecycle/promotion contract freeze):
  - `docs/hw/support_matrix_v4.md`
  - `docs/hw/driver_lifecycle_contract_v4.md`
  - `docs/hw/bare_metal_promotion_policy_v1.md`
  - `tests/hw/test_hw_matrix_docs_v4.py`
- PR-2 complete (deterministic matrix + promotion tooling and checks):
  - `tools/run_hw_matrix_v4.py`
  - `tools/collect_hw_promotion_evidence_v1.py`
  - `tests/hw/test_hw_matrix_v4.py`
  - `tests/hw/test_driver_lifecycle_v4.py`
  - `tests/hw/test_baremetal_promotion_v1.py`
  - `tests/hw/test_hw_negative_paths_v4.py`
- PR-3 complete (hardware v4 gate + bare-metal promotion sub-gate wiring):
  - `tests/hw/test_hw_gate_v4.py`
  - `tests/hw/test_hw_baremetal_promotion_gate_v1.py`
  - `Makefile` targets `test-hw-matrix-v4`, `test-hw-baremetal-promotion-v1`
  - `.github/workflows/ci.yml` steps `Hardware matrix v4 gate`, `Hardware bare-metal promotion v1 gate`
- M37 is done.

M38 execution update (2026-03-09):
- PR-1 complete (storage/platform feature contract freeze):
  - `docs/storage/fs_feature_contract_v1.md`
  - `docs/storage/snapshot_policy_v1.md`
  - `docs/storage/online_resize_policy_v1.md`
  - `docs/runtime/platform_feature_profile_v1.md`
  - `tests/storage/test_storage_feature_docs_v1.py`
- PR-2 complete (deterministic feature tooling + checks):
  - `tools/run_storage_feature_campaign_v1.py`
  - `tools/run_platform_feature_conformance_v1.py`
  - `tests/storage/test_snapshot_semantics_v1.py`
  - `tests/storage/test_online_resize_v1.py`
  - `tests/storage/test_advanced_fs_ops_v1.py`
  - `tests/runtime/test_platform_feature_profile_v1.py`
- PR-3 complete (storage platform gate + feature-contract sub-gate wiring):
  - `tests/storage/test_storage_platform_gate_v1.py`
  - `tests/storage/test_storage_feature_contract_gate_v1.py`
  - `Makefile` targets `test-storage-platform-v1`, `test-storage-feature-contract-v1`
  - `.github/workflows/ci.yml` steps `Storage platform v1 gate`, `Storage feature contract v1 gate`
- M38 is done.

M39 execution update (2026-03-09):
- PR-1 complete (ecosystem scale/distribution contract freeze):
  - `docs/pkg/ecosystem_scale_policy_v1.md`
  - `docs/pkg/catalog_quality_contract_v1.md`
  - `docs/pkg/distribution_workflow_v1.md`
  - `tests/pkg/test_ecosystem_scale_docs_v1.py`
- PR-2 complete (deterministic ecosystem-scale tooling + checks):
  - `tools/run_app_catalog_sim_v1.py`
  - `tools/run_pkg_install_success_campaign_v1.py`
  - `tools/run_reproducible_catalog_audit_v1.py`
  - `tests/pkg/test_app_catalog_sim_v1.py`
  - `tests/pkg/test_pkg_install_success_rate_v1.py`
  - `tests/pkg/test_catalog_reproducibility_v1.py`
  - `tests/pkg/test_distribution_workflow_v1.py`
- PR-3 complete (ecosystem gate + app-catalog-health sub-gate wiring):
  - `tests/pkg/test_ecosystem_scale_gate_v1.py`
  - `tests/pkg/test_app_catalog_health_gate_v1.py`
  - `Makefile` targets `test-ecosystem-scale-v1`, `test-app-catalog-health-v1`
  - `.github/workflows/ci.yml` steps `Ecosystem scale v1 gate`, `App catalog health v1 gate`
- M39 is done.

M40 execution update (2026-03-10):
- PR-1 complete (evidence-integrity contract freeze):
  - `docs/runtime/evidence_integrity_policy_v1.md`
  - `docs/runtime/runtime_evidence_schema_v1.md`
  - `docs/runtime/gate_provenance_policy_v1.md`
  - `tests/runtime/test_evidence_integrity_docs_v1.py`
- PR-2 complete (deterministic runtime evidence tooling + checks):
  - `tools/collect_runtime_evidence_v1.py`
  - `tools/audit_gate_evidence_v1.py`
  - `tests/runtime/test_runtime_evidence_collection_v1.py`
  - `tests/runtime/test_gate_evidence_audit_v1.py`
  - `tests/runtime/test_evidence_trace_linkage_v1.py`
- PR-3 complete (evidence-integrity gate + synthetic-ban sub-gate wiring):
  - `tests/runtime/test_evidence_integrity_gate_v1.py`
  - `tests/runtime/test_synthetic_evidence_ban_v1.py`
  - `Makefile` targets `test-evidence-integrity-v1`, `test-synthetic-evidence-ban-v1`
  - `.github/workflows/ci.yml` steps `Evidence integrity v1 gate`, `Synthetic evidence ban v1 gate`
- M40 is done.

M41 execution update (2026-03-10):
- PR-1 complete (process/readiness contract freeze):
  - `docs/abi/compat_profile_v5.md`
  - `docs/runtime/syscall_coverage_matrix_v4.md`
  - `docs/abi/process_model_v4.md`
  - `docs/abi/readiness_io_model_v1.md`
  - `tests/compat/test_compat_docs_v5.py`
- PR-2 complete (deterministic process/readiness campaign tooling + checks):
  - `tools/run_compat_surface_campaign_v2.py`
  - `tools/run_posix_gap_report_v2.py`
  - `tests/compat/test_fork_clone_surface_v1.py`
  - `tests/compat/test_epoll_surface_v1.py`
  - `tests/compat/test_process_model_v4.py`
  - `tests/compat/test_deferred_surface_behavior_v2.py`
- PR-3 complete (process/readiness gate + POSIX sub-gate wiring):
  - `tests/compat/test_process_readiness_gate_v1.py`
  - `tests/compat/test_posix_gap_closure_gate_v2.py`
  - `Makefile` targets `test-process-readiness-parity-v1`,
    `test-posix-gap-closure-v2`
  - `.github/workflows/ci.yml` steps `Process readiness parity v1 gate`,
    `POSIX gap closure v2 gate`
- M41 is done.

Post-G2 planning and execution:
- Extended roadmap (M21-M34): `docs/M21_M34_MATURITY_PARITY_ROADMAP.md`
- Next roadmap (M35-M39): `docs/M35_M39_GENERAL_PURPOSE_EXPANSION_ROADMAP.md`
- Active roadmap (M40-M44): `docs/M40_M44_GENERAL_PURPOSE_PARITY_ROADMAP.md`
- Completed backlogs (M35-M41):
  - `docs/M35_EXECUTION_BACKLOG.md`
  - `docs/M36_EXECUTION_BACKLOG.md`
  - `docs/M37_EXECUTION_BACKLOG.md`
  - `docs/M38_EXECUTION_BACKLOG.md`
  - `docs/M39_EXECUTION_BACKLOG.md`
  - `docs/M40_EXECUTION_BACKLOG.md`
  - `docs/M41_EXECUTION_BACKLOG.md`
- Next backlogs (M42-M44, proposed):
  - `docs/M42_EXECUTION_BACKLOG.md`
  - `docs/M43_EXECUTION_BACKLOG.md`
  - `docs/M44_EXECUTION_BACKLOG.md`
- Last completed backlog (M41): `docs/M41_EXECUTION_BACKLOG.md`
- M35-M39 roadmap execution remains complete, and M40-M44 execution is active
  with M40 evidence-integrity and M41 process/readiness closure.


