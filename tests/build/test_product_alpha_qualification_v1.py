"""Fixture-level checks for the product alpha qualification gate."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_product_alpha_qualification_v1 as tool  # noqa: E402


def test_product_alpha_fixture_schema_and_gate_pass(tmp_path: Path):
    out = tmp_path / "product-alpha-v1.json"
    artifact_dir = tmp_path / "artifacts"
    supporting_dir = tmp_path / "supporting"
    capture_out = artifact_dir / "product-alpha-runtime-capture-v1.json"
    image = tmp_path / "os-go-desktop-native.iso"
    kernel = tmp_path / "kernel-go-desktop-native.elf"
    panic = tmp_path / "os-panic.iso"

    rc = tool.main(
        [
            "--seed",
            "20260319",
            "--fixture",
            "--image",
            str(image),
            "--kernel",
            str(kernel),
            "--panic-image",
            str(panic),
            "--artifact-dir",
            str(artifact_dir),
            "--supporting-dir",
            str(supporting_dir),
            "--emit-supporting-reports",
            "--runtime-capture-out",
            str(capture_out),
            "--out",
            str(out),
        ]
    )
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    capture = json.loads(capture_out.read_text(encoding="utf-8"))

    assert data["schema"] == "rugo.product_alpha_qualification_report.v1"
    assert data["policy_id"] == "rugo.product_alpha_qualification.v1"
    assert data["gate"] == "test-product-alpha-v1"
    assert data["gate_pass"] is True
    assert data["total_failures"] == 0
    assert data["declared_profile"]["profile_id"] == "qemu-q35-default-desktop"
    assert data["declared_profile"]["machine"] == "q35"
    assert data["declared_profile"]["cpu"] == "qemu64,+x2apic"
    assert data["declared_profile"]["storage"] == "nvme"
    assert data["release_image"]["image_path"] == image.as_posix()
    assert data["release_image"]["kernel_path"] == kernel.as_posix()
    assert data["release_image"]["panic_image_path"] == panic.as_posix()

    assert capture["schema"] == "rugo.booted_runtime_capture.v1"
    assert capture["capture_mode"] == "fixture"
    assert capture["image_path"] == image.as_posix()
    assert capture["kernel_path"] == kernel.as_posix()
    assert capture["machine"] == "q35"
    assert capture["cpu"] == "qemu64,+x2apic"
    assert "nvme" in capture["disk_device"]

    check_map = {row["check_id"]: row["pass"] for row in data["checks"]}
    assert check_map == {
        "bootable_default_image": True,
        "durable_nvme_storage": True,
        "wired_networking": True,
        "desktop_or_shell_boot": True,
        "install_path": True,
        "update_path": True,
        "recovery_path": True,
        "diagnostics_path": True,
    }

    assert data["source_reports"]["desktop_profile_runtime"]["gate_pass"] is True
    assert data["source_reports"]["platform_runtime"]["gate_pass"] is True
    assert data["source_reports"]["graphical_installer"]["gate_pass"] is True
    assert data["source_reports"]["upgrade_drill"]["gate_pass"] is True
    assert data["source_reports"]["recovery_drill"]["gate_pass"] is True
    assert data["source_reports"]["trace_bundle"]["gate_pass"] is True
    assert data["source_reports"]["diagnostic_snapshot"]["gate_pass"] is True
    assert data["source_reports"]["crash_dump_symbolized"]["gate_pass"] is True

    assert (artifact_dir / "product-alpha-x4-runtime-v1.json").is_file()
    assert (artifact_dir / "product-alpha-x3-runtime-v1.json").is_file()
    assert (artifact_dir / "product-alpha-release-bundle-v1.json").is_file()
    assert (artifact_dir / "product-alpha-update-metadata-v1.json").is_file()
    assert (artifact_dir / "product-alpha-recovery-drill-v3.json").is_file()
    assert (artifact_dir / "product-alpha-crash-dump-symbolized-v1.json").is_file()
    assert (supporting_dir / "graphical-installer-v1.json").is_file()
    assert (supporting_dir / "real-pkg-install-v2.json").is_file()


def test_product_alpha_fixture_failure_propagates(tmp_path: Path):
    out = tmp_path / "product-alpha-fail.json"
    artifact_dir = tmp_path / "artifacts"
    capture_out = artifact_dir / "product-alpha-runtime-capture-v1.json"

    rc = tool.main(
        [
            "--fixture",
            "--artifact-dir",
            str(artifact_dir),
            "--runtime-capture-out",
            str(capture_out),
            "--inject-failure",
            "durable_nvme_storage",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    assert "durable_nvme_storage" in data["failures"]
