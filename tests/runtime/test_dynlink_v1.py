# Full-OS guide Part V.11 acceptance: dynamic loading (sys_dlctl dlopen/dlsym).
#
# `dlprobe` dlopen()s a module the kernel ships embedded ("dlmod"), dlsym()s an
# exported symbol ("addone"), and CALLS the resolved function with arg 41 — which
# must return 42. This proves the kernel loaded separately-authored,
# position-independent code into an executable user region, resolved a symbol
# from the loaded image's export table, and the app executed it (dlopen/dlsym/
# call semantics).
#
# Boundary: a real ELF .so linker (dynamic relocation / GOT / PLT) is blocked on
# the PE->ELF C toolchain (mingw refptr breaks C binaries past 2 pages); this v1
# uses a kernel-shipped PIC module to demonstrate the loading mechanism.


def test_dlopen_dlsym_call(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("probe dlprobe\nshutdown\n").stdout

    find_in_order(out, [
        "DLPROBE: dlsym ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "DLPROBE: FAIL" not in out
    assert "USERPF" not in out  # the loaded code ran in ring3 without faulting
