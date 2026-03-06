# M15-M20 Multipurpose OS Plan (Post-M14)

Date: 2026-03-04  
Lane: Rugo (Rust kernel + Go user space)  
Status: Proposed

## Goal

Define the next 6 milestones required to move from a strong QEMU-first, contract
and gate driven OS project into a serious multi-purpose OS baseline with:

- broader hardware confidence,
- stronger process and scheduler behavior,
- higher external software compatibility,
- stronger storage and networking operational reliability,
- install, upgrade, and recovery workflows suitable for regular use.

This plan follows the same structure used by M8-M14 execution backlogs.

## Current State Summary

- M0-M14 and G2 are marked complete with local and CI gate wiring.
- Core release gates already exist: runtime, network, storage, release.
- The project remains QEMU-first and heavily virtio-oriented for primary test
  confidence.
- A v0 syscall freeze policy and compatibility profile foundation are in place.

## Milestone Map

| Milestone | Focus | Primary output gate |
|---|---|---|
| M15 | Hardware Enablement Matrix v2 | `test-hw-matrix-v2` |
| M16 | Process + Scheduler Model v2 | `test-process-scheduler-v2` |
| M17 | Compatibility Profile v2 | `test-compat-v2` |
| M18 | Storage Reliability v2 | `test-storage-reliability-v2` |
| M19 | Network Stack v2 | `test-network-stack-v2` |
| M20 | Operability + Release UX v2 | `test-release-ops-v2` |

## Proposed Makefile Target Stubs

```make
# M15

test-hw-matrix-v2: image-blk image-net
	$(PYTHON) -m pytest tests/hw/test_hardware_matrix_v2.py tests/hw/test_probe_negative_paths_v2.py tests/hw/test_dma_iommu_policy_v2.py tests/hw/test_acpi_boot_paths_v2.py tests/hw/test_bare_metal_smoke_v2.py tests/hw/test_hw_gate_v2.py -v

# M16

test-process-scheduler-v2: image-thread-spawn image-thread-exit image-yield image-user-fault
	$(PYTHON) -m pytest tests/sched/test_preempt_timer_quantum_v2.py tests/sched/test_priority_fairness_v2.py tests/sched/test_scheduler_soak_v2.py tests/user/test_process_wait_kill_v2.py tests/user/test_signal_delivery_v2.py tests/sched/test_scheduler_gate_v2.py -v

# M17

test-compat-v2: image-go-std image-pkg-hash
	$(PYTHON) -m pytest tests/compat/test_abi_profile_v2_docs.py tests/compat/test_elf_loader_dynamic_v2.py tests/compat/test_posix_profile_v2.py tests/compat/test_external_apps_tier_v2.py tests/compat/test_compat_gate_v2.py -v

# M18

test-storage-reliability-v2: image-fs image-fs-badmagic
	$(PYTHON) tools/storage_recover_v2.py --check --out out/storage-recovery-v2.json
	$(PYTHON) tools/run_storage_powerfail_campaign_v2.py --seed 20260304 --out out/storage-powerfail-v2.json
	$(PYTHON) -m pytest tests/storage/test_journal_recovery_v2.py tests/storage/test_powerfail_campaign_v2.py tests/storage/test_metadata_integrity_v2.py tests/storage/test_storage_gate_v2.py -v

# M19

test-network-stack-v2: image-net
	$(PYTHON) tools/run_net_interop_matrix_v2.py --out out/net-interop-v2.json
	$(PYTHON) tools/run_net_soak_v2.py --out out/net-soak-v2.json
	$(PYTHON) -m pytest tests/net/test_tcp_interop_v2.py tests/net/test_ipv6_interop_v2.py tests/net/test_dns_stub_v2.py tests/net/test_network_gate_v2.py -v

# M20

test-release-ops-v2: image
	$(PYTHON) tools/build_installer_v2.py --out out/installer-v2.json
	$(PYTHON) tools/run_upgrade_recovery_drill_v2.py --out out/upgrade-recovery-v2.json
	$(PYTHON) tools/collect_support_bundle_v2.py --out out/support-bundle-v2.json
	$(PYTHON) -m pytest tests/build/test_installer_recovery_v2.py tests/build/test_upgrade_rollback_v2.py tests/build/test_support_bundle_v2.py tests/build/test_operability_gate_v2.py -v
```

## M15: Hardware Enablement Matrix v2

### Objective

Move from QEMU-first confidence to a tiered hardware confidence model with
repeatable evidence across representative real hardware classes.

### Execution plan (3 PRs)

#### PR-1: Matrix v2 contract + target classes

- Add docs:
  - `docs/hw/support_matrix_v2.md`
  - `docs/hw/device_profile_contract_v2.md`
