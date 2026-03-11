# M60 Execution Backlog (Multi-Device RAID Baseline v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Add bounded multi-device RAID semantics with explicit degraded boot, rebuild,
and scrub behavior for declared storage profiles.

M60 source of truth remains:

- `docs/POST_G2_EXTENDED_MILESTONES.md`
- `docs/M58_EXECUTION_BACKLOG.md`
- `docs/M59_EXECUTION_BACKLOG.md`
- this backlog

## Current State Summary

- M58 and M59 define journaled and encrypted storage semantics needed before
  multi-device behavior can be declared.
- Existing storage work does not yet define mirrored or striped device-set
  policy in a release-gated way.
- There is no versioned RAID contract, degraded boot policy, or rebuild/scrub
  report schema in-tree.
- M60 must add that baseline before integrity-repair features widen storage
  claims further.

## Execution plan

- PR-1: RAID contract freeze
- PR-2: rebuild and scrub campaign baseline
- PR-3: RAID gate wiring + closure

## Execution status

- PR-1: pending
- PR-2: pending
- PR-3: pending

## PR-1: RAID Contract Freeze

### Objective

Define device-set, degraded boot, and rebuild/scrub behavior as explicit
contracts before implementation broadens multi-disk claims.

### Scope

- Add docs:
  - `docs/storage/raid_contract_v1.md`
  - `docs/storage/device_set_policy_v1.md`
  - `docs/storage/rebuild_scrub_policy_v1.md`
- Add tests:
  - `tests/storage/test_raid_docs_v1.py`

### Primary files

- `docs/storage/raid_contract_v1.md`
- `docs/storage/device_set_policy_v1.md`
- `docs/storage/rebuild_scrub_policy_v1.md`
- `tests/storage/test_raid_docs_v1.py`

### Acceptance checks

- `python -m pytest tests/storage/test_raid_docs_v1.py -v`

### Done criteria for PR-1

- RAID topology, degraded boot, and rebuild/scrub semantics are explicit and
  versioned.
- Device-loss, reintegration, and unsupported-topology paths are reviewable
  before implementation lands.

## PR-2: Rebuild and Scrub Campaign Baseline

### Objective

Implement deterministic evidence for mirrored/striped sets, degraded boot, and
scrub/rebuild behavior.

### Scope

- Add tooling:
  - `tools/run_raid_campaign_v1.py`
  - `tools/run_raid_scrub_v1.py`
- Add tests:
  - `tests/storage/test_raid_mirror_rebuild_v1.py`
  - `tests/storage/test_raid_degraded_boot_v1.py`
  - `tests/storage/test_raid_scrub_v1.py`
  - `tests/storage/test_raid_negative_paths_v1.py`

### Primary files

- `tools/run_raid_campaign_v1.py`
- `tools/run_raid_scrub_v1.py`
- `tests/storage/test_raid_mirror_rebuild_v1.py`
- `tests/storage/test_raid_degraded_boot_v1.py`
- `tests/storage/test_raid_scrub_v1.py`
- `tests/storage/test_raid_negative_paths_v1.py`

### Acceptance checks

- `python tools/run_raid_campaign_v1.py --out out/raid-campaign-v1.json`
- `python tools/run_raid_scrub_v1.py --out out/raid-scrub-v1.json`
- `python -m pytest tests/storage/test_raid_mirror_rebuild_v1.py tests/storage/test_raid_degraded_boot_v1.py tests/storage/test_raid_scrub_v1.py tests/storage/test_raid_negative_paths_v1.py -v`

### Done criteria for PR-2

- RAID artifacts are deterministic and machine-readable.
- `RAID: degraded ok` and `RAID: rebuild ok` markers are stable.
- Rebuild and scrub outcomes remain policy-bounded under seeded disk-failure
  scenarios.

## PR-3: RAID Gate + Scrub Sub-gate

### Objective

Make RAID behavior release-blocking for declared multi-device profiles.

### Scope

- Add local gates:
  - `Makefile` target `test-storage-raid-v1`
  - `Makefile` target `test-raid-scrub-v1`
- Add CI steps:
  - `Storage raid v1 gate`
  - `Raid scrub v1 gate`
- Add aggregate tests:
  - `tests/storage/test_storage_raid_gate_v1.py`
  - `tests/storage/test_raid_scrub_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/storage/test_storage_raid_gate_v1.py`
- `tests/storage/test_raid_scrub_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-storage-raid-v1`
- `make test-raid-scrub-v1`

### Done criteria for PR-3

- RAID and scrub sub-gates are required in local and CI release lanes.
- M60 can be marked done only with deterministic degraded-boot and rebuild
  evidence for declared device-set profiles.

## Non-goals for M60 backlog

- full CoW snapshot and self-heal behavior owned by M61
- cluster/distributed storage beyond the declared single-host RAID baseline
- broad hardware promotion without release-gated RAID evidence
