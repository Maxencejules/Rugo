# Full-OS guide Part III acceptance: a kernel input event queue.
#
# The IRQ12 mouse path (and future keyboard path) enqueue decoded input events
# into a kernel ring that userspace drains via sys_ioctl op 5 (input_poll) -- the
# event plumbing a compositor / interactive app needs (vs the prior code that
# decoded mouse packets only to log them). input_event_selftest enqueues a
# synthetic mouse-move + key event and drains them back, verifying the ring and
# the 16-byte wire encoding (kind/data/x/y) round-trip exactly.


def test_input_event_queue(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    find_in_order(out, [
        "INPUT: event queue ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "INPUT: event queue FAIL" not in out
