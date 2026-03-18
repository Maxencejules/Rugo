# Security Hardening Profile v3

Date: 2026-03-09  
Milestone: M28 Security Hardening Program v3  
Status: active release gate

## Purpose

Define the hardened control profile for release lanes and map each control to
deterministic enforcement artifacts.

## Contract identifiers

- Hardening profile ID: `rugo.security_hardening_profile.v3`
- Threat model reference: `rugo.security_threat_model.v2`
- Attack suite report schema: `rugo.security_attack_suite_report.v3`
- Fuzz report schema: `rugo.security_fuzz_report.v2`

## Control families

### Control family A: Syscall and capability policy

- Control ID: `SEC-HARD-V3-A1` syscall allowlist policy must be enforced.
- Control ID: `SEC-HARD-V3-A2` handle rights may only reduce, never escalate.
- Control ID: `SEC-HARD-V3-A3` policy bypass attempts must be reported.

### Control family B: Response and advisory integrity

- Control ID: `SEC-HARD-V3-B1` vulnerability triage SLA must be machine-checked.
- Control ID: `SEC-HARD-V3-B2` embargo workflow stages must be auditable.
- Control ID: `SEC-HARD-V3-B3` advisory schema and CVE linkage must be linted.

### Control family C: Continuous hardening verification

- Control ID: `SEC-HARD-V3-C1` attack suite outcomes must be deterministic.
- Control ID: `SEC-HARD-V3-C2` fuzz campaign outcomes must be deterministic.
- Control ID: `SEC-HARD-V3-C3` hardening gate must fail when violations exceed
  threshold.

## Determinism and thresholds

- Attack suite seed: `20260309`.
- Fuzz campaign seed: `20260309`.
- Fuzz iterations per case: `1600`.
- Maximum allowed hardening failures: `0`.
- Maximum allowed hardening violations: `0`.

## Tooling and gate wiring

- Attack suite tool: `tools/run_security_attack_suite_v3.py`
- Fuzz tool: `tools/run_security_fuzz_v2.py`
- Runtime evidence artifact: `out/booted-runtime-v1.json`
- Runtime deny markers: `GOSH: recv deny`, `GOSH: reg deny`, `GOSH: spawn deny`
- Local gate: `make test-security-hardening-v3`
- Sub-gate: `make test-vuln-response-v1`
- CI steps:
  - `Security hardening v3 gate`
  - `Vulnerability response v1 gate`

## Required executable checks

- `tests/security/test_hardening_docs_v3.py`
- `tests/security/test_attack_suite_v3.py`
- `tests/security/test_fuzz_gate_v2.py`
- `tests/security/test_policy_enforcement_v3.py`
- `tests/security/test_security_hardening_gate_v3.py`
