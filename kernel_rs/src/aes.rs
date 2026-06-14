// AES-128 block cipher (full-os guide Part IV.10, crypto): a real, standard
// cipher to back at-rest disk encryption, replacing the demo xorshift keystream.
// Encrypt-only core (the disk path uses AES in CTR mode, where decryption is the
// same XOR-with-keystream as encryption). Verified against the FIPS-197 known
// answer test. No timing-side-channel hardening (table-based S-box) — a v1 that
// is correct per the standard, not constant-time.

#![allow(dead_code)]

use crate::serial_write;

#[rustfmt::skip]
static SBOX: [u8; 256] = [
    0x63,0x7c,0x77,0x7b,0xf2,0x6b,0x6f,0xc5,0x30,0x01,0x67,0x2b,0xfe,0xd7,0xab,0x76,
    0xca,0x82,0xc9,0x7d,0xfa,0x59,0x47,0xf0,0xad,0xd4,0xa2,0xaf,0x9c,0xa4,0x72,0xc0,
    0xb7,0xfd,0x93,0x26,0x36,0x3f,0xf7,0xcc,0x34,0xa5,0xe5,0xf1,0x71,0xd8,0x31,0x15,
    0x04,0xc7,0x23,0xc3,0x18,0x96,0x05,0x9a,0x07,0x12,0x80,0xe2,0xeb,0x27,0xb2,0x75,
    0x09,0x83,0x2c,0x1a,0x1b,0x6e,0x5a,0xa0,0x52,0x3b,0xd6,0xb3,0x29,0xe3,0x2f,0x84,
    0x53,0xd1,0x00,0xed,0x20,0xfc,0xb1,0x5b,0x6a,0xcb,0xbe,0x39,0x4a,0x4c,0x58,0xcf,
    0xd0,0xef,0xaa,0xfb,0x43,0x4d,0x33,0x85,0x45,0xf9,0x02,0x7f,0x50,0x3c,0x9f,0xa8,
    0x51,0xa3,0x40,0x8f,0x92,0x9d,0x38,0xf5,0xbc,0xb6,0xda,0x21,0x10,0xff,0xf3,0xd2,
    0xcd,0x0c,0x13,0xec,0x5f,0x97,0x44,0x17,0xc4,0xa7,0x7e,0x3d,0x64,0x5d,0x19,0x73,
    0x60,0x81,0x4f,0xdc,0x22,0x2a,0x90,0x88,0x46,0xee,0xb8,0x14,0xde,0x5e,0x0b,0xdb,
    0xe0,0x32,0x3a,0x0a,0x49,0x06,0x24,0x5c,0xc2,0xd3,0xac,0x62,0x91,0x95,0xe4,0x79,
    0xe7,0xc8,0x37,0x6d,0x8d,0xd5,0x4e,0xa9,0x6c,0x56,0xf4,0xea,0x65,0x7a,0xae,0x08,
    0xba,0x78,0x25,0x2e,0x1c,0xa6,0xb4,0xc6,0xe8,0xdd,0x74,0x1f,0x4b,0xbd,0x8b,0x8a,
    0x70,0x3e,0xb5,0x66,0x48,0x03,0xf6,0x0e,0x61,0x35,0x57,0xb9,0x86,0xc1,0x1d,0x9e,
    0xe1,0xf8,0x98,0x11,0x69,0xd9,0x8e,0x94,0x9b,0x1e,0x87,0xe9,0xce,0x55,0x28,0xdf,
    0x8c,0xa1,0x89,0x0d,0xbf,0xe6,0x42,0x68,0x41,0x99,0x2d,0x0f,0xb0,0x54,0xbb,0x16,
];

static RCON: [u8; 10] = [0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1b, 0x36];

/// Multiply by 2 in GF(2^8) (xtime).
#[inline]
fn xtime(b: u8) -> u8 {
    let hi = b & 0x80;
    let mut r = b << 1;
    if hi != 0 {
        r ^= 0x1b;
    }
    r
}

/// Multiply by 3 in GF(2^8): 2·b ^ b.
#[inline]
fn mul3(b: u8) -> u8 {
    xtime(b) ^ b
}

/// Expand a 128-bit key into 11 round keys (176 bytes).
fn key_expand(key: &[u8; 16]) -> [u8; 176] {
    let mut rk = [0u8; 176];
    rk[..16].copy_from_slice(key);
    let mut i = 16usize;
    let mut rcon_i = 0usize;
    while i < 176 {
        let mut t = [rk[i - 4], rk[i - 3], rk[i - 2], rk[i - 1]];
        if i % 16 == 0 {
            // RotWord + SubWord + Rcon.
            let tmp = t[0];
            t[0] = SBOX[t[1] as usize] ^ RCON[rcon_i];
            t[1] = SBOX[t[2] as usize];
            t[2] = SBOX[t[3] as usize];
            t[3] = SBOX[tmp as usize];
            rcon_i += 1;
        }
        rk[i] = rk[i - 16] ^ t[0];
        rk[i + 1] = rk[i - 15] ^ t[1];
        rk[i + 2] = rk[i - 14] ^ t[2];
        rk[i + 3] = rk[i - 13] ^ t[3];
        i += 4;
    }
    rk
}

