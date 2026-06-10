"""M8 PR-2 fd-table compatibility checks."""

import sys
from pathlib import Path

sys.path.append(str(Path(__file__).resolve().parent))

from v1_model import FdTableModel


def test_fd_doc_and_syscall_contract(read_repo_file):
    doc = read_repo_file("docs/abi/process_thread_model_v1.md")
    syscall_doc = read_repo_file("docs/abi/syscall_v1.md")
    kernel_src = read_repo_file("kernel_rs/src/lib.rs") + read_repo_file(
        "kernel_rs/src/syscall.rs"
    )

    assert "## File descriptor table v1" in doc
    assert "| 18 | `sys_open` |" in syscall_doc
    assert "| 19 | `sys_read` |" in syscall_doc
    assert "| 20 | `sys_write` |" in syscall_doc
    assert "| 21 | `sys_close` |" in syscall_doc
    assert "| 23 | `sys_poll` |" in syscall_doc
    assert "18 => sys_open_v1(arg1, arg2, arg3)" in kernel_src
    assert "19 => sys_read_v1(arg1, arg2, arg3)" in kernel_src
    assert "20 => sys_write_v1(arg1, arg2, arg3)" in kernel_src
    assert "21 => sys_close_v1(arg1)" in kernel_src
    assert "23 => sys_poll_v1(arg1, arg2, arg3)" in kernel_src


def test_fd_open_read_write_close_sequence():
    model = FdTableModel()
    fd = model.open("/compat/hello.txt")
    assert fd >= 3

    n, chunk = model.read(fd, 64)
    assert n == len(chunk)
    assert chunk == b"compat v1 hello\n"

    cfd = model.open("/dev/console")
    assert cfd >= 3
    assert model.write(cfd, b"abc") == 3
    assert bytes(model.console_log) == b"abc"

    assert model.close(fd) == 0
    assert model.close(fd) == -1


def test_poll_ready_count_is_deterministic():
    model = FdTableModel()
    fd = model.open("/compat/hello.txt")
    cfd = model.open("/dev/console")

    ready, entries = model.poll([(fd, 0x0001), (cfd, 0x0004), (99, 0x0001)])
    assert ready == 3
    assert entries[0][2] == 0x0001  # POLLIN
    assert entries[1][2] == 0x0004  # POLLOUT
    assert entries[2][2] == 0x0008  # POLLERR
