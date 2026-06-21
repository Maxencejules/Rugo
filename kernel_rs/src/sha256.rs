// SHA-256 (FIPS 180-4) + measured boot (full-os guide Part IV.10, secure boot).
// A from-scratch no_std hash, verified against known-answer vectors, used to
// implement a TPM-style measured boot: each measured component is folded into a
// PCR-like accumulator (pcr = SHA-256(pcr || measurement)). This is the integrity
// half of secure boot; signature verification of the measurement is carry-forward.

#![allow(dead_code)]

use crate::serial_write;

#[rustfmt::skip]
static K: [u32; 64] = [
    0x428a2f98,0x71374491,0xb5c0fbcf,0xe9b5dba5,0x3956c25b,0x59f111f1,0x923f82a4,0xab1c5ed5,
    0xd807aa98,0x12835b01,0x243185be,0x550c7dc3,0x72be5d74,0x80deb1fe,0x9bdc06a7,0xc19bf174,
    0xe49b69c1,0xefbe4786,0x0fc19dc6,0x240ca1cc,0x2de92c6f,0x4a7484aa,0x5cb0a9dc,0x76f988da,
    0x983e5152,0xa831c66d,0xb00327c8,0xbf597fc7,0xc6e00bf3,0xd5a79147,0x06ca6351,0x14292967,
    0x27b70a85,0x2e1b2138,0x4d2c6dfc,0x53380d13,0x650a7354,0x766a0abb,0x81c2c92e,0x92722c85,
    0xa2bfe8a1,0xa81a664b,0xc24b8b70,0xc76c51a3,0xd192e819,0xd6990624,0xf40e3585,0x106aa070,
    0x19a4c116,0x1e376c08,0x2748774c,0x34b0bcb5,0x391c0cb3,0x4ed8aa4a,0x5b9cca4f,0x682e6ff3,
    0x748f82ee,0x78a5636f,0x84c87814,0x8cc70208,0x90befffa,0xa4506ceb,0xbef9a3f7,0xc67178f2,
];

fn process_block(h: &mut [u32; 8], blk: &[u8]) {
    let mut w = [0u32; 64];
    let mut t = 0;
    while t < 16 {
        w[t] = u32::from_be_bytes([blk[t * 4], blk[t * 4 + 1], blk[t * 4 + 2], blk[t * 4 + 3]]);
        t += 1;
    }
    while t < 64 {
        let s0 = w[t - 15].rotate_right(7) ^ w[t - 15].rotate_right(18) ^ (w[t - 15] >> 3);
        let s1 = w[t - 2].rotate_right(17) ^ w[t - 2].rotate_right(19) ^ (w[t - 2] >> 10);
        w[t] = w[t - 16]
            .wrapping_add(s0)
            .wrapping_add(w[t - 7])
            .wrapping_add(s1);
        t += 1;
    }
    let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
        (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
    let mut i = 0;
    while i < 64 {
        let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
        let ch = (e & f) ^ ((!e) & g);
        let t1 = hh
            .wrapping_add(s1)
            .wrapping_add(ch)
            .wrapping_add(K[i])
            .wrapping_add(w[i]);
        let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
        let maj = (a & b) ^ (a & c) ^ (b & c);
        let t2 = s0.wrapping_add(maj);
        hh = g;
        g = f;
        f = e;
        e = d.wrapping_add(t1);
        d = c;
        c = b;
        b = a;
        a = t1.wrapping_add(t2);
        i += 1;
    }
    h[0] = h[0].wrapping_add(a);
    h[1] = h[1].wrapping_add(b);
    h[2] = h[2].wrapping_add(c);
    h[3] = h[3].wrapping_add(d);
    h[4] = h[4].wrapping_add(e);
    h[5] = h[5].wrapping_add(f);
    h[6] = h[6].wrapping_add(g);
    h[7] = h[7].wrapping_add(hh);
}

/// SHA-256 digest of `data`.
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let bit_len = (data.len() as u64).wrapping_mul(8);
    let full = data.len() / 64;
    let mut i = 0;
    while i < full {
        process_block(&mut h, &data[i * 64..i * 64 + 64]);
        i += 1;
    }
    // Final padded block(s): remainder + 0x80 + zeros + 64-bit big-endian length.
    let rem = &data[full * 64..];
    let mut block = [0u8; 128];
    block[..rem.len()].copy_from_slice(rem);
    block[rem.len()] = 0x80;
    let total = if rem.len() + 1 + 8 <= 64 { 64 } else { 128 };
    block[total - 8..total].copy_from_slice(&bit_len.to_be_bytes());
    process_block(&mut h, &block[..64]);
    if total == 128 {
        process_block(&mut h, &block[64..128]);
    }
    let mut out = [0u8; 32];
    let mut j = 0;
    while j < 8 {
        out[j * 4..j * 4 + 4].copy_from_slice(&h[j].to_be_bytes());
        j += 1;
    }
    out
}

/// Extend a PCR-like accumulator with a measurement: pcr = SHA-256(pcr || data).
fn pcr_extend(pcr: &mut [u8; 32], data: &[u8]) {
    let mut buf = [0u8; 64];
    buf[..32].copy_from_slice(pcr);
    // The measurement folded in is the digest of the component (fixed 32 bytes),
    // exactly like a TPM PCR extend (which extends with a digest, not raw data).
    let m = sha256(data);
    buf[32..64].copy_from_slice(&m);
    *pcr = sha256(&buf);
}

