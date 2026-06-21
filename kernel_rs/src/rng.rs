//! CSPRNG + `sys_getrandom` (full-os guide Part IV.10).
//!
//! Extracted from `lib.rs` (gap #9, maintainability). A xorshift64* pool seeded
//! once (lazily) from the CMOS wall clock, the PIT tick counter, a constant, AND
//! — when the CPU supports it — the RDRAND hardware RNG (CPUID-gated, to stay
//! portable). The hardware value is XOR-mixed into the timing/clock seed, so a
//! spoofed or unavailable RDRAND can never *weaken* the existing entropy.
//!
//! Entropy sources (`crate::cmos_unix_seconds`, `crate::R4_PREEMPT_TICKS`) and the
//! serial logger stay in `lib.rs`; this module consumes them via `crate::` paths.

use crate::memory::copyout_user;

static mut RNG_STATE: u64 = 0;
// 0 = not yet seeded; 1 = RDRAND contributed; 2 = soft seed only (no RDRAND).
static mut RNG_HWSEED: u8 = 0;

/// Read a 64-bit value from RDRAND. The caller MUST have confirmed RDRAND
/// support via CPUID first (calling this on a CPU without it is UB). Retries
/// the spec-recommended bounded number of times for a transient under-run.
#[target_feature(enable = "rdrand")]
unsafe fn rdrand_raw() -> Option<u64> {
    let mut v = 0u64;
    let mut tries = 0;
    while tries < 10 {
        if core::arch::x86_64::_rdrand64_step(&mut v) == 1 {
            return Some(v);
        }
        tries += 1;
    }
    None
}

/// RDRAND value if the CPU advertises it (CPUID.1:ECX[30]), else None.
unsafe fn rdrand64() -> Option<u64> {
    if core::arch::x86_64::__cpuid(1).ecx & (1 << 30) == 0 {
        return None;
    }
    rdrand_raw() // safe: CPUID just confirmed RDRAND support
}

pub(crate) unsafe fn rng_next() -> u64 {
    if RNG_STATE == 0 {
        let mut seed = crate::cmos_unix_seconds()
            ^ 0x9E37_79B9_7F4A_7C15
            ^ (crate::R4_PREEMPT_TICKS << 17 | crate::R4_PREEMPT_TICKS);
        match rdrand64() {
            Some(hw) => {
                seed ^= hw; // strengthen — XOR never weakens the soft seed
                RNG_HWSEED = 1;
            }
            None => RNG_HWSEED = 2,
        }
        RNG_STATE = if seed == 0 { 0x1234_5678_9ABC_DEF1 } else { seed };
    }
    // Mix in live tick entropy, then xorshift64*.
    RNG_STATE ^= crate::R4_PREEMPT_TICKS.wrapping_add(0xD1B5_4A32_D192_ED03);
    let mut x = RNG_STATE;
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    RNG_STATE = x;
    x.wrapping_mul(0x2545_F491_4F6C_DD1D)
}

/// sys_getrandom (ABI v3.2 id 54): fill the user buffer at `buf_ptr` with `len`
/// random bytes. Returns the count written, or -1 on a bad pointer or oversize
/// request.
pub(crate) unsafe fn sys_getrandom(buf_ptr: u64, len: u64) -> u64 {
    const ERR: u64 = 0xFFFF_FFFF_FFFF_FFFF;
    if len == 0 {
        return 0;
    }
    if len > 4096 {
        return ERR;
    }
    let n = len as usize;
    let mut tmp = [0u8; 256];
    let mut done = 0usize;
    while done < n {
        let chunk = core::cmp::min(tmp.len(), n - done);
        let mut i = 0usize;
        while i < chunk {
            let r = rng_next().to_le_bytes();
            let take = core::cmp::min(8, chunk - i);
            tmp[i..i + take].copy_from_slice(&r[..take]);
            i += take;
        }
        if copyout_user(buf_ptr + done as u64, &tmp[..chunk], chunk).is_err() {
            return ERR;
        }
        done += chunk;
    }
    len
}

/// RNG hardware-seeding self-test (full-os guide Part IV.10): force the CSPRNG to
/// seed, report whether RDRAND contributed (`RNG: hwseed rdrand ok` on a CPU that
/// advertises it, `RNG: hwseed soft (no rdrand)` otherwise — both are healthy;
/// the soft path is the portable fallback), and confirm the pool still produces
/// distinct output (two 16-byte draws differ).
pub(crate) unsafe fn rng_hwseed_selftest() -> u64 {
    let mut a = [0u8; 16];
    let mut b = [0u8; 16];
    let mut i = 0usize;
    while i < 16 {
        a[i..i + 8].copy_from_slice(&rng_next().to_le_bytes());
        i += 8;
    }
    i = 0;
    while i < 16 {
        b[i..i + 8].copy_from_slice(&rng_next().to_le_bytes());
        i += 8;
    }
    if RNG_HWSEED == 1 {
        crate::serial_write(b"RNG: hwseed rdrand ok\n");
    } else {
        crate::serial_write(b"RNG: hwseed soft (no rdrand)\n");
    }
    if a == b {
        crate::serial_write(b"RNG: hwseed FAIL\n");
        return 0;
    }
    1
}
