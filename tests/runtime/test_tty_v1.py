# Full-OS guide Part V.11 acceptance: TTY line discipline (canonical mode).
#
# At boot the kernel runs a line-discipline self-test: it feeds "ab\x08c\n" and
# confirms the cooked line is "ac\n" (the backspace erased the 'b') and the echo
# stream is "ab\b \bc\n" (each printable char echoed; the backspace rubbed out
# with the standard "\b \b"). This is the cooking layer a TTY/pty puts between the
# wire and an application's line-oriented read.


def test_tty_line_discipline(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    find_in_order(out, [
        "TTY: line discipline ok",
        # Control characters: Ctrl-U kills the line ("abc\x15x\n" -> "x\n"), Ctrl-W
        # erases the trailing word ("foo bar\x17\n" -> "foo \n"), Ctrl-C raises the
        # interrupt flag + flushes the line, Ctrl-D on an empty line signals EOF.
        "TTY: ctrl-chars ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "TTY: line discipline fail" not in out
    assert "TTY: ctrl-chars fail" not in out
