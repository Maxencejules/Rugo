# Phase 10a acceptance: W^X on dynamic user memory. The nxprobe app
# copies a `ret` onto its demand-paged stack and calls it; with
# EFER.NXE on and NX set on data pages the fetch must fault (USERPF
# err bit 4 = instruction fetch) and the kernel must kill the probe
# while the system carries on to a clean shutdown.


def _find_in_order(serial: str, markers: list[str]) -> None:
    pos = -1
    for marker in markers:
        pos = serial.find(marker, pos + 1)
        assert pos != -1, f"Missing '{marker}' in serial output.\nFull output:\n{serial}"


def test_stack_execution_is_blocked(qemu_go_c4_runtime):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("nxprobe\nshutdown\n").stdout

    _find_in_order(out, [
        "MM: nx on",
        "EXEC: nxprobe ok",
        "NXPROBE: jumping to stack",
        "USERPF: addr=0x00000000019",
        "err=0x0000000000000015",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "NXPROBE: executed from stack" not in out
    assert "GOINIT: err" not in out
