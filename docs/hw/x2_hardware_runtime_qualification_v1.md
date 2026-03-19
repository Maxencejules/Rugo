# X2 Hardware Runtime Qualification v1

Date: 2026-03-18  
Track: X2 Hardware, Firmware, and Driver Breadth  
Lane: Rugo (Rust kernel + Go user space)  
Status: active aggregate gate

## Goal

Turn the historical X2 support matrices into one shared runtime-backed
qualification bundle with:

- a reusable device registry,
- probe and bind lifecycle evidence,
- firmware and measured-boot checks,
- SMP-safe driver/runtime expectations,
- declared target-class qualification for virtual and bare-metal lanes.

## Report identity

Qualification report schema: `rugo.x2_hardware_runtime_report.v1`.
Qualification policy ID: `rugo.x2_hardware_runtime_qualification.v1`.
Device registry schema: `rugo.x2_device_registry.v1`.

## Historical backlog coverage

The X2 runtime-backed closure covers:

- `M9`
- `M15`
- `M23`
- `M37`
- `M43`
- `M45`
- `M46`
- `M47`

Each backlog must appear in `backlog_closure` with `Runtime-backed` status and
named `target_classes`.

## Required top-level fields

- `schema`
- `created_utc`
- `track_id`
- `policy_id`
- `device_registry_schema`
- `seed`
- `gate`
- `checks`
- `summary`
- `backlog_closure`
- `device_registry`
- `probe_bind_lifecycle`
- `firmware_runtime`
- `smp_runtime`
- `runtime_targets`
- `source_reports`
- `artifact_refs`
- `total_failures`
- `gate_pass`
- `digest`

## Device registry foundation

The `device_registry` must include at least these class IDs:

- `virtio-blk-pci-transitional`
- `virtio-net-pci-transitional`
- `ahci`
- `nvme`
- `virtio-blk-pci-modern`
- `virtio-net-pci-modern`
- `virtio-scsi-pci`
- `virtio-gpu-pci`
- `e1000e`
- `rtl8169`
- `xhci`
- `usb-hid`
- `usb-storage`

Each registry row must bind:

- `source_milestones`
- `qualified_tiers`
- `required_states`
- `required_markers`
- `states_observed`
- `claim_status`

## Probe and bind lifecycle

`probe_bind_lifecycle` must carry the reusable driver or device foundation:

- probe discovery state,
- runtime state,
- bind markers such as `DRV: bind`,
- IRQ or DMA policy context when a bind report exists,
- SMP-safe balancing evidence through `cpu_affinity_balance`,
- per-row `status`.

## Firmware and SMP requirements

`firmware_runtime` must bind all of:

- measured boot report `rugo.measured_boot_report.v1`,
- firmware plus SMP evidence `rugo.hw_firmware_smp_evidence.v1`,
- firmware blob policy `rugo.firmware_blob_policy.v1`.

`smp_runtime` must expose:

- `bootstrap_cpu_online_ratio`
- `application_cpu_online_ratio`
- `ipi_roundtrip_p95_ms`
- `lost_interrupt_events`
- required markers including `SMP: affinity balanced`

## Declared runtime targets

`runtime_targets` must qualify these target IDs:

- `qemu-q35-transitional`
- `qemu-i440fx-transitional`
- `qemu-q35-firmware-smp`
- `qemu-i440fx-firmware-smp`
- `qemu-q35-modern-virtio`
- `qemu-i440fx-modern-virtio`
- `intel-q470-e1000e-xhci`
- `amd-b550-rtl8169-xhci`

Each target must carry:

- `required_devices`
- `boot_markers`
- `runtime_markers`
- `capture`
- `qualification_pass`

## Supporting source reports

The aggregate report must bind the historical X2 lane to these supporting
artifacts:

- `out/measured-boot-v1.json`
- `out/hw-diagnostics-v3.json`
- `out/hw-matrix-v4.json`
- `out/hw-firmware-smp-v1.json`
- `out/hw-matrix-v6.json`
- `out/baremetal-io-v1.json`
- `out/hw-claim-promotion-v1.json`
- `out/native-driver-diagnostics-v1.json`

## Gate binding

- Local gate: `make test-x2-hardware-runtime-v1`.
- CI gate: `X2 hardware runtime v1 gate`.
- CI artifact: `x2-hardware-runtime-v1-artifacts`.
- Primary report: `out/x2-hardware-runtime-v1.json`.
