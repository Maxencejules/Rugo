# Phase 4 acceptance: the shell executes a real external program loaded
# from the package store on disk (sys_spawn, ABI v3.x id 46). Live runtime
# evidence - the app's own marker can only come from its ELF payload.


def _find_in_order(serial: str, markers: list[str]) -> None:
    pos = -1
    for marker in markers:
        pos = serial.find(marker, pos + 1)
        assert pos != -1, f"Missing '{marker}' in serial output.\nFull output:\n{serial}"


def test_shell_executes_app_from_disk(qemu_go_c4_runtime):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("pkg\nrun base-shell\nrun base-shell\nshutdown\n").stdout

    _find_in_order(out, [
        "GOSH: pkg ok",
        "rugo> run base-shell",
        "EXEC: base-shell ok",
        "BASESH: hello from disk",
        "APP: base-shell ok",
        # Second run proves the single-occupancy window is released on exit.
        "EXEC: base-shell ok",
        "BASESH: hello from disk",
        "APP: base-shell ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert out.count("EXEC: base-shell ok") == 2
    assert out.count("BASESH: hello from disk") == 2
    assert "EXEC: base-shell badhash" not in out
    assert "EXEC: base-shell missing" not in out
    assert "APP: base-shell exec err" not in out
    assert "GOINIT: err" not in out
