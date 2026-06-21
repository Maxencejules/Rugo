# Full-OS guide Part I.4 acceptance: 2 MiB huge pages.
#
# At boot the kernel maps a 2 MiB huge page (a single PD entry with the page-size
# bit, backed by a 2 MiB-aligned contiguous physical region) and confirms: the PD
# entry actually carries the PS bit (it is one mapping, not 512 4 KiB pages), and
# a read/write at offset 0 and at the last 8 bytes of the 2 MiB both work through
# that single mapping.


def test_huge_page_2mib(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    find_in_order(out, [
        "HUGEPAGE: 2M ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "HUGEPAGE: 2M fail" not in out
