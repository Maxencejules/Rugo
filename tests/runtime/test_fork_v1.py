# Full-OS guide Part I.2 acceptance: fork + copy-on-write.
#
# `forkprobe` writes a sentinel global, forks itself (sys_proc_ctl op 1),
# the child writes a different value to that same global, and the parent
# proves its OWN copy is untouched. That only holds if fork produced a
# copy-on-write duplicate: the same virtual address backed by a private
# frame once either side writes.
#
# Anchors are single-write kernel / app markers, never echoed prompt lines.


def test_fork_copy_on_write(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("forkprobe\nshutdown\n").stdout

    # The kernel reports the fork, the child writes its private copy, the
    # parent confirms isolation, and the child's address space is reclaimed.
    find_in_order(out, [
        "FORK: child tid=0x",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])

    # Both sides reached their success markers (order between them is
    # nondeterministic under preemption).
    assert "FORKPROBE: child ok wrote private" in out
    assert "FORKPROBE: parent ok cow-isolated" in out

    # CoW must hold: neither side saw the other's write.
    assert "FORKPROBE: parent FAIL clobbered" not in out
    assert "FORKPROBE: child FAIL" not in out

    # Exactly one fork happened, and the child's private address space was
    # reclaimed on exit (keystone ASRELEASE path).
    assert out.count("FORK: child tid=0x") == 1
    assert "ASRELEASE: tid=0x" in out

    # No fault/kill leaked through the CoW path.
    assert "USERPF:" not in out
    assert "GOINIT: err" not in out
