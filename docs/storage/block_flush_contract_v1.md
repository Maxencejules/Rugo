# Block Flush Contract v1

Date: 2026-03-12  
Milestone: M54 Native Storage Drivers v1  
Lane: Rugo (Rust kernel + Go user space)  
Status: active gate contract

## Contract identity

Block flush contract ID: `rugo.block_flush_contract.v1`.
Parent storage contract: `docs/storage/fs_v1.md`.
Parent native storage contract ID: `rugo.nvme_ahci_contract.v1`.
Support matrix ID: `rugo.hw.support_matrix.v7`.

## Purpose

Define the bounded block-layer durability behavior that M54 requires when NVMe
and AHCI controllers are used as the evidence source for `fdatasync` and
`fsync` style claims.

## Required markers

- `BLK: fua ok`
- `BLK: flush ordered`
- `BLK: flush timeout`
- `NVME: ready`
- `AHCI: port up`

## Durability rules

- NVMe FUA completion is the bounded positive-path signal for a write that must
  be reported as durable without a follow-up cache flush.
- AHCI cache flush completion is the bounded positive-path signal for a write
  set that must be reported as durable after queued DMA has reached the device.
- `fdatasync` remains a data-only durability class unless the caller promotes
  the operation into the full `fsync` path.
- `fsync` on a native storage class must name the device class used for the
  evidence artifact and must emit either `BLK: fua ok` or `BLK: flush ordered`.
- Timeout, missing completion, or reordered completion fails closed with
  `BLK: flush timeout`.

## Required report fields

- `device_class`
- `command`
- `latency_ms`
- `data_durable`
- `metadata_durable`
- `marker`
- `status`

## Gate binding

- Local gate: `make test-native-storage-v1`.
- Local sub-gate: `make test-hw-matrix-v7`.
- CI gate: `Native storage v1 gate`.
- CI sub-gate: `Hardware matrix v7 gate`.
