# Native Driver Contract v1

Date: 2026-03-11  
Milestone: M53 Native Driver Contract Expansion v1  
Lane: Rugo (Rust kernel + Go user space)  
Status: active gate contract

## Contract identity

Native driver contract ID: `rugo.native_driver_contract.v1`.
Parent lifecycle contract ID: `rugo.driver_lifecycle_report.v6`.
Parent support matrix ID: `rugo.hw.support_matrix.v6`.
PCIe DMA contract ID: `rugo.pcie_dma_contract.v1`.
Firmware blob policy ID: `rugo.firmware_blob_policy.v1`.
Diagnostics schema ID: `rugo.native_driver_diag_schema.v1`.

## Scope

- Freeze the reusable contract for PCIe-native and firmware-bearing drivers
  before M54-M57 broaden support claims.
- Candidate families that inherit this contract: `nvme`, `ahci`,
  `native-gpu`, `wifi-pcie`.
- Existing in-tree classes that inform the shared baseline:
  `virtio-blk-pci`, `virtio-net-pci`, `virtio-scsi-pci`, `virtio-gpu-pci`,
  `e1000e`, `rtl8169`, `xhci`, and `usb-storage`.

## Lifecycle boundary

Required positive-path lifecycle markers:

- `DRV: bind`
- `IRQ: vector bound`
- `DMA: map ok`
- `FW: allow signed`

Required negative-path lifecycle markers:

- `DMA: deny unsafe`
- `FW: denied unsigned`
- `FW: denied missing manifest`
- `FW: denied hash mismatch`

Required lifecycle expectations:

- Probe, bind, init, runtime, and recovery behavior must remain bounded by
  `docs/hw/driver_lifecycle_contract_v6.md`.
- Native-driver bind happens only after bus ownership, BAR arbitration, and IRQ
  policy checks pass.
- MSI or MSI-X selection must be explicit and auditable; fallback to legacy INTx
  is not implied by this contract.
- DMA remains fail-closed until `docs/hw/pcie_dma_contract_v1.md` policy is
  satisfied.
- External firmware remains denied until
  `docs/hw/firmware_blob_policy_v1.md` policy is satisfied.

## Kernel and userspace split

- `kernel_rs/src/` owns probe, bind, IRQ setup, DMA mapping, firmware allow or
  deny decisions, and machine-readable diagnostics emission.
- `services/go/` owns operator-facing policy wiring, diagnostics consumption,
  failure-marker surfacing, and release-lane evidence packaging.
- `services/go_std/` may exercise the diagnostics path, but it does not define
  contract compliance.

## Deterministic failure semantics

- Unsafe DMA paths fail closed with a stable `DMA: deny unsafe` marker.
- Unsigned or untracked firmware fails closed with stable denial markers rather
  than TODO-only behavior.
- Missing diagnostics fields or missing required markers fail the M53 release
  gates.
- Native-driver evidence is a contract surface, not an automatic support claim
  for new hardware rows.

## Diagnostics contract hooks

The machine-readable report defined by
`docs/hw/native_driver_diag_schema_v1.md` must expose:

- bind coverage for the declared shared baseline,
- IRQ and DMA policy outcomes,
- firmware allow and deny outcomes,
- negative-path evidence for unsupported or unsafe operations,
- explicit artifact references for local and CI gates.

## Gate binding

- Local gate: `make test-native-driver-contract-v1`.
- Local sub-gate: `make test-native-driver-diagnostics-v1`.
- CI gate: `Native driver contract v1 gate`.
- CI sub-gate: `Native driver diagnostics v1 gate`.
