# Hardware Expansion Plan v6

Date: 2026-03-10  
Lane: Rugo (Rust kernel + Go user space)  
Status: proposed planning document  
Supersedes: nothing; `docs/hw/support_matrix_v5.md` remains the active contract

Milestone planning and execution detail lives in:

- `docs/M45_M47_HARDWARE_EXPANSION_ROADMAP.md`
- `docs/M45_EXECUTION_BACKLOG.md`
- `docs/M46_EXECUTION_BACKLOG.md`
- `docs/M47_EXECUTION_BACKLOG.md`

## Goal

Broaden hardware coverage without weakening claim discipline.

The objective for v6 is not "support more hardware" in the abstract. The
objective is to add a small set of high-value device classes that:

- improve VM portability beyond the current transitional VirtIO baseline,
- improve real bare-metal usefulness on common desktop/server hardware,
- connect desktop claims to actual display/input hardware classes,
- preserve deterministic probe/init/runtime/recovery evidence,
- remain promotable through the existing matrix and bare-metal policy flow.

## Planning rules

- v5 remains the only active support contract until v6 docs, tooling, and tests
  exist and pass.
- New classes must enter as evidence-only or promotion-candidate coverage first.
- No class becomes release-claimable solely because a driver exists.
- Preference goes to classes that unlock multiple workloads:
  boot, install, network bring-up, recovery, and desktop interaction.
- Desktop-facing classes must be tied to `docs/desktop/*` latency and
  reliability contracts rather than ad hoc visual checks.

## Recommended next target classes

| Priority | Device class | Why it should be next | Initial validation lane | Promotion target |
|---|---|---|---|---|
| P1 | `virtio-blk-pci` modern | Closes the gap between current transitional VirtIO storage and modern virtual hardware defaults. | QEMU Tier 0/Tier 1 evidence-only, then release-blocking parity | Tier 0/Tier 1 required |
| P1 | `virtio-net-pci` modern | Same rationale for networking; improves modern VM portability and avoids overfitting to legacy transitional mode. | QEMU Tier 0/Tier 1 evidence-only, then release-blocking parity | Tier 0/Tier 1 required |
| P1 | `virtio-scsi-pci` | Adds multi-disk and common virtualization storage coverage beyond `virtio-blk`. | QEMU evidence-only | Tier 1 required after parity |
| P2 | `e1000e` | More realistic bare-metal/VM NIC target than legacy `e1000`; useful for wider lab and workstation compatibility. | Bare-metal candidate + VM evidence lane | Tier 2 promotion candidate |
| P2 | `rtl8169` | More relevant real-world Realtek desktop NIC family than `rtl8139`; high practical value for commodity boards. | Bare-metal candidate | Tier 2 promotion candidate |
| P2 | `xhci` + `usb-hid` keyboard/mouse baseline | Connects desktop claims to real interactive input hardware rather than only model-level desktop smoke. | QEMU + bare-metal candidate | Tier 2 promotion candidate |
| P2 | `usb-storage` | Important for installer, rescue, removable-media, and recovery workflows. | QEMU + bare-metal candidate | Tier 2 promotion candidate |
| P3 | `virtio-gpu-pci` framebuffer baseline | Connects desktop and GUI qualification to an explicit display device class with repeatable VM coverage. | QEMU evidence-only first | Tier 1 required after desktop parity |

## What not to do in v6

Do not spend v6 on:

- Wi-Fi
- Bluetooth
- audio
- webcams
- discrete GPU acceleration
- laptop battery/power-management breadth
- broad suspend/resume parity beyond existing bounded checks

Those areas have high surface area and weak near-term payoff relative to the
boot/install/network/display/input classes above.

## Milestone mapping

## M45: Modern virtual-platform parity v1

Objective:

- make Rugo less dependent on legacy-emulation-friendly defaults,
- close obvious gaps in modern VM deployability,
- add a display class that matches desktop qualification work.

Target classes:

- `virtio-blk-pci` modern
- `virtio-net-pci` modern
- `virtio-scsi-pci`
- `virtio-gpu-pci` framebuffer baseline

Recommended deliverables:

- `docs/hw/support_matrix_v6.md`
- `docs/hw/driver_lifecycle_contract_v6.md`
- `docs/hw/virtio_platform_profile_v1.md`
- `tests/hw/test_hw_matrix_docs_v6.py`
- `tests/hw/test_virtio_platform_profile_v1.py`
- `tests/hw/test_virtio_modern_storage_v1.py`
- `tests/hw/test_virtio_modern_net_v1.py`
- `tests/hw/test_virtio_scsi_v1.py`
- `tests/hw/test_virtio_gpu_framebuffer_v1.py`
- `tests/hw/test_hw_gate_v6.py`
- `tools/run_hw_matrix_v6.py`

Execution backlog:

- `docs/M45_EXECUTION_BACKLOG.md`

Exit criteria:

