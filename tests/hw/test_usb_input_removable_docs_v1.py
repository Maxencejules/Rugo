"""M46 PR-1: USB input and removable-media contract doc checks."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_usb_input_removable_contract_v1_doc_declares_required_tokens():
    doc = _read("docs/hw/usb_input_removable_contract_v1.md")
    for token in [
        "Contract ID: `rugo.usb_input_removable_contract.v1`",
        "Parent profile ID: `rugo.baremetal_io_profile.v1`",
        "Desktop input contract ID: `rugo.input_stack_contract.v1`",
        "Recovery workflow ID: `rugo.recovery_workflow.v3`",
        "Baseline schema: `rugo.baremetal_io_baseline.v1`",
        "`xhci`",
        "`usb-hid`",
        "`usb-storage`",
        "`xhci_enumeration`",
        "`usb_keyboard_latency`",
        "`usb_pointer_latency`",
        "`usb_focus_delivery`",
        "`usb_repeat_consistency`",
        "`usb_storage_enumeration`",
        "`usb_storage_mount`",
        "`recovery_media_bootstrap`",
        "`desktop_input_checks`",
        "`install_recovery_checks`",
        "`USB: hid not found`",
        "`USBSTOR: not found`",
        "Local sub-gate: `make test-usb-input-removable-v1`",
        "CI sub-gate: `USB input removable v1 gate`",
    ]:
        assert token in doc


def test_input_stack_contract_v1_doc_declares_usb_bridge_tokens():
    doc = _read("docs/desktop/input_stack_contract_v1.md")
    for token in [
        "Input device bridge requirements",
        "`input_class`",
        "`input_device`",
        "`desktop_input_checks`",
        "`usb-hid`",
        "`rugo.usb_input_removable_contract.v1`",
    ]:
        assert token in doc


def test_m46_roadmap_anchor_declares_baremetal_io_gates():
    roadmap = _read("docs/M45_M47_HARDWARE_EXPANSION_ROADMAP.md")
    assert "test-baremetal-io-baseline-v1" in roadmap
    assert "test-usb-input-removable-v1" in roadmap

