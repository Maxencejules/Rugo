# Product Alpha Qualification v1

Date: 2026-03-19  
Track: Product qualification  
Status: active

This doc turns the `Alpha` section of [../RUGO_V1_PRODUCT.md](../RUGO_V1_PRODUCT.md)
into an executable repo gate.

Qualification report schema: `rugo.product_alpha_qualification_report.v1`.
Qualification policy ID: `rugo.product_alpha_qualification.v1`.

## Declared Alpha Candidate

- Alpha candidate boot image: `out/os-go-desktop-native.iso`.
- Alpha candidate kernel image: `out/kernel-go-desktop-native.elf`.
- Panic validation image: `out/os-panic.iso`.
- Runtime capture schema: `rugo.booted_runtime_capture.v1`.
- Primary runtime capture: `out/product-alpha-runtime-capture-v1.json`.
- Primary report: `out/product-alpha-v1.json`.
- Primary profile ID: `qemu-q35-default-desktop`.
- Machine profile: `q35`.
- CPU profile: `qemu64,+x2apic`.
- Storage class: `nvme`.
- Network class: `wired-virtio-net`.

## Tooling

- Runtime tool: `tools/run_product_alpha_qualification_v1.py`.
- Shared helper: `tools/product_alpha_common_v1.py`.
- Local gate: `make test-product-alpha-v1`.
- CI gate: `Product alpha v1 gate`.
- CI artifact: `product-alpha-v1-artifacts`.
- JUnit output: `out/pytest-product-alpha-v1.xml`.

## Required Alpha Checks

- `bootable_default_image`
- `durable_nvme_storage`
- `wired_networking`
- `desktop_or_shell_boot`
- `install_path`
- `update_path`
- `recovery_path`
- `diagnostics_path`

## Generated Evidence

- `out/product-alpha-x4-runtime-v1.json`
- `out/product-alpha-x3-runtime-v1.json`
- `out/product-alpha-graphical-installer-v1.json`
- `out/product-alpha-release-bundle-v1.json`
- `out/product-alpha-installer-v2.json`
- `out/product-alpha-install-state-v1.json`
- `out/product-alpha-update-metadata-v1.json`
- `out/product-alpha-upgrade-drill-v3.json`
- `out/product-alpha-recovery-drill-v3.json`
- `out/product-alpha-trace-bundle-v2.json`
- `out/product-alpha-diagnostic-snapshot-v2.json`
- `out/product-alpha-crash-dump-v1.json`
- `out/product-alpha-crash-dump-symbolized-v1.json`
- `out/product-alpha-supporting/`