- modern VirtIO classes achieve deterministic probe/init/runtime markers,
- no regression in transitional VirtIO lanes,
- `virtio-gpu-pci` is tied to desktop smoke evidence rather than a standalone
  "display detected" marker,
- Tier 0 and Tier 1 can be run with both transitional and modern class
  coverage.

## M46: Bare-metal I/O baseline v1

Objective:

- move from VM-centric breadth to useful low-risk bare-metal breadth,
- prioritize ubiquitous wired networking and removable-media paths,
- connect interactive desktop claims to common USB input hardware.

Target classes:

- `e1000e`
- `rtl8169`
- `xhci` + `usb-hid`
- `usb-storage`

Recommended deliverables:

- `docs/hw/baremetal_io_profile_v1.md`
- `docs/hw/usb_input_removable_contract_v1.md`
- `tests/hw/test_e1000e_baseline_v1.py`
- `tests/hw/test_rtl8169_baseline_v1.py`
- `tests/hw/test_xhci_usb_hid_v1.py`
- `tests/hw/test_usb_storage_v1.py`
- `tests/hw/test_baremetal_io_gate_v1.py`
- `tools/collect_hw_promotion_evidence_v2.py`

Execution backlog:

- `docs/M46_EXECUTION_BACKLOG.md`

Exit criteria:

- each class has deterministic probe-negative and probe-positive coverage,
- recovery and reset behavior is accounted for in lifecycle evidence,
- input classes are validated against `docs/desktop/input_stack_contract_v1.md`
  thresholds,
- removable media paths are validated against installer/recovery workflows,
- at least one Tier 2 board profile produces a complete green evidence bundle.

## M47: Hardware claim promotion program v1

Objective:

- convert selected classes from evidence-only to claimable support,
- keep unsupported classes explicit,
- ensure the new breadth does not dilute gate credibility.

Recommended promotion order:

1. `virtio-blk-pci` modern
2. `virtio-net-pci` modern
3. `virtio-scsi-pci`
4. `virtio-gpu-pci`
5. `e1000e`
6. `rtl8169`
7. `xhci` + `usb-hid`
8. `usb-storage`

Promotion requirements:

- inherit v5/v1 promotion rules as a floor,
- require `12` consecutive green runs for bare-metal promotion candidates,
- require `0` fatal lifecycle errors,
- require deterministic negative-path behavior to stay intact,
- require desktop-linked classes to pass their desktop latency/reliability
  contracts,
- require installer/recovery-linked classes to pass rollback/recovery checks.

Execution backlog:

- `docs/M47_EXECUTION_BACKLOG.md`

## v6 matrix shape

Recommended matrix shape for v6:

| Tier | Intended purpose | Required class coverage |
|---|---|---|
| Tier 0 | QEMU reference | transitional VirtIO + modern VirtIO + `virtio-scsi-pci` + `virtio-gpu-pci` |
| Tier 1 | QEMU compatibility | same classes under alternate machine profile with parity markers |
| Tier 2 | Bare-metal qualification boards | `ahci` or `nvme`, plus `e1000e` or `rtl8169`, plus `xhci`/`usb-hid`; `usb-storage` strongly preferred |
| Tier 3 | Bare-metal breadth candidates | any new class under evidence-only campaigns |
| Tier 4 | Exploratory profiles | bring-up notes only; never claimable |

## Evidence model additions recommended for v6

Add the following report dimensions to the v5 evidence shape:

- `boot_transport_class`
- `display_class`
- `input_class`
- `removable_media_class`
- `desktop_input_checks`
- `desktop_display_checks`
- `install_recovery_checks`

Reason:

v5 is strong on storage/network/firmware/SMP, but a broader general-purpose
hardware strategy needs the matrix to understand "can I see the screen, type,
boot external media, and recover the machine?" not just "did storage and NIC
come up?"

## Proposed success criteria for the M45-M47 phase

The first v6 milestone phase should be considered successful if it achieves all
of:

- modern VirtIO storage and network parity in Tier 0/Tier 1,
- `virtio-scsi-pci` and `virtio-gpu-pci` covered in QEMU with deterministic
  reports,
- one `e1000e` board profile and one `rtl8169` board profile promoted or close
  to promotion,
- `xhci` + `usb-hid` able to satisfy desktop input thresholds,
- `usb-storage` validated against installer and recovery workflows,
- no relaxation of unsupported-class policy.

## Non-goals

- declaring "broad PC compatibility"
- adding laptop-first peripheral breadth
- chasing every emulator-only device class
- weakening gate thresholds to accelerate promotions
- mixing experimental hardware bring-up with release claims

## Bottom line

Rugo should broaden hardware support in v6, but only along a curated path:

1. modern VirtIO parity,
2. common wired bare-metal NICs,
3. USB input/removable-media baseline,
4. explicit display-class coverage tied to desktop evidence.

That gives Rugo a materially better hardware story without collapsing into
unbounded "PC compatibility" claims that the current matrix discipline cannot
justify.
