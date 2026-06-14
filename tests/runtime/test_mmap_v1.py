# Full-OS guide Part I.4 acceptance: mmap / brk / munmap (sys_vm_ctl id 50).
#
# `vmprobe` (default) queries+grows the program break, uses brk memory,
# mmaps an anonymous RW page, uses it, and munmaps it. `vmprobe ro` maps a
# PROT_READ page and writes it, which must fault and be killed (no CoW
# promotion of a genuinely read-only page).
#
# Anchors are single-write kernel / app markers, never echoed prompt lines.


def test_mmap_brk_munmap(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("probe vmprobe\nshutdown\n").stdout

    find_in_order(out, [
        "MM: brk 0x",
        "MM: mmap va=0x0000000001200000",
        "MM: munmap va=0x0000000001200000",
        "VMPROBE: ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "VMPROBE: FAIL" not in out
    # The probe ran in its own address space and it was reclaimed on exit.
    assert "ASRELEASE: tid=0x" in out
    assert "GOINIT: err" not in out


def test_mmap_prot_read_only_enforced(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("probe vmprobe ro\nshutdown\n").stdout

    # The read-only mapping is created and read, but the write faults and is
    # contained (USERPF) - prot is enforced, not silently promoted to CoW.
    find_in_order(out, [
        "VMPROBE: ro mapped",
        "USERPF: addr=0x0000000001220000",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    # The write must NOT have succeeded.
    assert "VMPROBE: ro WROTE" not in out
    assert "GOINIT: err" not in out


def test_mprotect_downgrade_enforced(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("probe vmprobe mp\nshutdown\n").stdout

    # A RW page is mmapped and written, then mprotect drops it to read-only;
    # the next write faults and is contained.
    find_in_order(out, [
        "MM: mprotect va=0x0000000001230000",
        "VMPROBE: mp protected",
        "USERPF: addr=0x0000000001230000",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "VMPROBE: mp WROTE" not in out
    assert "GOINIT: err" not in out
