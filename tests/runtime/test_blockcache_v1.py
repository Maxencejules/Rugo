# Full-OS guide Part II.5 acceptance: block buffer cache.
#
# At boot the kernel runs a block-cache self-test against scratch sectors. It
# proves: write-back deferral (a cache_write does not immediately hit disk),
# flush-on-evict (the deferred write reaches disk when the LRU line is evicted),
# LRU eviction order, and read-hit caching (a repeat read is a hit, not a disk
# miss). The go-lane boot disk (this fixture attaches one) makes storage available.


def test_block_cache_selftest(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    find_in_order(out, [
        "CACHE: selftest ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "CACHE: selftest fail" not in out
    assert "CACHE: selftest skip" not in out
