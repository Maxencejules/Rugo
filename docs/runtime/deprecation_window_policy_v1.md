# Runtime Deprecation Window Policy v1

Date: 2026-03-09  
Milestone: M21 ABI + API Stability Program v3  
Policy ID: `rugo.deprecation_window_policy.v1`

## Purpose

Define explicit deprecation windows so ABI/API evolution stays predictable
across multi-release compatibility commitments.

## Minimum deprecation window

Minimum tagged-release window: 3.

Minimum calendar window: 180 days.

Both conditions are required before a deprecated syscall/API can be removed in
the next major ABI line.

## Required metadata for each deprecation entry

Every deprecated symbol must record:

- symbol name;
- first deprecation release (for example `v3.2`);
- first deprecation date (UTC);
- earliest removal release (major-line boundary);
- replacement path;
- migration note link;
- owning maintainer.

## Removal eligibility rule

Earliest removal release must be at least the minimum tagged-release window
after the deprecation release, and at least 180 days after the deprecation date.

Removal within the same ABI line (`v3.x`) is forbidden.

## Exception process

Exceptions are allowed only for security-critical breakage and require:

- security issue reference;
- explicit approval record in release notes;
- temporary compatibility shim plan.

## Enforcement hooks

- `tools/check_syscall_compat_v3.py` validates deprecation window metadata.
- `tests/runtime/test_abi_window_v3.py` validates policy/doc alignment.
- `tests/runtime/test_abi_stability_gate_v3.py` validates release-gate wiring.