- Add tests:
  - `tests/hw/test_hardware_matrix_v2.py`
  - `tests/hw/test_probe_negative_paths_v2.py`
- Define Tier 0, Tier 1, Tier 2 pass/fail criteria with explicit artifact schema.

Acceptance checks:

- `python -m pytest tests/hw/test_hardware_matrix_v2.py tests/hw/test_probe_negative_paths_v2.py -v`

#### PR-2: Driver lifecycle and DMA policy hardening v2

- Add docs:
  - `docs/hw/dma_iommu_strategy_v2.md`
  - `docs/hw/acpi_uefi_hardening_v2.md`
- Add tests:
  - `tests/hw/test_dma_iommu_policy_v2.py`
  - `tests/hw/test_acpi_boot_paths_v2.py`
- Enforce probe/init/runtime-health failure semantics across supported classes.

Acceptance checks:

- `python -m pytest tests/hw/test_dma_iommu_policy_v2.py tests/hw/test_acpi_boot_paths_v2.py -v`

#### PR-3: Bare-metal lane + release gate

- Add docs:
  - `docs/hw/bare_metal_bringup_v2.md`
- Add tests:
  - `tests/hw/test_bare_metal_smoke_v2.py`
  - `tests/hw/test_hw_gate_v2.py`
- Add local gate:
  - `Makefile` target `test-hw-matrix-v2`
- Add CI gate:
  - `.github/workflows/ci.yml` step `Hardware matrix v2 gate`

Acceptance checks:

- `make test-hw-matrix-v2`

Done criteria for M15:

- Tiered matrix is versioned and gate-enforced.
- Storage and network smoke pass deterministically on all Tier 0 and Tier 1 targets.
- Hardware claims are constrained to matrix evidence.

## M16: Process + Scheduler Model v2

### Objective

Upgrade from baseline cooperative behavior to stronger preemption, fairness, and
process lifecycle semantics suitable for mixed workloads.

### Execution plan (3 PRs)

#### PR-1: Process/thread contract v2

- Add docs:
  - `docs/abi/process_thread_model_v2.md`
  - `docs/abi/scheduling_policy_v2.md`
- Add tests:
  - `tests/user/test_process_wait_kill_v2.py`
  - `tests/user/test_signal_delivery_v2.py`
- Freeze process lifecycle semantics for v2 profile.

Acceptance checks:

- `python -m pytest tests/user/test_process_wait_kill_v2.py tests/user/test_signal_delivery_v2.py -v`

#### PR-2: Scheduler behavior and fairness

- Add tests:
  - `tests/sched/test_preempt_timer_quantum_v2.py`
  - `tests/sched/test_priority_fairness_v2.py`
  - `tests/sched/test_scheduler_soak_v2.py`
- Add model helper:
  - `tests/sched/v2_model.py`

Acceptance checks:

- `python -m pytest tests/sched/test_preempt_timer_quantum_v2.py tests/sched/test_priority_fairness_v2.py tests/sched/test_scheduler_soak_v2.py -v`
#### PR-3: Scheduler gate and milestone closure

- Add aggregate gate test:
  - `tests/sched/test_scheduler_gate_v2.py`
- Add local gate:
  - `Makefile` target `test-process-scheduler-v2`
- Add CI gate:
  - `.github/workflows/ci.yml` step `Process scheduler v2 gate`

Acceptance checks:

- `make test-process-scheduler-v2`

Done criteria for M16:

- Process/thread semantics are versioned and enforced by tests.
- Preemption and fairness regressions are gate-blocking.
- Faulted user tasks are contained without scheduler collapse.

## M17: Compatibility Profile v2

### Objective

Increase external software viability by expanding compatibility contracts and
validating a curated external app lane from signed packages.

### Execution plan (3 PRs)

#### PR-1: ABI and loader contract v2

- Add docs:
  - `docs/abi/syscall_v2.md`
  - `docs/abi/compat_profile_v2.md`
  - `docs/abi/elf_loader_contract_v2.md`
- Add tests:
  - `tests/compat/test_abi_profile_v2_docs.py`
  - `tests/compat/test_elf_loader_dynamic_v2.py`

Acceptance checks:

- `python -m pytest tests/compat/test_abi_profile_v2_docs.py tests/compat/test_elf_loader_dynamic_v2.py -v`

#### PR-2: POSIX subset expansion and app tier

- Add docs:
  - `docs/runtime/syscall_coverage_matrix_v2.md`
- Add tests:
  - `tests/compat/test_posix_profile_v2.py`
  - `tests/compat/test_external_apps_tier_v2.py`
- Add fixtures:
  - `tests/compat/v2_model.py`

