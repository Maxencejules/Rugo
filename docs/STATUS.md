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

Post-G2 planning and execution:
- Extended roadmap (M21-M34): `docs/M21_M34_MATURITY_PARITY_ROADMAP.md`
- Last completed backlog (M21): `docs/M21_EXECUTION_BACKLOG.md`
- Post-M21 focus: start M22 kernel reliability + soak v1 execution backlog.


