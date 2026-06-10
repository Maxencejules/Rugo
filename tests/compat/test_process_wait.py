"""M8 PR-2 process/wait compatibility checks."""

import sys
from pathlib import Path

sys.path.append(str(Path(__file__).resolve().parent))

from v1_model import ProcessModel


def test_wait_doc_and_syscall_contract(read_repo_file):
    doc = read_repo_file("docs/abi/process_thread_model_v1.md")
    syscall_doc = read_repo_file("docs/abi/syscall_v1.md")
    kernel_src = read_repo_file("kernel_rs/src/lib.rs") + read_repo_file(
        "kernel_rs/src/syscall.rs"
    )

    assert "## Exit/wait semantics v1" in doc
    assert "`argv`/`envp` delivery" in doc
    assert "| 22 | `sys_wait` |" in syscall_doc
    assert "22 => sys_wait_v1(arg1, arg2, arg3)" in kernel_src
    assert "unsafe fn sys_wait_v1" in kernel_src


def test_waitpid_deterministic_sequence():
    model = ProcessModel()
    assert model.execve(["/init"], ["K=V"]) == 0
    assert model.waitpid(-1) == (-1, None)

    assert model.exit(13) == 0
    assert model.waitpid(1) == (1, 13)
    assert model.waitpid(1) == (-1, None)


def test_waitpid_rejects_unsupported_options():
    model = ProcessModel()
    assert model.execve(["/init"], []) == 0
    assert model.exit(0) == 0
    assert model.waitpid(-1, options=1) == (-1, None)
