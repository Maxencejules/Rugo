# Phase 9 acceptance: the libc-equivalent. `hello` is a real C program
# compiled with gcc against rlibc (libc/), running from the package
# store: printf formatting, malloc, and open/read on the /data tree all
# go through the POSIX-ish layer over the int 0x80 ABI.


def test_c_program_runs_against_rlibc(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot(
        "fsmk /data/etc\n"
        "fswrite /data/etc/motd from-c-with-love\n"
        "hello /data/etc/motd\n"
        "shutdown\n"
    ).stdout

    find_in_order(out, [
        "FSH: write ok",
        "EXEC: hello ok",
        "HELLOC: printf d=42 x=0xff s=works",
        "HELLOC: args=/data/etc/motd",
        "HELLOC: file[16]=from-c-with-love",
        "HELLOC: done",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "HELLOC: open err" not in out
    assert "HELLOC: read err" not in out
    assert "HELLOC: malloc err" not in out
    assert "APP: run err" not in out
    assert "USERPF" not in out
    assert "GOINIT: err" not in out
