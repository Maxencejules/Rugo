# Full-OS guide Part III acceptance: PS/2 mouse bring-up.
#
# At boot the kernel brings up the i8042 auxiliary (mouse) port: it enables the
# aux device, resets the mouse, and reads its Basic Assurance Test result (0xAA)
# and device ID (0x00 for a standard PS/2 mouse), reporting
# "MOUSE: reset bat=0xAA id=0x00 ok". The keyboard poll already drains stray aux
# bytes (status bit 5), so an enabled mouse does not disturb the console (the
# shell still takes the "shutdown" keystrokes and exits cleanly).


def test_ps2_mouse_reset_and_id(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    find_in_order(out, [
        "MOUSE: reset bat=0x00000000000000AA id=0x0000000000000000 ok",
        # The keyboard-driven shell still works with the mouse enabled.
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "GOINIT: err" not in out
