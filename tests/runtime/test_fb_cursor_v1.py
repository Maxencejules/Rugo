# Full-OS guide Part III acceptance: mouse cursor compositing with save-under.
#
# A windowing system draws a software mouse cursor over whatever is on screen and
# must restore the pixels underneath when the cursor moves, or it leaves a trail.
# At boot the kernel runs a save-under self-test directly against the linear
# framebuffer (no QMP needed -- the kernel reads back its own pixels): it paints a
# known background patch, draws the 8x8 cursor over it (the cursor colour must
# appear), restores (the background must come back pixel-for-pixel), and leaves the
# screen exactly as found. "FBCURSOR: save-under ok" means draw + save-under +
# restore all round-tripped.


def test_fb_cursor_save_under(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    find_in_order(out, [
        "FBCURSOR: save-under ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "FBCURSOR: save-under FAIL" not in out
