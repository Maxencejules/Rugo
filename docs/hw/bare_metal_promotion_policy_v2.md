# Bare-Metal Promotion Policy v2

Date: 2026-03-10  
Milestone: M47 Hardware Claim Promotion Program v1  
Lane: Rugo (Rust kernel + Go user space)  
Status: active promotion policy

## Purpose

Define the Tier 2 promotion rules that convert the M46 bare-metal I/O baseline
from promotion-grade evidence into auditable support claims.

## Policy identity

- Policy identifier: `rugo.hw_baremetal_promotion_policy.v2`.
- Input report schema: `rugo.hw_baremetal_promotion_report.v2`.
- Support claim policy: `rugo.hw_support_claim_policy.v1`.
- Claim promotion report schema: `rugo.hw_claim_promotion_report.v1`.
- Bare-metal profile ID: `rugo.baremetal_io_profile.v1`.

## Promotion thresholds

- Minimum consecutive green runs: `12`.
- Minimum campaign pass rate: `0.98`.
- Maximum tolerated fatal lifecycle errors: `0`.
- Maximum tolerated deterministic negative-path violations: `0`.
- Desktop-linked classes: `xhci`, `usb-hid`.
- Recovery-linked class: `usb-storage`.

## Claimable Tier 2 classes

- `e1000e`
- `rtl8169`
- `xhci`
- `usb-hid`
- `usb-storage`

## Qualifying board floor

- `intel_q470_e1000e_xhci`
- `amd_b550_rtl8169_xhci`

At least one Tier 2 profile must satisfy the full bundle without manual
exception handling before claim promotion can pass.

## Required evidence bundle

- `out/baremetal-io-v1.json`
- `out/hw-promotion-v2.json`
- `out/desktop-smoke-v1.json`
- `out/recovery-drill-v3.json`
- `out/hw-claim-promotion-v1.json`

## Policy checks

- Current bare-metal I/O baseline remains green.
- Bare-metal promotion v2 remains green with complete artifacts.
- Tier 2 profile floor remains green.
- Desktop-linked classes keep desktop input checks green.
- Recovery-linked classes keep recovery workflow checks green.

## Gate binding

- Bare-metal promotion input command:
  - `python tools/collect_hw_promotion_evidence_v2.py --out out/hw-promotion-v2.json`
- Claim promotion command:
  - `python tools/run_hw_claim_promotion_v1.py --out out/hw-claim-promotion-v1.json`
- Local gate: `make test-hw-claim-promotion-v1`.
- Local sub-gate: `make test-hw-support-tier-audit-v1`.
- CI gate: `Hardware claim promotion v1 gate`.
- CI sub-gate: `Hardware support tier audit v1 gate`.

## Failure handling

- Any Tier 2 regression returns the affected class set to evidence-only status.
- Desktop or recovery bridge regressions block promotion even when raw driver
  checks pass.
- Unsupported or undocumented board exceptions do not produce support claims.
