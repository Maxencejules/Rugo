# Full-OS guide Part I.4 acceptance: swap / page eviction.
#
# At boot the kernel maps a user page, writes a pattern, EVICTS it to a disk swap
# slot (the physical frame is freed and the PTE marked swapped, present=0), then
# swaps it back in (a fresh frame is allocated and the page read back from disk)
# and confirms the page reads byte-exact through its VA -- proving the page
# survived the round-trip frame -> disk -> fresh frame. The swap-in path is also
# wired into the page-fault handler for live swap-on-demand.


def test_swap_roundtrip(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    find_in_order(out, [
        "SWAP: roundtrip ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "SWAP: fail" not in out
    assert "SWAP: skip" not in out
