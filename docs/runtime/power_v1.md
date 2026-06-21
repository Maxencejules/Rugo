# power / ACPI — contract v1

Status: boot-verified via `make test-power-v1`
Source: `kernel_rs/src/lib.rs` (`sys_power`), `services/go/start.asm` +
`services/go/syscalls.go` (`sysPower`), `services/go/shell_session.go`
(`poweroff` builtin).
Proof: `tests/runtime/test_power_v1.py`.

Full-OS implementation guide Part IV.9 (power/ACPI), the shutdown/reboot
slice.

## ABI

`sys_power` — ABI v3.2 id **58**: `rdi` = op. **uid 0 only** (-1 otherwise).

| op | action |
|----|--------|
| 0 | shutdown |
| 1 | reboot |

Neither op returns on success.

## Mechanism

- **Shutdown** drains the UART (so the marker reaches the host), writes the
  ACPI S5 sleep command (`0x2000` = SLP_EN) to the PM1a control port — q35
  `0x604` (PMBASE 0x600) and i440fx `0xB004` — then falls back to the
  isa-debug-exit port. QEMU stops either way.
- **Reboot** pulses the 8042 controller reset line (`0xFE` → port `0x64`).

The `poweroff` shell builtin (the shell runs as uid 0) calls `sys_power(0)`.

## v1 boundary / carry-forward

- ACPI is done by hard-coded PM port writes, not by parsing the FADT/RSDP
  (the guide's userspace `acpi.go` discovery service is carry-forward; it
  would supply the real PM base and `_S5` SLP_TYP).
- `reboot` is implemented but not in the automated gate (a reboot restarts
  the VM, which the serial-capture harness does not script a second boot
  for); shutdown is the gated path.
- No suspend (S1/S3) or hibernate (S4).

## Acceptance

`make test-power-v1`: the `poweroff` builtin emits `POWER: shutdown`, the
machine stops, and the orderly service-teardown markers do **not** appear
(power-off is abrupt by design).
