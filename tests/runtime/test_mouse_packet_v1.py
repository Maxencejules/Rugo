# Full-OS guide Part III acceptance: PS/2 mouse movement-packet parsing.
#
# Beyond the mouse reset/identify (mouse_v1), the kernel decodes standard 3-byte
# PS/2 movement packets into signed (dx, dy) + a button bitmap and accumulates a
# cursor. The boot self-test feeds synthetic packets: +5,+3 with the left button,
# then -2,-1 (negative via the sign bits) with no buttons, and confirms the cursor
# accumulated to (3, 2); it also rejects an out-of-sync packet (sync bit clear).


def test_mouse_movement_packets(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    find_in_order(out, [
        "MOUSE: reset bat=0x00000000000000AA id=0x0000000000000000 ok",
        "MOUSE: packet ok x=0x0000000000000003 y=0x0000000000000002",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "MOUSE: packet fail" not in out
