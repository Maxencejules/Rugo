"""M11 bootstrap gate: verify stock-Go toolchain and build path are real."""

from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


_BOOTSTRAP_REQUIRED_FILES = [
    "tools/build_go_std_spike.sh",
    "tools/gostd_stock_builder/main.go",
    "tools/runtime_toolchain_contract_v1.py",
    "tools/bootstrap_go_port_v1.sh",
    "tools/extract_kernel_syscalls.py",
    "tools/extract_go_std_syscalls.py",
    "docs/runtime/port_contract_v1.md",
    "docs/runtime/syscall_coverage_matrix_v1.md",
    "docs/runtime/abi_stability_policy_v1.md",
    "docs/runtime/toolchain_bootstrap_v1.md",
    "docs/runtime/maintainers_v1.md",
    "services/go_std/main.go",
    "services/go_std/go.mod",
    "services/go_std/linker.ld",
    "services/go_std/start.asm",
    "services/go_std/rt0.asm",
    "services/go_std/runtime_stubs.asm",
    "services/go_std/syscalls.asm",
]


def _target_line(makefile: str, target: str) -> str:
    for line in makefile.splitlines():
        if line.startswith(f"{target}:"):
            return line
    raise AssertionError(f"Missing make target: {target}")


def _rule_block(makefile: str, target: str) -> str:
    match = re.search(
        rf"^{re.escape(target)}:.*(?:\n\t.*)*",
        makefile,
        re.MULTILINE,
    )
    if not match:
        raise AssertionError(f"Missing make rule block for: {target}")
    return match.group(0)


def test_all_bootstrap_files_present():
    """Every file referenced by the bootstrap script must exist."""
    missing = [rel for rel in _BOOTSTRAP_REQUIRED_FILES if not (ROOT / rel).is_file()]
    assert not missing, (
        "Bootstrap build-path files missing:\n"
        + "\n".join(f"  - {rel}" for rel in missing)
    )


def test_stock_builder_is_valid_go():
    """The stock Go builder tool must be parseable Go source."""
    builder = ROOT / "tools" / "gostd_stock_builder" / "main.go"
    text = builder.read_text(encoding="utf-8")
    assert "package main" in text, "Builder must be a Go main package."
    assert "func main()" in text, "Builder must have a main function."


def test_go_std_service_uses_rugo_syscalls():
    """The stock-Go service must import or reference rugo syscall bridges."""
    main_go = ROOT / "services" / "go_std" / "main.go"
    text = main_go.read_text(encoding="utf-8")
    assert "syscall" in text.lower() or "//go:linkname" in text or "unsafe" in text, (
        "Stock-Go main.go must reference syscall bridges to be a real runtime binary."
    )


def test_bootstrap_script_is_executable():
    """The bootstrap checker must exist and contain the expected interface."""
    script = ROOT / "tools" / "bootstrap_go_port_v1.sh"
    text = script.read_text(encoding="utf-8")
    assert "--check" in text, "Bootstrap script must accept --check flag."
    assert "--rebuild" in text, "Bootstrap script must accept --rebuild flag."
    assert "runtime-bootstrap: ok" in text, (
        "Bootstrap script must print 'runtime-bootstrap: ok' on success."
    )


def test_toolchain_contract_script_has_repro_mode():
    """The toolchain contract tool must support --repro for reproducibility."""
    script = ROOT / "tools" / "runtime_toolchain_contract_v1.py"
    text = script.read_text(encoding="utf-8")
    assert "--repro" in text, (
        "Toolchain contract script must support --repro for reproducibility checks."
    )
    assert "--out" in text, (
        "Toolchain contract script must support --out for artifact output."
    )


def test_makefile_has_stock_go_targets():
    """Makefile must expose build, boot, and smoke targets for stock Go."""
    makefile = (ROOT / "Makefile").read_text(encoding="utf-8")
    for target in ("build-go-std", "image-go-std", "boot-std", "smoke-std"):
        assert target in makefile, (
            f"Makefile must contain target '{target}' for first-class stock-Go support."
        )


def test_makefile_build_go_std_uses_supported_bootstrap_path():
    """The supported stock-Go build path must use bootstrap and contract rules."""
    makefile = (ROOT / "Makefile").read_text(encoding="utf-8")
    assert "GO_STD_CONTRACT = $(OUT)/gostd-contract.env" in makefile
    assert (
        "RUNTIME_TOOLCHAIN_CONTRACT = $(OUT)/runtime-toolchain-contract.env"
        in makefile
    )

    build_line = _target_line(makefile, "build-go-std")
    for dep in ("$(GO_STD_BIN)", "$(GO_STD_CONTRACT)", "$(RUNTIME_TOOLCHAIN_CONTRACT)"):
        assert dep in build_line

    userspace_line = _target_line(makefile, "userspace-std")
    for dep in ("$(GO_STD_BIN)", "$(GO_STD_CONTRACT)", "$(RUNTIME_TOOLCHAIN_CONTRACT)"):
        assert dep in userspace_line

    go_std_rule = _rule_block(makefile, "$(GO_STD_BIN) $(GO_STD_CONTRACT)")
    assert "tools/bootstrap_go_port_v1.sh --rebuild" in go_std_rule

    contract_rule = _rule_block(makefile, "$(RUNTIME_TOOLCHAIN_CONTRACT)")
    assert (
        "$(PYTHON) tools/runtime_toolchain_contract_v1.py --out $(RUNTIME_TOOLCHAIN_CONTRACT)"
        in contract_rule
    )


def test_makefile_smoke_std_expects_boot_markers():
    """smoke-std must check for RUGO boot and GOSTD markers."""
    makefile = (ROOT / "Makefile").read_text(encoding="utf-8")
    assert "GOSTD: ok" in makefile, (
        "smoke-std must verify 'GOSTD: ok' marker from the booted stock-Go image."
    )
