"""Product alpha qualification doc checks."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_product_alpha_doc_artifacts_exist():
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


def test_product_alpha_doc_declares_required_tokens():
    doc = _read("docs/build/product_alpha_qualification_v1.md")
    for token in [
        "Qualification report schema: `rugo.product_alpha_qualification_report.v1`.",
        "Qualification policy ID: `rugo.product_alpha_qualification.v1`.",
        "Alpha candidate boot image: `out/os-go-desktop-native.iso`.",
        "Alpha candidate kernel image: `out/kernel-go-desktop-native.elf`.",
        "Panic validation image: `out/os-panic.iso`.",
        "Primary runtime capture: `out/product-alpha-runtime-capture-v1.json`.",
        "Primary report: `out/product-alpha-v1.json`.",
        "Runtime tool: `tools/run_product_alpha_qualification_v1.py`.",
        "Shared helper: `tools/product_alpha_common_v1.py`.",
        "Local gate: `make test-product-alpha-v1`.",
        "CI gate: `Product alpha v1 gate`.",
        "CI artifact: `product-alpha-v1-artifacts`.",
        "`bootable_default_image`",
        "`durable_nvme_storage`",
        "`wired_networking`",
        "`desktop_or_shell_boot`",
        "`install_path`",
        "`update_path`",
        "`recovery_path`",
        "`diagnostics_path`",
    ]:
        assert token in doc
