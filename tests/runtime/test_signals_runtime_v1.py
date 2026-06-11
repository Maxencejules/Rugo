# Phase 10b acceptance: signals. The sigprobe app registers a handler
# and sends itself signal 15: the handler must run with the signal
# number, and sigreturn must resume the interrupted code path. In "die"
# mode no handler is registered and the kernel's default action must
# terminate the task - while the shell carries on.


def test_signal_delivery_and_default_kill(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("sigprobe\nsigprobe die\nshutdown\n").stdout

    find_in_order(out, [
        "EXEC: sigprobe ok",
        "SIGPROBE: handler sig=15",
        "SIGPROBE: resumed after handler",
        # second run: no handler -> default action kills the task
        "EXEC: sigprobe ok",
        "SIG: kill tid=",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "SIGPROBE: bad path" not in out
    assert out.count("SIGPROBE: handler sig=15") == 1
    assert out.count("SIG: kill tid=") == 1
    assert "GOINIT: err" not in out