/// SHA-256 + measured-boot self-test (full-os guide Part IV.10): verify the hash
/// against three FIPS 180-4 known-answer vectors (empty, "abc", and a 56-byte
/// message that forces a second padded block), then perform a measured boot —
/// extend a zeroed PCR with two kernel components and confirm the result is a
/// stable, non-zero measurement. Emits `SHA256: kat ok` and `MEASURE: pcr=0x...`.
pub fn sha256_selftest() -> u64 {
    let empty = sha256(b"");
    let expect_empty: [u8; 32] = [
        0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14, 0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f, 0xb9,
        0x24, 0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c, 0xa4, 0x95, 0x99, 0x1b, 0x78, 0x52,
        0xb8, 0x55,
    ];
    let abc = sha256(b"abc");
    let expect_abc: [u8; 32] = [
        0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22,
        0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00,
        0x15, 0xad,
    ];
    // 56 bytes -> after +1+8 the message needs a SECOND 64-byte block (exercises
    // the multi-block padding path). FIPS 180-4 Appendix B.2 vector.
    let long_msg = b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
    let long = sha256(long_msg);
    let expect_long: [u8; 32] = [
        0x24, 0x8d, 0x6a, 0x61, 0xd2, 0x06, 0x38, 0xb8, 0xe5, 0xc0, 0x26, 0x93, 0x0c, 0x3e, 0x60,
        0x39, 0xa3, 0x3c, 0xe4, 0x59, 0x64, 0xff, 0x21, 0x67, 0xf6, 0xec, 0xed, 0xd4, 0x19, 0xdb,
        0x06, 0xc1,
    ];
    if empty != expect_empty || abc != expect_abc || long != expect_long {
        serial_write(b"SHA256: kat fail\n");
        return 0;
    }
    serial_write(b"SHA256: kat ok\n");

    // Measured boot: extend a zeroed PCR with two real kernel components (the
    // SHA-256 round constants and the AES S-box), then record the measurement.
    let mut pcr = [0u8; 32];
    let k_bytes = {
        let mut b = [0u8; 256];
        let mut i = 0;
        while i < 64 {
            b[i * 4..i * 4 + 4].copy_from_slice(&K[i].to_be_bytes());
            i += 1;
        }
        b
    };
    pcr_extend(&mut pcr, &k_bytes);
    pcr_extend(&mut pcr, crate::aes::sbox_bytes());
    // A non-zero PCR proves the extend chain ran; the value is deterministic for
    // a given component set (a verifier compares it to a known-good measurement).
    let nonzero = pcr.iter().any(|&b| b != 0);
    serial_write(b"MEASURE: pcr=0x");
    let mut acc = 0u64;
    let mut i = 0;
    while i < 8 {
        acc = (acc << 8) | pcr[i] as u64;
        i += 1;
    }
    crate::serial_write_hex(acc);
    serial_write(b"\n");
    if nonzero {
        1
    } else {
        0
    }
}

/// HMAC-SHA256 (RFC 2104) over `msg` (bounded to 256 bytes) with `key`. The
/// package manager uses it as a symmetric package signature: a holder of the key
/// signs the payload, and the installer rejects any payload whose recomputed MAC
/// does not match. (Asymmetric signatures are carry-forward.)
pub fn hmac_sha256(key: &[u8], msg: &[u8]) -> [u8; 32] {
    let mut k = [0u8; 64];
    if key.len() > 64 {
        k[..32].copy_from_slice(&sha256(key));
    } else {
        k[..key.len()].copy_from_slice(key);
    }
    let mut ipad = [0u8; 64 + 256];
    let mut opad = [0u8; 64 + 32];
    let n = msg.len().min(256);
    let mut i = 0;
    while i < 64 {
        ipad[i] = k[i] ^ 0x36;
        opad[i] = k[i] ^ 0x5C;
        i += 1;
    }
    ipad[64..64 + n].copy_from_slice(&msg[..n]);
    let inner = sha256(&ipad[..64 + n]);
    opad[64..96].copy_from_slice(&inner);
    sha256(&opad)
}

/// Secure boot (full-os guide Part IV.10): measure a boot component into a PCR and
/// verify it against a GOLDEN digest baked into the kernel, refusing on mismatch.
/// Demonstrates the chain-of-trust gate: a trusted component measures to the
/// golden value (boot proceeds), and a one-byte tamper changes the measurement so
/// the verify rejects it (boot would refuse). Returns 1 on success.
pub fn secure_boot_selftest() -> u64 {
    // A fixed, trusted boot component (a boot manifest). Its golden measurement
    // (PCR = SHA-256(0^32 || component)) is computed offline and baked in.
    const COMPONENT: [u8; 29] = *b"RUGO secure-boot manifest v1\n";
    const GOLDEN: [u8; 32] = [
        0x26, 0xD9, 0x60, 0xBE, 0x62, 0x49, 0x95, 0x70, 0x05, 0xD7, 0x3C, 0x31, 0x48, 0x79, 0xC1,
        0x05, 0x7A, 0xFB, 0x0C, 0x29, 0x7D, 0x1F, 0x11, 0x3C, 0x3E, 0x68, 0x5C, 0xBB, 0x1D, 0x12,
        0x8E, 0xE2,
    ];
    // Measure the trusted component and verify it against the golden value.
    let mut pcr = [0u8; 32];
    pcr_extend(&mut pcr, &COMPONENT);
    let trusted_ok = pcr == GOLDEN;
    // Tamper one byte: the measurement must change, so the verify must REJECT it
    // (a chain-of-trust gate refuses to run a modified component).
    let mut bad = COMPONENT;
    bad[0] ^= 0x01;
    let mut pcr_bad = [0u8; 32];
    pcr_extend(&mut pcr_bad, &bad);
    let tamper_rejected = pcr_bad != GOLDEN;
    if trusted_ok && tamper_rejected {
        serial_write(b"SECURE_BOOT: golden ok\n");
        1
    } else {
        serial_write(b"SECURE_BOOT: FAIL\n");
        0
    }
}
