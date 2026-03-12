# Firmware Blob Policy v1

Date: 2026-03-11  
Milestone: M53 Native Driver Contract Expansion v1  
Lane: Rugo (Rust kernel + Go user space)  
Status: active gate policy

## Policy identity

Firmware blob policy ID: `rugo.firmware_blob_policy.v1`.
Parent native driver contract ID: `rugo.native_driver_contract.v1`.
Parent PCIe DMA contract ID: `rugo.pcie_dma_contract.v1`.
Diagnostics schema ID: `rugo.native_driver_diag_schema.v1`.
Firmware manifest schema: `rugo.firmware_manifest.v1`.

## Core rules

- firmware remains outside the base kernel image and is pinned by a signed
  manifest.
- measured-boot reference required for firmware that can influence native
  driver behavior before userspace policy takes over.
- allowlist ownership is explicit per device family and blob identifier.
- License and provenance metadata are required, not optional notes.

## Required allow and deny markers

- `FW: allow signed`
- `FW: denied unsigned`
- `FW: denied missing manifest`
- `FW: denied hash mismatch`

## Allow policy

- Signed firmware with a matching hash, approved manifest, and allowlisted blob
  ID may load.
- The manifest must carry vendor, blob ID, SHA-256 digest, signature profile,
  license tag, and measured-boot linkage.
- Firmware load decisions must be replayable from diagnostics artifacts without
  re-reading the binary blob.

## Deny policy

- Unsigned firmware is denied with `FW: denied unsigned`.
- Missing manifest records are denied with `FW: denied missing manifest`.
- Hash or measurement drift is denied with `FW: denied hash mismatch`.
- Emergency bypasses are out of scope for M53 and cannot silently override the
  release floor.

## Required audit fields

- `blob_id`
- `device_class`
- `manifest_present`
- `signature_valid`
- `hash_match`
- `measured_boot_ref`
- `decision`
- `marker`
- `status`

## Executable conformance

- `tests/hw/test_firmware_blob_policy_v1.py`
- `tests/hw/test_firmware_blob_enforcement_v1.py`

## Gate binding

- Local gate: `make test-native-driver-contract-v1`.
- Local sub-gate: `make test-native-driver-diagnostics-v1`.
