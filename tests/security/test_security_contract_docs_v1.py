"""M10 acceptance: security contract documentation and syscall bindings."""

from pathlib import Path


def _read_repo_file(relpath: str) -> str:
    root = Path(__file__).resolve().parents[2]
    return (root / relpath).read_text(encoding="utf-8")


def test_security_docs_and_syscall_contract():
    syscall_doc = _read_repo_file("docs/abi/syscall_v1.md")
    # Dispatch arms live in syscall.rs since the kernel was modularized.
    kernel_src = _read_repo_file("kernel_rs/src/lib.rs") + _read_repo_file(
        "kernel_rs/src/syscall.rs"
    )
    rights_doc = _read_repo_file("docs/security/rights_capability_model_v1.md")
    filter_doc = _read_repo_file("docs/security/syscall_filtering_v1.md")
    boot_doc = _read_repo_file("docs/security/secure_boot_policy_v1.md")
    ir_doc = _read_repo_file("docs/security/incident_response_v1.md")

    assert "| 24 | `sys_fd_rights_get` |" in syscall_doc
    assert "| 25 | `sys_fd_rights_reduce` |" in syscall_doc
    assert "| 26 | `sys_fd_rights_transfer` |" in syscall_doc
    assert "| 27 | `sys_sec_profile_set` |" in syscall_doc
    assert "24 => sys_fd_rights_get_v1(arg1)" in kernel_src
    assert "25 => sys_fd_rights_reduce_v1(arg1, arg2)" in kernel_src
    assert "26 => sys_fd_rights_transfer_v1(arg1, arg2)" in kernel_src
    assert "27 => sys_sec_profile_set_v1(arg1)" in kernel_src

    assert "Per-handle rights" in rights_doc
    assert "default go service lane" in rights_doc.lower()
    assert "tests/security/test_go_service_policy_rights_v1.py" in rights_doc
    assert "Restricted profile" in filter_doc
    assert "key rotation" in boot_doc.lower()
    assert "security advisory" in ir_doc.lower()
