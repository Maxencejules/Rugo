//! Lamport one-time signature VERIFY (full-os guide Part IV.10, public-key
//! package signing).
//!
//! The earlier package-integrity scheme used a SYMMETRIC keyed hash
//! (HMAC-SHA256): the kernel held the key, so it could equally well FORGE a
//! signature -- not real public-key signing. This module verifies a genuine
//! asymmetric **Lamport** signature: the kernel embeds ONLY the public key (256
//! pairs of SHA-256 hashes) and a reference signature; it has no private key, so
//! it can verify but can never forge. The keypair + signature are produced
//! offline by `tools/lamport_keygen_v1.py` (deterministic seed, committed as
//! `lamport_pub.bin` / `lamport_sig.bin`). Reuses the in-kernel SHA-256.
//!
//! Lamport (SHA-256, n = 32, 256 message-hash bits):
//!   public key  pk[i][b] = SHA256(private preimage i,b)        (256x2x32 = 16384 B)
//!   signature   sig[i]   = preimage i selected by bit i of SHA256(msg)   (256x32)
//! Verify: for each bit i of SHA256(msg), SHA256(sig[i]) == pk[i][bit].

use crate::sha256::sha256;

const N: usize = 32;
const BITS: usize = 256;
const SIG_LEN: usize = BITS * N; // 8192
const PUB_LEN: usize = BITS * 2 * N; // 16384

/// The signed message (must match `MESSAGE` in `tools/lamport_keygen_v1.py`).
const LAMPORT_MSG: &[u8] = b"rugo-pkg-sign-v1";
static LAMPORT_PUB: &[u8; PUB_LEN] = include_bytes!("lamport_pub.bin");
static LAMPORT_SIG: &[u8; SIG_LEN] = include_bytes!("lamport_sig.bin");

// Scratch for the tamper case, so op 3 need not copy 8 KiB onto the kernel stack.
static mut SIG_SCRATCH: [u8; SIG_LEN] = [0u8; SIG_LEN];

/// Verify Lamport signature `sig` over `msg` against `pubkey`: for each bit i of
/// SHA-256(msg), SHA-256(sig[i]) must equal the matching public-key hash.
/// Constant in structure (always all 256 positions) -- a single mismatch fails.
pub(crate) fn lamport_verify(msg: &[u8], sig: &[u8; SIG_LEN], pubkey: &[u8; PUB_LEN]) -> bool {
    let mh = sha256(msg);
    let mut ok = true;
    let mut i = 0usize;
    while i < BITS {
        let bit = ((mh[i / 8] >> (7 - (i % 8))) & 1) as usize;
        let got = sha256(&sig[i * N..i * N + N]);
        let want = &pubkey[(i * 2 + bit) * N..(i * 2 + bit) * N + N];
        if &got[..] != want {
            ok = false;
        }
        i += 1;
    }
    ok
}

/// sys_sigverify (ABI v3.2 id 63): exercise the Lamport public-key verifier on
/// the embedded key + reference signature. op 1 = the genuine signature (-> 1);
/// op 2 = a tampered MESSAGE (-> 0); op 3 = a tampered SIGNATURE (-> 0). Proves
/// the verifier actually checks rather than rubber-stamping. 1 = accept, 0 = reject.
pub(crate) unsafe fn sys_sigverify(op: u64) -> u64 {
    match op {
        1 => lamport_verify(LAMPORT_MSG, LAMPORT_SIG, LAMPORT_PUB) as u64,
        2 => lamport_verify(b"rugo-pkg-sign-vX", LAMPORT_SIG, LAMPORT_PUB) as u64,
        3 => {
            SIG_SCRATCH.copy_from_slice(LAMPORT_SIG);
            SIG_SCRATCH[0] ^= 0x01; // flip one bit of one revealed preimage
            lamport_verify(LAMPORT_MSG, &SIG_SCRATCH, LAMPORT_PUB) as u64
        }
        _ => 0,
    }
}

/// Boot self-test (full-os guide Part IV.10): the embedded signature verifies,
/// and both a message tamper and a signature tamper are rejected.
pub(crate) unsafe fn sigverify_selftest() {
    let good = sys_sigverify(1) == 1;
    let msg_bad = sys_sigverify(2) == 0;
    let sig_bad = sys_sigverify(3) == 0;
    if good && msg_bad && sig_bad {
        crate::serial_write(b"PQSIG: lamport verify ok, forgery rejected\n");
    } else {
        crate::serial_write(b"PQSIG: lamport FAIL\n");
    }
}
