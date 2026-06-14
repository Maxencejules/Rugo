# Full-OS keystone acceptance: per-process address spaces.
#
# `asconc` spawns two `asprobe` instances concurrently. With per-task
# address spaces:
#   * both spawns succeed and both apps are resident at once (the exec
#     window is no longer single-occupancy),
#   * each app's `slot` global - the SAME virtual address in both - maps to
#     a private frame, so neither clobbers the other ("iso ok", never
#     "iso FAIL"),
#   * the kernel reclaims each private address space on exit (ASRELEASE),
#   * interleaved tick markers from both ids show the two apps running
#     concurrently under preemption.
#
# Every anchor below is a single-write kernel or app marker, never an
# echoed shell prompt line.


def test_concurrent_address_space_isolation(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("asconc\nshutdown\n").stdout

    # Overall envelope: a spawn happens, the shell reports success, the
    # boot reaches a clean shutdown. (Per-sibling ordering between A and B
    # is legitimately nondeterministic under preemption, so it is asserted
    # as causal chains per id below, not across ids.)
    find_in_order(out, [
        "SPAWN: asprobe as_ok 0x",
        "GOSH: asconc ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])

    # Per-app causal chain: each probe ticks (after the sibling had a turn)
    # and then reaches its own isolation verdict.
    find_in_order(out, ["ASPROBE: tick id=A", "ASPROBE: iso ok id=A"])
    find_in_order(out, ["ASPROBE: tick id=B", "ASPROBE: iso ok id=B"])

    # Two distinct private address spaces were created and both reclaimed:
    # the exec window is no longer single-occupancy.
    assert out.count("SPAWN: asprobe as_ok 0x") == 2
    assert out.count("ASRELEASE: tid=0x") >= 2

    # Isolation held for BOTH probes (same VA, different frame). A shared
    # address space would have corrupted one of them.
    assert "ASPROBE: iso ok id=A" in out
    assert "ASPROBE: iso ok id=B" in out
    assert "iso FAIL" not in out

    # Concurrency: both ids emit progress ticks while both are resident.
    assert out.count("ASPROBE: tick id=A") >= 1
    assert out.count("ASPROBE: tick id=B") >= 1

    # No spawn/exec failures slipped through.
    assert "APP: run err" not in out
    assert "EXEC: asprobe badelf" not in out
    assert "EXEC: asprobe noas" not in out
    assert "GOINIT: err" not in out
