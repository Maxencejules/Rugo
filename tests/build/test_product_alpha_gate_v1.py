"""Product alpha gate wiring checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_product_alpha_qualification_v1 as tool  # noqa: E402


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_product_alpha_gate_wiring_and_artifacts(tmp_path: Path):
    required = [
        "docs/RUGO_V1_PRODUCT.md",
        "docs/build/product_alpha_qualification_v1.md",
        "tools/product_alpha_common_v1.py",
        "tools/run_product_alpha_qualification_v1.py",
        "tests/build/test_product_alpha_docs_v1.py",
        "tests/build/test_product_alpha_qualification_v1.py",
        "tests/build/test_product_alpha_gate_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing product alpha artifact: {rel}"

    readme = _read("README.md")
    makefile = _read("Makefile")
    ci = _read(".github/workflows/ci.yml")
    product = _read("docs/RUGO_V1_PRODUCT.md")

    assert "make test-product-alpha-v1" in readme
    assert "make test-product-alpha-v1" in product

    for entry in [
        "build-go-desktop-native",
        "image-go-desktop-native",
        "test-product-alpha-v1: image-go-desktop-native image-panic",
        "tools/run_product_alpha_qualification_v1.py --image $(OUT)/os-go-desktop-native.iso --kernel $(OUT)/kernel-go-desktop-native.elf --panic-image $(OUT)/os-panic.iso --runtime-capture-out $(OUT)/product-alpha-runtime-capture-v1.json --artifact-dir $(OUT) --supporting-dir $(OUT)/product-alpha-supporting --emit-supporting-reports --out $(OUT)/product-alpha-v1.json",
        "tests/build/test_product_alpha_docs_v1.py",
        "tests/build/test_product_alpha_qualification_v1.py",
        "tests/build/test_product_alpha_gate_v1.py",
        "pytest-product-alpha-v1.xml",
    ]:
        assert entry in makefile

    assert "Product alpha v1 gate" in ci
    assert "make test-product-alpha-v1" in ci
    assert "product-alpha-v1-artifacts" in ci
    for artifact in [
        "out/pytest-product-alpha-v1.xml",
        "out/product-alpha-runtime-capture-v1.json",
        "out/product-alpha-v1.json",
        "out/product-alpha-x4-runtime-v1.json",
        "out/product-alpha-x3-runtime-v1.json",
        "out/product-alpha-graphical-installer-v1.json",
        "out/product-alpha-release-bundle-v1.json",
        "out/product-alpha-update-metadata-v1.json",
        "out/product-alpha-recovery-drill-v3.json",
        "out/product-alpha-crash-dump-symbolized-v1.json",
        "out/os-go-desktop-native.iso",
        "out/kernel-go-desktop-native.elf",
    ]:
        assert artifact in ci

    out = tmp_path / "product-alpha-v1.json"
    artifact_dir = tmp_path / "artifacts"
    rc = tool.main(
        [
            "--fixture",
            "--artifact-dir",
            str(artifact_dir),
            "--runtime-capture-out",
            str(artifact_dir / "product-alpha-runtime-capture-v1.json"),
            "--out",
            str(out),
        ]
    )
    assert rc == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.product_alpha_qualification_report.v1"
    assert data["gate_pass"] is True
