"""M46 PR-1: bare-metal I/O profile and lifecycle contract doc checks."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_m46_pr1_baremetal_io_artifacts_exist():
    required = [
        "docs/M46_EXECUTION_BACKLOG.md",
        "docs/hw/baremetal_io_profile_v1.md",
        "docs/hw/usb_input_removable_contract_v1.md",
        "docs/hw/driver_lifecycle_contract_v6.md",
        "docs/desktop/input_stack_contract_v1.md",
        "tests/hw/test_baremetal_io_profile_v1.py",
        "tests/hw/test_usb_input_removable_docs_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M46 PR-1 artifact: {rel}"


def test_baremetal_io_profile_v1_doc_declares_required_tokens():
    doc = _read("docs/hw/baremetal_io_profile_v1.md")
    for token in [
        "Milestone: M46 Bare-Metal I/O Baseline v1",
        "Profile identifier: `rugo.baremetal_io_profile.v1`",
        "Report schema: `rugo.baremetal_io_baseline.v1`",
        "Driver lifecycle contract ID: `rugo.driver_lifecycle_report.v6`",
        "USB/removable contract ID: `rugo.usb_input_removable_contract.v1`",
        "Desktop input contract ID: `rugo.input_stack_contract.v1`",
        "Recovery workflow ID: `rugo.recovery_workflow.v3`",
        "`e1000e`",
        "`rtl8169`",
        "`xhci`",
        "`usb-hid`",
        "`usb-storage`",
        "Local gate: `make test-baremetal-io-baseline-v1`.",
        "Local sub-gate: `make test-usb-input-removable-v1`.",
        "CI gate: `Bare-metal io baseline v1 gate`.",
        "CI sub-gate: `USB input removable v1 gate`.",
        "`intel_q470_e1000e_xhci`",
        "`amd_b550_rtl8169_xhci`",
    ]:
        assert token in doc


def test_driver_lifecycle_contract_v6_doc_declares_m46_tokens():
    doc = _read("docs/hw/driver_lifecycle_contract_v6.md")
    for token in [
        "`link_ready`",
        "`hid_ready`",
        "`focus_delivery_ready`",
        "`media_ready`",
        "`recovery_media_bootstrap`",
        "`e1000e`",
        "`rtl8169`",
        "`xhci`",
        "`usb-hid`",
        "`usb-storage`",
        "`NET: e1000e not found`",
        "`NET: rtl8169 not found`",
        "`USB: hid not found`",
        "`USBSTOR: not found`",
    ]:
        assert token in doc

