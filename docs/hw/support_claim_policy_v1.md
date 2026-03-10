# Hardware Support Claim Policy v1

Date: 2026-03-10  
Milestone: M47 Hardware Claim Promotion Program v1  
Lane: Rugo (Rust kernel + Go user space)  
Status: active claim policy

## Purpose

Define explicit support-tier claim rules for the hardware classes introduced in
M45 and M46 so support status is auditable by policy, promotion history, and
tier assignment rather than inferred from test presence alone.

## Policy identity

- Policy identifier: `rugo.hw_support_claim_policy.v1`.
- Claim promotion report schema: `rugo.hw_claim_promotion_report.v1`.
- Support-tier audit report schema: `rugo.hw_support_tier_audit_report.v1`.
- Matrix evidence schema: `rugo.hw_matrix_evidence.v6`.
- Bare-metal I/O evidence schema: `rugo.baremetal_io_baseline.v1`.
- Bare-metal promotion input schema: `rugo.hw_baremetal_promotion_report.v2`.

## Support-tier model

| Support tier | Claim state | Claimable classes | Entry requirements |
|---|---|---|---|
| Tier 0 | reference-only support floor | `q35` reference machine profile | required precursor for Tier 1 virtual-platform claims |
| Tier 1 | claimable virtual-platform support | `virtio-blk-pci` modern, `virtio-net-pci` modern, `virtio-scsi-pci`, `virtio-gpu-pci` | M45 shadow criteria stay green and zero fatal lifecycle errors remain true |
| Tier 2 | claimable bare-metal qualification support | `e1000e`, `rtl8169`, `xhci`, `usb-hid`, `usb-storage` | M46 bare-metal baseline and bare-metal promotion policy v2 stay green |
| Tier 3 | evidence-only expansion candidates | future hardware breadth classes | deterministic evidence only, never claimable until promoted |
| Tier 4 | exploratory profiles | bring-up-only profiles | never claimable |

## Claim promotion rules

- Tier 1 claims require:
  - `rugo.hw.support_matrix.v6` evidence stays green,
  - both modern and transitional VirtIO profiles remain reproducible,
  - `virtio-gpu-pci` keeps `desktop_display_checks` green.
- Tier 2 claims require:
  - `rugo.baremetal_io_profile.v1` evidence stays green,
  - `rugo.hw_baremetal_promotion_policy.v2` stays green,
  - `xhci` and `usb-hid` keep desktop input checks green,
  - `usb-storage` keeps recovery checks green.
- Promotion remains explicit even when thresholds pass; the policy does not
  create support claims for classes outside the declared registry.

## Claimed class registry

- Tier 1 claimable classes:
  - `virtio-blk-pci` modern
  - `virtio-net-pci` modern
  - `virtio-scsi-pci`
  - `virtio-gpu-pci`
- Tier 2 claimable classes:
  - `e1000e`
  - `rtl8169`
  - `xhci`
  - `usb-hid`
  - `usb-storage`
- Unsupported registry remains explicit:
  - `wifi`
  - `bluetooth`
  - `audio`
  - `webcam`
  - `discrete-gpu`
  - `laptop-power-management`

## Required artifacts

- `out/hw-matrix-v6.json`
- `out/baremetal-io-v1.json`
- `out/hw-promotion-v2.json`
- `out/hw-claim-promotion-v1.json`
- `out/hw-support-tier-audit-v1.json`

## Gate binding

- Claim promotion command:
  - `python tools/run_hw_claim_promotion_v1.py --out out/hw-claim-promotion-v1.json`
- Support-tier audit command:
  - `python tools/run_hw_support_tier_audit_v1.py --out out/hw-support-tier-audit-v1.json`
- Local gate: `make test-hw-claim-promotion-v1`.
- Local sub-gate: `make test-hw-support-tier-audit-v1`.
- CI gate: `Hardware claim promotion v1 gate`.
- CI sub-gate: `Hardware support tier audit v1 gate`.

## Failure handling

- Any undocumented support-tier drift blocks claim promotion.
- Any missing promotion history blocks the affected claim.
- Unsupported classes remain non-claiming even if evidence artifacts exist.
- A passing test without a matching policy identifier is not a support claim.
