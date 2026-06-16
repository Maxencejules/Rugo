# Full-OS guide Part V.11 acceptance: dynamic loading (sys_dlctl dlopen/dlsym).
#
# `dlprobe` dlopen()s the kernel-embedded real ELF `.so` ("libdl") and dlsym()s +
# CALLS four exported symbols in ring 3, each return value gated on a different
# relocation having been applied by the loader:
#   getval()  == 42  -> R_X86_64_RELATIVE  (a relocated pointer word)
#   addtwo(40)== 42  -> plain symbol resolve + call
#   getgvar() == 99  -> R_X86_64_GLOB_DAT  (a SYMBOLIC GOT slot, resolved from
#                                           .dynsym, for a cross-object global)
#   callsum() == 42  -> R_X86_64_JUMP_SLOT (a SYMBOLIC .got.plt slot, eager-bound)
# So "DLPROBE: dlsym ok" means the loader applied RELATIVE + GLOB_DAT + JUMP_SLOT
# and the ring-3 code ran through the GOT/PLT correctly.
#
# Boundary: lazy PLT binding, DT_NEEDED chains, multiple objects + dlclose, and
# on-disk `.so` loading remain carry-forward; the C `.so` toolchain stays routed
# around via the asm-authored module (libdl.asm + libdl2.asm).


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
