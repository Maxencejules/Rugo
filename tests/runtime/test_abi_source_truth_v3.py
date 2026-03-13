"""M21 source-of-truth gate: validate docs, kernel, and shipped interface."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))

import extract_go_std_syscalls  # noqa: E402
import extract_kernel_syscalls  # noqa: E402


DOC_ROW_RE = re.compile(r"^\|\s*(\d+)\s*\|\s*`([^`]+)`\s*\|")


def _parse_v3_doc_table() -> dict[int, str]:
    """Return {id: name} from the frozen syscall surface in syscall_v3.md."""
    text = (ROOT / "docs" / "abi" / "syscall_v3.md").read_text(encoding="utf-8")
    table: dict[int, str] = {}
    for line in text.splitlines():
        match = DOC_ROW_RE.match(line.strip())
        if match:
            table[int(match.group(1))] = match.group(2)
    return table


def _kernel_table() -> dict[int, str]:
    source = ROOT / "kernel_rs" / "src" / "lib.rs"
    return extract_kernel_syscalls.extract_syscalls(source)


def _gostd_table() -> dict[int, str]:
    source = ROOT / "services" / "go_std" / "syscalls.asm"
    return extract_go_std_syscalls.extract_syscalls(source)


def test_frozen_v3_ids_present_in_kernel():
    """Every syscall ID in the frozen v3 doc must exist in the kernel dispatch."""
    doc = _parse_v3_doc_table()
    kernel = _kernel_table()
    missing = {syscall_id: name for syscall_id, name in doc.items() if syscall_id not in kernel}
    assert not missing, (
        f"Frozen v3 syscall IDs missing from kernel dispatch: {missing}\n"
        "The ABI doc promises these IDs but the kernel does not dispatch them."
    )


def test_frozen_v3_names_match_kernel():
    """Syscall names in the v3 doc must match the kernel's canonical names."""
    doc = _parse_v3_doc_table()
    kernel = _kernel_table()
    mismatches: list[str] = []
    for syscall_id, doc_name in sorted(doc.items()):
        kernel_name = kernel.get(syscall_id)
        if kernel_name is None:
            continue
        if kernel_name != doc_name:
            mismatches.append(
                f"  ID {syscall_id}: doc='{doc_name}' kernel='{kernel_name}'"
            )
    assert not mismatches, "ABI doc / kernel name mismatches:\n" + "\n".join(mismatches)


def test_kernel_has_no_undocumented_ids_in_frozen_range():
    """IDs 0-27 (frozen v3 range) in the kernel must all appear in the doc."""
    doc = _parse_v3_doc_table()
    kernel = _kernel_table()
    undocumented = {
        syscall_id: name
        for syscall_id, name in kernel.items()
        if syscall_id <= 27 and syscall_id not in doc
    }
    assert not undocumented, (
        f"Kernel dispatches undocumented IDs in frozen range 0-27: {undocumented}\n"
        "These must be added to docs/abi/syscall_v3.md or removed from dispatch."
    )


def test_extractor_finds_minimum_syscall_count():
    """Sanity: the kernel must implement at least the 28 frozen v3 syscalls."""
    kernel = _kernel_table()
    frozen_count = sum(1 for syscall_id in kernel if syscall_id <= 27)
    assert frozen_count >= 28, (
        f"Expected at least 28 syscalls in frozen range, found {frozen_count}."
    )


def test_kernel_extractor_report_schema():
    """The kernel JSON report has the expected schema."""
    source = ROOT / "kernel_rs" / "src" / "lib.rs"
    report = extract_kernel_syscalls.build_report(source)
    assert report["schema"] == "rugo.kernel_syscall_table.v1"
    assert isinstance(report["total_syscalls"], int)
    assert report["total_syscalls"] > 0
    assert isinstance(report["syscalls"], list)
    assert isinstance(report["syscalls_by_id"], dict)


def test_gostd_interface_matches_frozen_v3_doc_and_kernel():
    """Shipped stock-Go wrappers must agree with the frozen v3 ABI and kernel."""
    doc = _parse_v3_doc_table()
    kernel = _kernel_table()
    gostd = _gostd_table()
    mismatches: list[str] = []

    for syscall_id, interface_name in sorted(gostd.items()):
        doc_name = doc.get(syscall_id)
        kernel_name = kernel.get(syscall_id)
        if doc_name != interface_name:
            mismatches.append(
                f"doc mismatch id={syscall_id}: interface='{interface_name}' doc='{doc_name}'"
            )
        if kernel_name != interface_name:
            mismatches.append(
                f"kernel mismatch id={syscall_id}: interface='{interface_name}' kernel='{kernel_name}'"
            )

    assert not mismatches, "\n".join(mismatches)


def test_gostd_interface_has_expected_runtime_hook_subset():
    """The supported stock-Go lane exports the frozen runtime hook subset."""
    assert _gostd_table() == {
        0: "sys_debug_write",
        1: "sys_thread_spawn",
        2: "sys_thread_exit",
        3: "sys_yield",
        4: "sys_vm_map",
        5: "sys_vm_unmap",
        10: "sys_time_now",
    }


def test_gostd_interface_report_schema():
    """The stock-Go userspace interface report has the expected schema."""
    source = ROOT / "services" / "go_std" / "syscalls.asm"
    report = extract_go_std_syscalls.build_report(source)
    assert report["schema"] == "rugo.gostd_syscall_interface.v1"
    assert report["total_wrappers"] == 7
    assert isinstance(report["syscalls"], list)
    assert isinstance(report["syscalls_by_id"], dict)
    assert isinstance(report["wrappers_by_symbol"], dict)