Acceptance checks:

- `python -m pytest tests/compat/test_posix_profile_v2.py tests/compat/test_external_apps_tier_v2.py -v`

#### PR-3: Compatibility gate and CI promotion

- Add aggregate gate test:
  - `tests/compat/test_compat_gate_v2.py`
- Add local gate:
  - `Makefile` target `test-compat-v2`
- Add CI gate:
  - `.github/workflows/ci.yml` step `Compatibility profile v2 gate`

Acceptance checks:

- `make test-compat-v2`

Done criteria for M17:

- Compatibility profile v2 has explicit supported and unsupported surfaces.
- External app tier has deterministic pass thresholds and signed artifact inputs.
- ABI v2 changes follow documented deprecation and freeze rules.

## M18: Storage Reliability v2

### Objective

Raise data safety from v1 baseline to stronger crash consistency and power-fault
resilience suitable for multi-purpose usage.

### Execution plan (3 PRs)

#### PR-1: Storage contract and journaling policy v2

- Add docs:
  - `docs/storage/fs_v2.md`
  - `docs/storage/durability_model_v2.md`
  - `docs/storage/write_ordering_policy_v2.md`
- Add tests:
  - `tests/storage/test_journal_recovery_v2.py`
  - `tests/storage/test_metadata_integrity_v2.py`

Acceptance checks:

- `python -m pytest tests/storage/test_journal_recovery_v2.py tests/storage/test_metadata_integrity_v2.py -v`

#### PR-2: Recovery and powerfail campaign v2

- Add tooling:
  - `tools/storage_recover_v2.py`
  - `tools/run_storage_powerfail_campaign_v2.py`
- Add tests:
  - `tests/storage/test_powerfail_campaign_v2.py`
- Add docs:
  - `docs/storage/recovery_playbook_v2.md`
  - `docs/storage/fault_campaign_v2.md`

Acceptance checks:

- `python tools/storage_recover_v2.py --check --out out/storage-recovery-v2.json`
- `python tools/run_storage_powerfail_campaign_v2.py --seed 20260304 --out out/storage-powerfail-v2.json`
- `python -m pytest tests/storage/test_powerfail_campaign_v2.py -v`

#### PR-3: Storage v2 gate and closure

- Add aggregate gate test:
  - `tests/storage/test_storage_gate_v2.py`
- Add local gate:
  - `Makefile` target `test-storage-reliability-v2`
- Add CI gate:
  - `.github/workflows/ci.yml` step `Storage reliability v2 gate`

Acceptance checks:

- `make test-storage-reliability-v2`

Done criteria for M18:

- Recovery and powerfail artifacts are stable and machine-readable.
- Storage v2 gate blocks release on durability regression.
- Crash consistency guarantees are explicit and test-backed.
## M19: Network Stack v2

### Objective

Deliver practical network behavior for common multi-purpose workloads with stronger
interop and soak guarantees.

### Execution plan (3 PRs)

#### PR-1: Protocol and socket contract v2

- Add docs:
  - `docs/net/network_stack_contract_v2.md`
  - `docs/net/socket_contract_v2.md`
  - `docs/net/tcp_profile_v2.md`
- Add tests:
  - `tests/net/test_tcp_interop_v2.py`
  - `tests/net/test_ipv6_interop_v2.py`

Acceptance checks:

- `python -m pytest tests/net/test_tcp_interop_v2.py tests/net/test_ipv6_interop_v2.py -v`

#### PR-2: Service behavior and diagnostics

- Add tooling:
  - `tools/run_net_interop_matrix_v2.py`
  - `tools/run_net_soak_v2.py`
- Add tests:
  - `tests/net/test_dns_stub_v2.py`
- Add docs:
  - `docs/net/interop_matrix_v2.md`

Acceptance checks:

- `python tools/run_net_interop_matrix_v2.py --out out/net-interop-v2.json`
- `python tools/run_net_soak_v2.py --out out/net-soak-v2.json`
- `python -m pytest tests/net/test_dns_stub_v2.py -v`

#### PR-3: Network gate and closure

- Add aggregate gate test:
  - `tests/net/test_network_gate_v2.py`
- Add local gate:
  - `Makefile` target `test-network-stack-v2`
- Add CI gate:
  - `.github/workflows/ci.yml` step `Network stack v2 gate`

Acceptance checks:

- `make test-network-stack-v2`

Done criteria for M19:

- Interop and soak thresholds are explicit and release-blocking.
- Socket and protocol contracts are stable and versioned.
- Network diagnostics and artifact schemas are machine-readable.

## M20: Operability + Release UX v2

### Objective

Close the final gap between engineering milestones and practical day-to-day OS
operation: install, upgrade, rollback, recovery, and supportability.

