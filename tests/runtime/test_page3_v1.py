# Exec-loader hardening: an app that spans a THIRD page (vaddr >= 0x1402000).
#
# Most apps fit in 1-2 pages; page3probe pads an initialized global into the 3rd
# page and reads it back, proving exec_load_app/as_copyout map and load every
# page of a multi-page image (a regression guard discovered while debugging a
# 3-page C binary -- the kernel path is correct; the PE->ELF toolchain is the
# fragile part for C apps).


def test_exec_three_page_app(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("probe page3probe\nshutdown\n").stdout

    find_in_order(out, [
        "EXEC: page3probe ok",
        "PAGE3: ok",
        "RUGO: halt ok",
    ])
    assert "PAGE3: FAIL" not in out
    assert "USERPF" not in out
