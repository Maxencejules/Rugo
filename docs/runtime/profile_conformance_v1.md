# Profile Conformance v1

Date: 2026-03-09  
Milestone: M32 Conformance + Profile Qualification v1  
Status: active release gate

## Purpose

Define deterministic, machine-verifiable profile qualification for release
lanes and require profile conformance evidence in release artifacts.

## Conformance identifiers

- Conformance policy ID: `rugo.profile_conformance_policy.v1`
- Conformance report schema: `rugo.profile_conformance_report.v1`
- Profile requirement schema: `rugo.profile_requirement_set.v1`
- Qualification digest algorithm: `sha256`

## Profiles in scope

- `server_v1`
  - optimized for long-running, networked service workloads
- `developer_v1`
  - optimized for interactive build/test/debug workflows
- `appliance_v1`
  - optimized for fixed-purpose deployment with minimized mutable state

## Server profile requirements (`server_v1`)

- `service_restart_coverage_pct` must be `>= 95`.
- `max_crash_recovery_seconds` must be `<= 30`.
- `network_ipv6_enabled` must be `== 1`.
- `supply_chain_attestation` must be `== 1`.

## Developer profile requirements (`developer_v1`)

- `toolchain_smoke_pass` must be `== 1`.
- `debug_symbols_available` must be `== 1`.
- `package_build_success_rate_pct` must be `>= 98`.
- `interactive_shell_latency_ms_p95` must be `<= 200`.

## Appliance profile requirements (`appliance_v1`)

- `immutable_rootfs_enforced` must be `== 1`.
- `read_only_runtime_pct` must be `>= 99`.
- `boot_to_service_seconds_p95` must be `<= 45`.
- `remote_mgmt_surface_minimized` must be `== 1`.

## Tooling and gate wiring

- Conformance suite tool: `tools/run_conformance_suite_v1.py`
- Default deterministic seed: `20260309`
- Runtime capture artifact: `out/booted-runtime-v1.json`
- Release attestation artifact: `out/release-attestation-verification-v1.json`
- Package rebuild artifact: `out/pkg-rebuild-v3.json`
- LTS-qualified profiles: `server_v1`, `appliance_v1`
- Non-LTS profile: `developer_v1`
- Local gate: `make test-conformance-v1`
- CI gate: `Conformance v1 gate`
- Release artifact: `out/conformance-v1.json`
- Gate outcome: `total_failures` must be `0`.

## Required executable checks

- `tests/runtime/test_profile_conformance_docs_v1.py`
- `tests/runtime/test_server_profile_v1.py`
- `tests/runtime/test_dev_profile_v1.py`
- `tests/runtime/test_conformance_gate_v1.py`
