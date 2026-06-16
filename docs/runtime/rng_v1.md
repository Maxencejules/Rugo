# getrandom / CSPRNG — contract v1

Status: boot-verified via `make test-rng-v1` + `make test-rng-hwseed-v1`
Source: `kernel_rs/src/lib.rs` (`RNG_STATE`, `RNG_HWSEED`, `rdrand64`,
`rng_next`, `sys_getrandom`, `rng_hwseed_selftest`),
`apps/coreutils/rngprobe.asm`.
Proof: `tests/runtime/test_rng_v1.py`, `tests/runtime/test_rng_hwseed_v1.py`.

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

- **RDRAND hardware seeding DONE** (CPUID.1:ECX[30]-gated). When the CPU
  advertises RDRAND, `rng_next`'s lazy init folds an `_rdrand64_step` value into
  the seed, **XOR-mixed** with the CMOS/PIT soft seed so a spoofed or absent
  RDRAND can only strengthen, never weaken, the entropy. On CPUs without it the
  portable soft seed is used. `RDSEED` (the higher-quality entropy source) and
  continuous **reseeding** from RDRAND/interrupt jitter remain carry-forward, as
  does a `/dev/urandom` node wired to the pool.

## Acceptance

`make test-rng-v1`: `probe rngprobe` draws two 16-byte buffers and asserts
the output is not all zero (the pool produced bytes) and the two draws
differ (the pool advances). Emits `RNGPROBE: ok`.

`make test-rng-hwseed-v1`: booted on `-cpu qemu64,+rdrand` the kernel reports
`RNG: hwseed rdrand ok` (RDRAND folded into the seed); on plain `-cpu qemu64` it
reports `RNG: hwseed soft (no rdrand)` (portable fallback). Both paths confirm
two distinct draws (never `RNG: hwseed FAIL`) and reach `RUGO: halt ok`.
