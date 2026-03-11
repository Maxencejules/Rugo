# M59 Execution Backlog (Encryption + Keyslot Baseline v1)

Date: 2026-03-11  
Lane: Rugo (Rust kernel + Go user space)  
Status: planned

## Goal

Add explicit at-rest encryption and keyslot semantics without weakening storage
reliability, reproducibility, or the existing capability model.

M59 source of truth remains:

- `docs/POST_G2_EXTENDED_MILESTONES.md`
- `docs/M58_EXECUTION_BACKLOG.md`
- `docs/security/rights_capability_model_v1.md`
- this backlog

## Current State Summary

- M58 establishes the crash-consistency foundation needed for encrypted
  volumes.
- Security hardening and update trust policies exist, but storage encryption is
  not yet a declared contract surface.
- There is no versioned encrypted-volume, keyslot, or storage-crypto syscall
  contract in-tree.
- M59 must close those gaps before multi-device and snapshot work assumes
  encrypted storage semantics.

## Execution plan

- PR-1: encrypted-volume and key policy freeze
- PR-2: unlock and recovery campaign baseline
- PR-3: encryption gate wiring + closure

## Execution status

- PR-1: pending
- PR-2: pending
- PR-3: pending

## PR-1: Encrypted-volume and Key Policy Freeze

### Objective

Define encrypted-volume, keyslot, and storage-crypto ABI behavior before
implementation lands.

### Scope

- Add docs:
  - `docs/storage/encrypted_volume_contract_v1.md`
  - `docs/security/storage_key_policy_v1.md`
  - `docs/abi/storage_crypto_syscalls_v1.md`
- Add tests:
  - `tests/storage/test_encrypted_volume_docs_v1.py`
  - `tests/security/test_storage_key_policy_v1.py`

### Primary files

- `docs/storage/encrypted_volume_contract_v1.md`
- `docs/security/storage_key_policy_v1.md`
- `docs/abi/storage_crypto_syscalls_v1.md`
- `tests/storage/test_encrypted_volume_docs_v1.py`
- `tests/security/test_storage_key_policy_v1.py`

### Acceptance checks

- `python -m pytest tests/storage/test_encrypted_volume_docs_v1.py tests/security/test_storage_key_policy_v1.py -v`

### Done criteria for PR-1

- Encryption, unlock, and recovery-key behavior are explicit and versioned.
- Secret-handling and revoked-key paths are described as deterministic failures.

## PR-2: Unlock and Recovery Campaign Baseline

### Objective

Implement deterministic evidence for unlock, power-fail recovery, and keyslot
rotation behavior.

### Scope

- Add tooling:
  - `tools/run_volume_unlock_campaign_v1.py`
  - `tools/run_storage_key_rotation_drill_v1.py`
- Add tests:
  - `tests/storage/test_volume_unlock_v1.py`
  - `tests/storage/test_encrypted_powerfail_recovery_v1.py`
  - `tests/security/test_storage_keyslot_rotation_v1.py`
  - `tests/storage/test_storage_crypto_negative_v1.py`

### Primary files

- `tools/run_volume_unlock_campaign_v1.py`
- `tools/run_storage_key_rotation_drill_v1.py`
- `tests/storage/test_volume_unlock_v1.py`
- `tests/storage/test_encrypted_powerfail_recovery_v1.py`
- `tests/security/test_storage_keyslot_rotation_v1.py`
- `tests/storage/test_storage_crypto_negative_v1.py`

### Acceptance checks

- `python tools/run_volume_unlock_campaign_v1.py --out out/volume-unlock-v1.json`
- `python tools/run_storage_key_rotation_drill_v1.py --out out/storage-key-rotation-v1.json`
- `python -m pytest tests/storage/test_volume_unlock_v1.py tests/storage/test_encrypted_powerfail_recovery_v1.py tests/security/test_storage_keyslot_rotation_v1.py tests/storage/test_storage_crypto_negative_v1.py -v`

### Done criteria for PR-2

- Encryption artifacts are deterministic and machine-readable.
- `ENC: unlock ok` and explicit bad-key denial markers are stable.
- Keyslot rotation and crash recovery remain auditable across seeded runs.

## PR-3: Storage Encryption Gate + Key Policy Sub-gate

### Objective

Make encrypted-storage semantics enforceable in local and CI lanes.

### Scope

- Add local gates:
  - `Makefile` target `test-storage-encryption-v1`
  - `Makefile` target `test-storage-key-policy-v1`
- Add CI steps:
  - `Storage encryption v1 gate`
  - `Storage key policy v1 gate`
- Add aggregate tests:
  - `tests/storage/test_storage_encryption_gate_v1.py`
  - `tests/security/test_storage_key_policy_gate_v1.py`

### Primary files

- `Makefile`
- `.github/workflows/ci.yml`
- `tests/storage/test_storage_encryption_gate_v1.py`
- `tests/security/test_storage_key_policy_gate_v1.py`
- `MILESTONES.md`
- `docs/STATUS.md`
- `README.md`

### Acceptance checks

- `make test-storage-encryption-v1`
- `make test-storage-key-policy-v1`

### Done criteria for PR-3

- Encryption and key-policy sub-gates are required in local and CI release
  lanes.
- M59 can be marked done only with release-gated unlock, recovery, and keyslot
  rotation evidence.

## Non-goals for M59 backlog

- multi-device RAID coordination owned by M60
- snapshot and integrity repair semantics owned by M61
- hidden auto-unlock or secret-management shortcuts outside declared policy
