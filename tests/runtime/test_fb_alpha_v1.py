# Full-OS guide Part III acceptance: framebuffer alpha blending (src-over).
#
# fb_blit_rect_blend composites a translucent ARGB color onto the existing pixels
# (out = src*a + dst*(255-a) per channel) instead of overwriting, so translucent
# surfaces can show what is behind them. fb_alpha_selftest paints an opaque blue
# background on a single saved+restored pixel, blends 50%-alpha red over it, and
# verifies the read-back equals the src-over mix (0x80007E) -- then restores the
# pixel so the on-screen console is left untouched.


def test_fb_alpha_blend(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    find_in_order(out, [
        "FBALPHA: blend ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "FBALPHA: blend FAIL" not in out
    assert "FBALPHA: blend skip" not in out  # the go lane always has a framebuffer
