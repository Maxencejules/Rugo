# Hardware Support Tier Audit v1

Date: 2026-03-10  
Milestone: M47 Hardware Claim Promotion Program v1  
Lane: Rugo (Rust kernel + Go user space)  
Status: active audit contract

## Purpose

Make hardware support claims machine-auditable by requiring every promoted
class to carry an explicit support tier, policy identifiers, and promotion
history.

## Audit identity

- Audit identifier: `rugo.hw_support_tier_audit.v1`.
- Report schema: `rugo.hw_support_tier_audit_report.v1`.
- Claim policy: `rugo.hw_support_claim_policy.v1`.
- Bare-metal promotion policy: `rugo.hw_baremetal_promotion_policy.v2`.
- Claim report schema: `rugo.hw_claim_promotion_report.v1`.

## Required claim fields

- `class_id`
- `support_tier`
- `claim_status`
- `policy_id`
- `promotion_policy_id`
- `promotion_history`
- `qualifying_profiles`

## Required tier mappings

- `virtio-blk-pci` modern -> `tier1`
- `virtio-net-pci` modern -> `tier1`
- `virtio-scsi-pci` -> `tier1`
- `virtio-gpu-pci` -> `tier1`
- `e1000e` -> `tier2`
- `rtl8169` -> `tier2`
- `xhci` -> `tier2`
- `usb-hid` -> `tier2`
- `usb-storage` -> `tier2`

## Unsupported registry

The audit must keep the unsupported registry explicit and non-claiming:

- `wifi`
- `bluetooth`
- `audio`
- `webcam`
- `discrete-gpu`
- `laptop-power-management`

## Audit failures

- undocumented support-tier drift
- missing promotion history
- promoted unsupported class
- missing policy identifiers
- mismatched tier summary or policy bindings

## Gate binding

- Audit command:
  - `python tools/run_hw_support_tier_audit_v1.py --out out/hw-support-tier-audit-v1.json`
- Local sub-gate: `make test-hw-support-tier-audit-v1`.
- Local gate: `make test-hw-claim-promotion-v1`.
- CI sub-gate: `Hardware support tier audit v1 gate`.
- CI gate: `Hardware claim promotion v1 gate`.

## Expected artifacts

- `out/hw-claim-promotion-v1.json`
- `out/hw-support-tier-audit-v1.json`

## Result policy

- Claimed classes are valid only when audit checks stay green.
- Unsupported classes remain explicit and machine-auditable.
- Any support-tier change requires a matching claim report and audit pass.
