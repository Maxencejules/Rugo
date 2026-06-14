# Full-OS guide Part I.3 acceptance: an application processor services REAL
# syscalls for a migrated ring-3 task, with a per-CPU `current`.
#
# Beyond the SMP capstone (a ring-3 task merely computing + reporting on an AP),
# this proves the AP runs the full multi-CPU scheduler primitives:
#   * it sets its per-CPU `current` task through its own GS base (gs:[16]) before
#     entering ring 3 and reads it back in the trap handler on the same core
#     (SMP: ap-current = the dispatched task id 0x5A);
#   * the migrated task issues TWO real `int 0x80` syscalls (sys_time_now) on the
#     AP -- taking the ring-3->ring-0 transition onto the AP's own per-CPU TSS
#     rsp0, running real kernel code, and returning to ring 3 -- whose monotonic
#     tick delta is exactly 1 (SMP: ap-syscall delta=0x...1), which is only
#     possible if the kernel actually serviced both syscalls on the AP.
#
# No page fault must occur (an AP running on its private address space must reach
# all kernel state the syscall path touches).

import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import test_smp_runtime_v1 as smp  # noqa: E402

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import conftest  # noqa: E402


def test_ap_services_real_syscalls_with_percpu_current(find_in_order):
    iso = os.path.join(conftest.REPO_ROOT, "out", "os-go.iso")
    if not os.path.isfile(iso):
        import pytest

        pytest.skip(f"ISO not built: {iso}")
    out = smp._boot_smp(iso, 2, input_text="shutdown\n", with_devices=True, timeout=40)
    find_in_order(out, [
        "SMP: aps online=0x0000000000000001",
        # The AP serviced two real int 0x80 syscalls; consecutive monotonic ticks
        # differ by exactly 1, so real kernel code ran for each on the AP.
        "SMP: ap-syscall delta=0x0000000000000001",
        # The AP set + read back its per-CPU `current` (the dispatched task id).
        "SMP: ap-current=0x000000000000005A",
        "SMP: ap user task ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    # The AP must not have faulted on its private address space.
    assert "USERPF" not in out
    assert " FAIL" not in out
