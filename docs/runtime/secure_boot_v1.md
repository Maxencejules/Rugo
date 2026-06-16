# Secure boot (measured chain-of-trust gate) — contract v1

Status: boot-verified via `make test-secure-boot-v1`
Source: `kernel_rs/src/sha256.rs` (`secure_boot_selftest`, `pcr_extend`); boot call
after `sha256_selftest`.
Proof: `tests/runtime/test_secure_boot_v1.py`.

Full-OS guide Part IV.10 (security: secure boot). Builds on the measured-boot PCR
([`sha256_v1.md`] / `sha256_selftest`): a chain-of-trust gate that **verifies a
boot component's measurement against a golden value baked into the kernel and
refuses on mismatch**.

## Behaviour

`secure_boot_selftest` measures a trusted boot component (a fixed boot manifest)
into a PCR — `pcr = SHA-256(0^32 || SHA-256(component))`, the TPM-style extend
already used by measured boot — and compares it to a **golden digest** compiled
into the kernel:

- the **trusted** component measures to the golden value → boot proceeds;
- a **one-byte tamper** of the component changes the measurement, so the verify
  rejects it → a real gate would refuse to run the modified component.

Both must hold (`SECURE_BOOT: golden ok`); otherwise `SECURE_BOOT: FAIL`.

## Acceptance

`make test-secure-boot-v1`: the go lane boots and prints `SECURE_BOOT: golden ok`
(the trusted component verified against the golden PCR **and** the tamper was
rejected), reaching a clean shutdown with no `SECURE_BOOT: FAIL`.

## v1 boundary / carry-forward

- Demonstrates the **verify-against-golden + refuse-on-tamper** mechanism over a
  fixed component, reusing the SHA-256 PCR. Measuring the **actual** boot chain
  (the kernel image / install image bytes) against a per-release golden, an
  asymmetric **signature** (vs a baked golden digest), a hardware root of trust
  (TPM/UEFI Secure Boot keys), and *halting* the boot on failure (vs a self-test
  marker) are carry-forward.
