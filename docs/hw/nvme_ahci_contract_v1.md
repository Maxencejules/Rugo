# NVMe + AHCI Contract v1

Date: 2026-03-12  
Milestone: M54 Native Storage Drivers v1  
Lane: Rugo (Rust kernel + Go user space)  
Status: active gate contract

## Contract identity

Native storage contract ID: `rugo.nvme_ahci_contract.v1`.
Parent native driver contract ID: `rugo.native_driver_contract.v1`.
Driver lifecycle contract ID: `rugo.driver_lifecycle_report.v6`.
Support matrix ID: `rugo.hw.support_matrix.v7`.
Block flush contract ID: `rugo.block_flush_contract.v1`.

## Scope

- Freeze the bounded storage-driver semantics that let M54 move beyond the
  virtio-only release floor without turning AHCI and NVMe into open-ended
  support claims.
- Covered controller families:
  - `nvme`
  - `ahci`
- Covered queue and recovery surfaces:
  - NVMe admin identify and namespace discovery
  - NVMe I/O queue submission and completion
  - AHCI port link-up, DMA read or write, and cache flush
  - controller reset recovery with deterministic serial markers

## Required positive-path markers

- `NVME: ready`
- `NVME: identify ok`
- `NVME: io queue ok`
- `NVME: reset recover`
- `AHCI: port up`
- `AHCI: rw ok`
- `AHCI: flush ok`
- `BLK: fua ok`
- `BLK: flush ordered`

## Required negative-path markers

- `NVME: namespace missing`
- `AHCI: port absent`
- `BLK: flush timeout`

## Queue, reset, and power boundary

- NVMe must expose one deterministic admin queue and at least one deterministic
  I/O queue before the driver is considered ready.
- Namespace geometry is pinned and auditable; dynamic namespace discovery does
  not broaden support claims beyond the declared report rows.
- AHCI readiness requires at least one linked port with stable DMA read or
  write completion and an auditable cache-flush path.
- Reset handling must fail closed with stable markers if queue or port state
  does not recover into a known-ready state.
- Power management remains bounded to deterministic startup and recovery hooks;
  opportunistic runtime power tuning is out of scope for this contract.

## Kernel and userspace split

- `kernel_rs/src/` owns probe, queue setup, namespace or port discovery, IRQ
  setup, reset recovery, and flush completion semantics.
- `services/go/` owns storage diagnostics collection, device-class reporting,
  durability-evidence packaging, and operator-facing failure markers.
- `services/go_std/` may exercise the path for comparison only; it does not
  define compliance for this contract.

## Flush and durability hooks

- `docs/storage/block_flush_contract_v1.md` defines the normative block flush
  and FUA behavior consumed by `fsync` and `fdatasync` style durability paths.
- NVMe FUA completion must not be reported as successful until the bounded
  durability contract names the write as durable.
- AHCI cache flush completion must remain ordered relative to DMA writes and
  must emit a stable success or timeout marker.
- Missing or ambiguous flush markers fail the M54 release gates.

## Gate binding

- Local gate: `make test-native-storage-v1`.
- Local sub-gate: `make test-hw-matrix-v7`.
- CI gate: `Native storage v1 gate`.
- CI sub-gate: `Hardware matrix v7 gate`.
