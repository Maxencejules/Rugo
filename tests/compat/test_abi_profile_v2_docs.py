"""M17 PR-1: ABI/profile v2 docs and baseline wiring checks."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_m17_pr1_artifacts_exist():
    required = [
        "docs/M17_EXECUTION_BACKLOG.md",
        "docs/abi/syscall_v2.md",
        "docs/abi/compat_profile_v2.md",
        "docs/abi/elf_loader_contract_v2.md",
        "docs/runtime/syscall_coverage_matrix_v2.md",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M17 artifact: {rel}"


def test_v2_docs_declare_contract_ids_and_gate_hooks():
    syscall_doc = _read("docs/abi/syscall_v2.md")
    profile_doc = _read("docs/abi/compat_profile_v2.md")
    loader_doc = _read("docs/abi/elf_loader_contract_v2.md")
    matrix_doc = _read("docs/runtime/syscall_coverage_matrix_v2.md")

    for token in [
        "Syscall ABI identifier: `rugo.syscall_abi.v2`.",
        "Freeze window: v2.x.",
        "No syscall ID renumbering is allowed in v2.x.",
        "| 18 | `sys_open` | required |",
        "| 27 | `sys_sec_profile_set` | required |",
    ]:
        assert token in syscall_doc

    for token in [
        "Compatibility profile identifier: `rugo.compat_profile.v2`.",
        "Tier A (`required`): signed static CLI payloads.",
        "Tier B (`required`): signed dynamic/runtime payloads.",
        "Signed artifact inputs are mandatory.",
        "Local gate: `make test-compat-v2`.",
    ]:
        assert token in profile_doc

    for token in [
        "Loader contract identifier: `rugo.elf_loader_contract.v2`.",
        "Dynamic ELF (`ET_DYN`) is accepted when interpreter and relocation constraints",
        "Only `R_X86_64_RELATIVE` relocations are supported.",
    ]:
        assert token in loader_doc

    assert "compatibility gate | `test-compat-v2`" in matrix_doc


def test_kernel_dispatch_remains_v2_compatible_for_required_ids():
    # Dispatch arms live in syscall.rs since the kernel was modularized;
    # loader/validation helpers remain in lib.rs.
    kernel_src = _read("kernel_rs/src/lib.rs") + _read("kernel_rs/src/syscall.rs")
    for token in [
        "fn elf_v1_validate_image",
        "18 => sys_open_v1(arg1, arg2, arg3)",
        "19 => sys_read_v1(arg1, arg2, arg3)",
        "20 => sys_write_v1(arg1, arg2, arg3)",
        "21 => sys_close_v1(arg1)",
        "22 => sys_wait_v1(arg1, arg2, arg3)",
        "23 => sys_poll_v1(arg1, arg2, arg3)",
        "24 => sys_fd_rights_get_v1(arg1)",
        "25 => sys_fd_rights_reduce_v1(arg1, arg2)",
        "26 => sys_fd_rights_transfer_v1(arg1, arg2)",
        "27 => sys_sec_profile_set_v1(arg1)",
    ]:
        assert token in kernel_src

