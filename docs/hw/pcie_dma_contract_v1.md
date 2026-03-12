# PCIe DMA Contract v1

Date: 2026-03-11  
Milestone: M53 Native Driver Contract Expansion v1  
Lane: Rugo (Rust kernel + Go user space)  
Status: active gate contract

## Contract identity

PCIe DMA contract ID: `rugo.pcie_dma_contract.v1`.
Parent native driver contract ID: `rugo.native_driver_contract.v1`.
Parent lifecycle contract ID: `rugo.driver_lifecycle_report.v6`.
Parent support matrix ID: `rugo.hw.support_matrix.v6`.

## Policy modes

Supported DMA policy modes:

- `strict`
- `passthrough-shadow`
- `deny`

Mode rules:

- `strict` is the release-floor mode and requires an auditable IOMMU domain for
  every native-driver mapping.
- `passthrough-shadow` is evidence-only and cannot satisfy release claims.
- `deny` is the deterministic fallback when the platform cannot prove safe
  mapping semantics.

## Required behavior

- DMA remains fail-closed until a valid IOMMU domain, alignment check, and
  aperture check succeed.
- `DMA: map ok` is emitted only for mappings inside the allowed DMA window.
- `DMA: map bounce` is emitted when a bounded bounce-buffer path is required.
- `DMA: deny unsafe` is emitted for peer-to-peer DMA, out-of-window mappings,
  or untracked physical addresses.
- `IRQ: vector bound` must accompany successful queue activation for DMA-backed
  rings or descriptors.
- Software validation mandatory even when hardware isolation is available.
- peer-to-peer DMA remains denied unless a later contract explicitly promotes it.

## Required audit fields

- `iommu_mode`
- `iommu_domain_id`
- `dma_window_bytes`
- `bounce_buffer_used`
- `map_kind`
- `marker`
- `status`
- `unsafe_reason`

## Negative-path requirements

- Out-of-window mappings must fail with a stable denial reason.
- Mappings without ownership records must fail with a stable denial reason.
- Firmware-supplied DMA addresses are not trusted by default.
- The diagnostics report must keep denial markers machine-readable for local and
  CI triage.

## Executable conformance

- `tests/hw/test_pcie_dma_contract_v1.py`
- `tests/hw/test_irq_dma_policy_v1.py`

## Gate binding

- Local gate: `make test-native-driver-contract-v1`.
- Local sub-gate: `make test-native-driver-diagnostics-v1`.
