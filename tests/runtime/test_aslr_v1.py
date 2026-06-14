# Full-OS guide Part IV.10 acceptance: stack ASLR.
#
# Each spawn starts the stack a random page-aligned offset below the slot's
# top (drawn from the CSPRNG). Spawning the same program repeatedly reuses
# the same task slot, so the only variation in the reported rsp is the ASLR
# offset; several spawns must yield at least two distinct stack bases.

import re


def test_stack_aslr_varies_per_spawn(qemu_go_c4_runtime):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot(
        "echo a\necho b\necho c\necho d\necho e\necho f\nshutdown\n"
    ).stdout

    rsps = re.findall(r"SPAWN: echo as_ok 0x[0-9A-F]+ rsp=0x([0-9A-F]+)", out)
    assert len(rsps) >= 5, f"expected several echo spawns, got {rsps}"
    assert len(set(rsps)) >= 2, f"stack base not randomized across spawns: {rsps}"
    assert "EXEC: echo ok" in out
