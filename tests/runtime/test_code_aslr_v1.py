# full-os guide Part IV.10: main-app code-base ASLR. base-shell is now an ET_DYN (PIE)
# app; the kernel loads it at a RANDOM, page-aligned base each spawn (exec_load_pie ->
# exec_aslr_base, drawn from the CSPRNG, avoiding the immediately-previous base). Running
# it TWICE in one boot must yield TWO DIFFERENT load bases, and the app must still run
# correctly at each (its RIP-relative code resolves at any base) -- proving the code, not
# just the stack, is randomized. Live evidence: the "ASLR: dyn base=" markers come from
# the kernel loader, the "BASESH: hello from disk" marker from the relocated app payload.

import re


def test_code_base_aslr_distinct_bases(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("pkg\nrun base-shell\nrun base-shell\nshutdown\n").stdout

    find_in_order(out, [
        "GOSH: pkg ok",
        "ASLR: dyn base=0x",
        "BASESH: hello from disk",
        "ASLR: dyn base=0x",
        "BASESH: hello from disk",
        "RUGO: halt ok",
    ])

    bases = [int(h, 16) for h in re.findall(r"ASLR: dyn base=0x([0-9A-Fa-f]+)", out)]
    assert len(bases) >= 2, f"expected >=2 PIE load bases, got {bases}"

    # The two consecutive spawns must land at DIFFERENT bases (code ASLR working).
    assert bases[0] != bases[1], f"ASLR load bases identical (no randomization): {[hex(b) for b in bases]}"

    # Each base must be page-aligned and inside the exec ASLR window.
    for b in bases[:2]:
        assert b % 0x1000 == 0, f"PIE base not page-aligned: {b:#x}"
        assert 0x0140_0000 <= b < 0x0170_0000, f"PIE base out of ASLR window: {b:#x}"

    # The app ran correctly at each random base (RIP-relative code resolved its rodata).
    assert out.count("BASESH: hello from disk") >= 2
    assert "BASESH: hello from disk" in out
    assert "EXEC: base-shell badhash" not in out
    assert "GOINIT: err" not in out