### Execution plan (3 PRs)

#### PR-1: Installer and operational contract v2

- Add docs:
  - `docs/build/installer_recovery_baseline_v2.md`
  - `docs/build/operations_runbook_v2.md`
- Add tooling:
  - `tools/build_installer_v2.py`
- Add tests:
  - `tests/build/test_installer_recovery_v2.py`

Acceptance checks:

- `python tools/build_installer_v2.py --out out/installer-v2.json`
- `python -m pytest tests/build/test_installer_recovery_v2.py -v`

#### PR-2: Upgrade and rollback drill v2

- Add docs:
  - `docs/pkg/update_protocol_v2.md`
  - `docs/pkg/rollback_policy_v2.md`
- Add tooling:
  - `tools/run_upgrade_recovery_drill_v2.py`
  - `tools/collect_support_bundle_v2.py`
- Add tests:
  - `tests/build/test_upgrade_rollback_v2.py`
  - `tests/build/test_support_bundle_v2.py`

Acceptance checks:

- `python tools/run_upgrade_recovery_drill_v2.py --out out/upgrade-recovery-v2.json`
- `python tools/collect_support_bundle_v2.py --out out/support-bundle-v2.json`
- `python -m pytest tests/build/test_upgrade_rollback_v2.py tests/build/test_support_bundle_v2.py -v`

#### PR-3: Operability gate and final closure

- Add aggregate gate test:
  - `tests/build/test_operability_gate_v2.py`
- Add local gate:
  - `Makefile` target `test-release-ops-v2`
- Add CI gate:
  - `.github/workflows/ci.yml` step `Operability and release UX v2 gate`
- Update milestone/status docs after gate passes:
  - `MILESTONES.md`
  - `docs/STATUS.md`
  - `README.md`

Acceptance checks:

- `make test-release-ops-v2`

Done criteria for M20:

- Install, upgrade, rollback, and recovery flows are test-backed and repeatable.
- Support bundle schema is stable and useful for incident triage.
- Operability gate is required for release qualification.

## Exact Proposed Test File Inventory

### M15 tests

- `tests/hw/test_hardware_matrix_v2.py`
- `tests/hw/test_probe_negative_paths_v2.py`
- `tests/hw/test_dma_iommu_policy_v2.py`
- `tests/hw/test_acpi_boot_paths_v2.py`
- `tests/hw/test_bare_metal_smoke_v2.py`
- `tests/hw/test_hw_gate_v2.py`

### M16 tests

- `tests/user/test_process_wait_kill_v2.py`
- `tests/user/test_signal_delivery_v2.py`
- `tests/sched/test_preempt_timer_quantum_v2.py`
- `tests/sched/test_priority_fairness_v2.py`
- `tests/sched/test_scheduler_soak_v2.py`
- `tests/sched/test_scheduler_gate_v2.py`

### M17 tests

- `tests/compat/test_abi_profile_v2_docs.py`
- `tests/compat/test_elf_loader_dynamic_v2.py`
- `tests/compat/test_posix_profile_v2.py`
- `tests/compat/test_external_apps_tier_v2.py`
- `tests/compat/test_compat_gate_v2.py`

### M18 tests

- `tests/storage/test_journal_recovery_v2.py`
- `tests/storage/test_metadata_integrity_v2.py`
- `tests/storage/test_powerfail_campaign_v2.py`
- `tests/storage/test_storage_gate_v2.py`

### M19 tests

- `tests/net/test_tcp_interop_v2.py`
- `tests/net/test_ipv6_interop_v2.py`
- `tests/net/test_dns_stub_v2.py`
- `tests/net/test_network_gate_v2.py`

### M20 tests

- `tests/build/test_installer_recovery_v2.py`
- `tests/build/test_upgrade_rollback_v2.py`
- `tests/build/test_support_bundle_v2.py`
- `tests/build/test_operability_gate_v2.py`

## Recommended Sequencing and Exit Rule

Recommended sequencing remains strict:

- M15 -> M16 -> M17 -> M18 -> M19 -> M20

Exit rule for declaring "serious multi-purpose baseline":

1. All six new gates are green in local and CI lanes.
2. Hardware claims are restricted to matrix-passing tiers.
3. Compatibility profile v2 external app tier has a stable pass threshold.
4. Storage and network v2 soak and fault artifacts are published per release.
5. Install/upgrade/recovery/supportability drills are release-blocking.

## Non-goals for this plan

- Full Linux distribution parity in package ecosystem breadth.
- Immediate broad desktop UX scope beyond operability baselines.
- Broad non-x86 architecture support in the same milestone window unless
  explicitly re-scoped.
