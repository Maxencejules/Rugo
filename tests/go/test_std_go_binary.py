"""G2 prep scaffold: standard Go binary acceptance marker.

This test is intentionally scaffold-only for now. It becomes active once the
standard Go toolchain path produces `out/os-go-std.iso`.
"""


def test_std_go_binary(qemu_serial_go_std):
    """Standard-Go user binary prints GOSTD: ok via syscall bridge."""
    serial = qemu_serial_go_std.stdout
    assert "GOSTD: ok" in serial, (
        "Expected 'GOSTD: ok' in serial output for G2 acceptance.\n"
        f"Full output:\n{serial}"
    )
