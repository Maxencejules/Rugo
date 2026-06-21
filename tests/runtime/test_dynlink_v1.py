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
# It also dlopen()s the module TWICE and requires the two load bases to DIFFER --
# code-base ASLR (full-os Part IV.10): the .so loads at a randomized, non-overlapping
# slot each time and the relocations are applied relative to it, so the symbol calls
# still resolve + run at the new base. "DLPROBE: aslr+dlsym ok" therefore means the
# load base was randomized AND the loader applied RELATIVE + GLOB_DAT + JUMP_SLOT and
# the ring-3 code ran through the GOT/PLT correctly.
#
# Boundary: code-base ASLR here is for dlopen'd shared objects; the main ET_EXEC app
# code base stays fixed (would need PIE main apps, blocked by the PE->ELF C toolchain).
# Lazy PLT binding, DT_NEEDED chains, multiple objects + dlclose remain carry-forward.


def test_dlopen_dlsym_call_and_code_aslr(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("probe dlprobe\nshutdown\n").stdout

    find_in_order(out, [
        "DLPROBE: aslr+dlsym ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "DLPROBE: FAIL" not in out
    assert "USERPF" not in out  # the loaded code ran in ring3 without faulting


def test_dlopen_handles_reclaimed_on_task_exit(qemu_go_c4_runtime, find_in_order):
    # Resource-lifecycle: each dlprobe run dlopen()s "libdl" TWICE and exits
    # WITHOUT dlclose, leaking 2 of the 4 global DL_HANDLES slots. Unless
    # task-exit cleanup reclaims a dead task's handles, two dlprobe runs fill
    # the table and the third run's first dlopen fails (handle table full).
    # Running dlprobe 3 times (2 leaks each > DL_MAX_HANDLES=4) must succeed
    # every time iff exiting tasks release their dlopen handles.
    boot, _disk_path = qemu_go_c4_runtime

    runs = 3
    out = boot("probe dlprobe\n" * runs + "shutdown\n").stdout

    assert "DLPROBE: FAIL" not in out
    assert "USERPF" not in out
    assert out.count("DLPROBE: aslr+dlsym ok") == runs, (
        f"expected {runs} successful dlprobe runs, "
        f"got {out.count('DLPROBE: aslr+dlsym ok')} "
        f"(dlopen handles leaked on exit -> table exhausted)"
    )


def test_dlclose_unmaps(qemu_go_c4_runtime):
    # Full-OS guide Part V.11: sys_dlctl op 3 = dlclose(base) unmaps the pages of the
    # most-recent dlopen and releases their frames. The boot self-test dlopens the
    # embedded module, confirms its base page is mapped, dlcloses it, confirms the
    # page is now unmapped and a double-close is rejected, then confirms a fresh
    # dlopen re-maps (and cleans that up). Runs in the shared boot address space after
    # the user page tables are live.
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    assert "DLCLOSE: unmap ok" in out
    assert "DLCLOSE: unmap fail" not in out
    assert "USERPF" not in out
