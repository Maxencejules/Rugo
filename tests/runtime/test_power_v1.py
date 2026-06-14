# Full-OS guide Part IV.9 acceptance: power / ACPI shutdown (sys_power 58).
#
# The `poweroff` shell builtin (uid 0) calls sys_power(0): the kernel emits
# the marker, drains the UART, writes the ACPI S5 command to the PM control
# port, and falls back to debug-exit. The machine stops; the test asserts
# the marker reached the host and the boot ended (no clean-teardown markers,
# because power-off is abrupt by design).


def test_poweroff_shutdown(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("poweroff\n").stdout

    assert "POWER: shutdown" in out
    # Power-off is abrupt: the orderly service teardown must NOT have run.
    assert "GOSVCM: phase shutdown" not in out
