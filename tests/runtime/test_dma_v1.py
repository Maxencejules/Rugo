# Full-OS guide Part II.7 acceptance: driver DMA allocator.
#
# At boot the kernel reserves a physically-contiguous DMA pool from the PMM and
# runs a self-test: two allocations are page-aligned, non-overlapping, contiguous,
# and pool-contained, and after freeing the first a same-size re-alloc reuses its
# exact base (first-fit) — proving the bitmap accounts the region correctly. This
# is the allocation primitive a virtio/NVMe/e1000 driver uses for descriptor rings.


def test_dma_pool_and_selftest(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    find_in_order(out, [
        "DMA: pool base=0x",
        "DMA: selftest ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "DMA: pool none" not in out
    assert "DMA: selftest fail" not in out