#[inline]
fn add_round_key(state: &mut [u8; 16], rk: &[u8], round: usize) {
    let base = round * 16;
    let mut i = 0;
    while i < 16 {
        state[i] ^= rk[base + i];
        i += 1;
    }
}

fn sub_bytes(state: &mut [u8; 16]) {
    let mut i = 0;
    while i < 16 {
        state[i] = SBOX[state[i] as usize];
        i += 1;
    }
}

/// ShiftRows on a column-major state (byte index = row + 4*col): row r rotates
/// left by r.
fn shift_rows(state: &mut [u8; 16]) {
    let s = *state;
    let mut r = 1usize;
    while r < 4 {
        let mut c = 0usize;
        while c < 4 {
            state[r + 4 * c] = s[r + 4 * ((c + r) % 4)];
            c += 1;
        }
        r += 1;
    }
}

fn mix_columns(state: &mut [u8; 16]) {
    let mut c = 0usize;
    while c < 4 {
        let i = 4 * c;
        let a0 = state[i];
        let a1 = state[i + 1];
        let a2 = state[i + 2];
        let a3 = state[i + 3];
        state[i] = xtime(a0) ^ mul3(a1) ^ a2 ^ a3;
        state[i + 1] = a0 ^ xtime(a1) ^ mul3(a2) ^ a3;
        state[i + 2] = a0 ^ a1 ^ xtime(a2) ^ mul3(a3);
        state[i + 3] = mul3(a0) ^ a1 ^ a2 ^ xtime(a3);
        c += 1;
    }
}

/// Encrypt one 16-byte block under a 128-bit key (10 rounds, FIPS-197).
pub fn encrypt_block(key: &[u8; 16], input: &[u8; 16]) -> [u8; 16] {
    let rk = key_expand(key);
    let mut state = *input;
    add_round_key(&mut state, &rk, 0);
    let mut round = 1usize;
    while round < 10 {
        sub_bytes(&mut state);
        shift_rows(&mut state);
        mix_columns(&mut state);
        add_round_key(&mut state, &rk, round);
        round += 1;
    }
    sub_bytes(&mut state);
    shift_rows(&mut state);
    add_round_key(&mut state, &rk, 10);
    state
}

/// XOR `buf` with the AES-128-CTR keystream for sector `lba` (counter block =
/// lba in the low 8 bytes, 16-byte block index in the high 8 bytes). Same
/// operation encrypts and decrypts (stream cipher).
pub fn ctr_xor(key: &[u8; 16], lba: u64, buf: &mut [u8]) {
    let mut block_index = 0u64;
    let mut off = 0usize;
    while off < buf.len() {
        let mut ctr = [0u8; 16];
        ctr[0..8].copy_from_slice(&lba.to_le_bytes());
        ctr[8..16].copy_from_slice(&block_index.to_le_bytes());
        let ks = encrypt_block(key, &ctr);
        let n = core::cmp::min(16, buf.len() - off);
        let mut k = 0;
        while k < n {
            buf[off + k] ^= ks[k];
            k += 1;
        }
        off += 16;
        block_index = block_index.wrapping_add(1);
    }
}

/// Boot self-test: AES-128 encrypt of the FIPS-197 known-answer vector, plus a
/// CTR round-trip (encrypt then decrypt restores the plaintext). Emits
/// `AES: kat ok` / `AES: kat fail`.
pub fn aes_selftest() -> u64 {
    // FIPS-197 Appendix C.1: key 000102…0f, in 00112233…ff -> 69c4e0d8…0a.
    let key: [u8; 16] = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
        0x0f,
    ];
    let pt: [u8; 16] = [
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee,
        0xff,
    ];
    let expect: [u8; 16] = [
        0x69, 0xc4, 0xe0, 0xd8, 0x6a, 0x7b, 0x04, 0x30, 0xd8, 0xcd, 0xb7, 0x80, 0x70, 0xb4, 0xc5,
        0x5a,
    ];
    let ct = encrypt_block(&key, &pt);
    if ct != expect {
        serial_write(b"AES: kat fail\n");
        return 0;
    }
    // CTR round-trip across a multi-block, non-block-aligned buffer.
    let mut data = [0u8; 35];
    let mut i = 0;
    while i < data.len() {
        data[i] = (i as u8).wrapping_mul(7).wrapping_add(1);
        i += 1;
    }
    let orig = data;
    ctr_xor(&key, 0xDEAD_BEEF, &mut data);
    if data == orig {
        serial_write(b"AES: kat fail\n"); // ciphertext must differ from plaintext
        return 0;
    }
    ctr_xor(&key, 0xDEAD_BEEF, &mut data); // decrypt
    if data != orig {
        serial_write(b"AES: kat fail\n");
        return 0;
    }
    serial_write(b"AES: kat ok\n");
    1
}
