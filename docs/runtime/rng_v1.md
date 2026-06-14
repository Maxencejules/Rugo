# getrandom / CSPRNG — contract v1

Status: boot-verified via `make test-rng-v1`
Source: `kernel_rs/src/lib.rs` (`RNG_STATE`, `rng_next`, `sys_getrandom`),
`apps/coreutils/rngprobe.asm`.
Proof: `tests/runtime/test_rng_v1.py`.

Full-OS implementation guide Part IV.10 (security), item 1 (RNG). The first
slice of the security cascade; ASLR / sandbox / audit / secure-boot are
later slices.

## ABI

`sys_getrandom` — ABI v3.2 id **54**: `rdi` = buffer pointer, `rsi` = len
(≤ 4096). Fills the user buffer with random bytes and returns the count
written, or -1 on a bad pointer or oversize request.

## Entropy

A `xorshift64*` pool (`RNG_STATE`), seeded lazily on first use from:

- the CMOS wall clock (`cmos_unix_seconds`),
- the PIT tick counter (`R4_PREEMPT_TICKS`),
- a fixed mixing constant,

and advanced per draw with live tick entropy mixed in before each
`xorshift64*` step and a final multiply. The buffer is filled 8 bytes at a
time and `copyout_user`-ed in ≤256-byte chunks.

## v1 boundary / carry-forward

- **No RDRAND/RDSEED seeding.** v1 relies on clock + timing entropy; CPUID-
  gated hardware seeding is carry-forward (kept out for portability across
  QEMU CPU models). Output is therefore **not** cryptographic-grade yet.
- **No `/dev/urandom` node** wired to the pool yet (carry-forward).
- **No reseeding** from interrupt jitter / network handshakes yet.

## Acceptance

`make test-rng-v1`: `probe rngprobe` draws two 16-byte buffers and asserts
the output is not all zero (the pool produced bytes) and the two draws
differ (the pool advances). Emits `RNGPROBE: ok`.
