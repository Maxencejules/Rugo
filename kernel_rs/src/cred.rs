//! Credential database and authenticated login (full-os guide Part IV.10).
//!
//! Extracted from `lib.rs` (gap #9, maintainability): a self-contained module
//! holding the password table, the iterated salted KDF, and account lockout,
//! depending only on `crate::sha256`. The login *dispatch* (uid assignment,
//! audit) stays in the syscall layer and calls [`login_verify`].

/// One account in the credential database. The password is stored as the
/// PBKDF2-HMAC-SHA256 digest of a per-user `salt` + cleartext, never the
/// cleartext itself.
struct PasswdEntry {
    name: [u8; 8],
    uid: u8,
    /// Per-user random salt, mixed in front of the password before hashing so
    /// identical passwords on different accounts produce different digests and
    /// a rainbow table keyed on the bare password cannot match.
    salt: [u8; 16],
    pw_hash: [u8; 32],
}

/// Iteration count for the credential KDF. A slow (iterated) hash makes an
/// offline guess cost `PBKDF2_ITERS` HMACs instead of one, throttling brute
/// force; the value is a balance against per-login boot cost.
const PBKDF2_ITERS: u32 = 4096;

/// The credential database: root (uid 0) and a regular user (uid 100). Each
/// stored hash is `PBKDF2-HMAC-SHA256(pw, salt, PBKDF2_ITERS)` (32-byte output)
/// with a distinct per-user salt — an iterated slow KDF, so an offline attacker
/// pays `PBKDF2_ITERS` HMACs per guess and precomputed/rainbow tables and
/// cross-account hash equality are both defeated. `login_verify` recomputes the
/// KDF and compares, so the cleartext is never stored. (Moving the db to a
/// root-owned `/etc/shadow` VFS file is the next hardening step.)
static PASSWD: [PasswdEntry; 2] = [
    PasswdEntry {
        name: *b"root\0\0\0\0",
        uid: 0,
        salt: [
            0x72, 0x75, 0x67, 0x6F, 0x2D, 0x73, 0x61, 0x6C, 0x74, 0x2D, 0x72, 0x6F, 0x6F, 0x74,
            0x21, 0x21,
        ],
        pw_hash: [
            0xA1, 0x9A, 0xB6, 0x9F, 0x39, 0xA9, 0x81, 0xA6, 0x9E, 0xC7, 0x18, 0xC1, 0xE2, 0xCF,
            0x65, 0xD0, 0x07, 0x68, 0x4C, 0x16, 0xF6, 0xE2, 0x53, 0x57, 0xC4, 0x31, 0xB0, 0x82,
            0x99, 0x57, 0xCE, 0x3A,
        ],
    },
    PasswdEntry {
        name: *b"user\0\0\0\0",
        uid: 100,
        salt: [
            0x72, 0x75, 0x67, 0x6F, 0x2D, 0x73, 0x61, 0x6C, 0x74, 0x2D, 0x75, 0x73, 0x65, 0x72,
            0x21, 0x21,
        ],
        pw_hash: [
            0xCB, 0x29, 0x36, 0x4E, 0x8C, 0x92, 0xD9, 0x0C, 0x1F, 0x71, 0x31, 0xC2, 0xB4, 0x0A,
            0xFA, 0x05, 0x04, 0xB1, 0x93, 0x84, 0x8E, 0xF8, 0x8B, 0xDB, 0xB6, 0x8E, 0xCA, 0xB7,
            0xBB, 0xB8, 0xFF, 0xA4,
        ],
    },
];

/// PBKDF2-HMAC-SHA256 with a 32-byte derived key (one output block, block index
/// 1), per RFC 2898. `pw` is the HMAC key; `salt` is the per-user salt. The
/// derived key is `U1 ^ U2 ^ ... ^ Uc` where `U1 = HMAC(pw, salt || INT32BE(1))`
/// and `Ui = HMAC(pw, U(i-1))`.
fn pbkdf2_sha256_dk32(pw: &[u8], salt: &[u8; 16]) -> [u8; 32] {
    let mut msg = [0u8; 20];
    msg[..16].copy_from_slice(salt);
    msg[16..20].copy_from_slice(&1u32.to_be_bytes());
    let mut u = crate::sha256::hmac_sha256(pw, &msg);
    let mut dk = u;
    let mut i = 1u32;
    while i < PBKDF2_ITERS {
        u = crate::sha256::hmac_sha256(pw, &u);
        let mut j = 0usize;
        while j < 32 {
            dk[j] ^= u[j];
            j += 1;
        }
        i += 1;
    }
    dk
}

/// Consecutive failed-login counters, one per `PASSWD` account. Reset to 0 on a
/// successful login; once an account reaches `LOGIN_LOCKOUT` it is locked.
static mut LOGIN_FAILS: [u8; 2] = [0, 0];

/// Consecutive failures that lock an account, throttling online brute force.
const LOGIN_LOCKOUT: u8 = 3;

/// On-disk credential store: a root-owned, owner-only `/data/shadow` file — the
/// `/etc/shadow` analogue in the writable VFS tree (rooted at `/data`). Each
/// record is `name[8] | salt[16] | pw_hash[32] | uid[1]`. NOTE: the kernel-side
/// `vfs_*` API takes paths with the `/data` mount prefix already stripped (see
/// `vfs::resolve`), so the internal path is `/shadow`; userspace reaches the
/// same file as `/data/shadow` (sys_open strips the prefix).
const SHADOW_PATH: &[u8] = b"/shadow";
const REC_LEN: usize = 57;
/// VFS mode bits (vfs.rs): owner read + owner write, NO other access — so an
/// unprivileged process cannot read the credential store.
const SHADOW_MODE: u8 = 0b0011;

/// Serialize the compiled seed records to the on-disk record format.
fn shadow_seed_bytes() -> [u8; REC_LEN * 2] {
    let mut buf = [0u8; REC_LEN * 2];
    let mut i = 0usize;
    while i < PASSWD.len() {
        let off = i * REC_LEN;
        buf[off..off + 8].copy_from_slice(&PASSWD[i].name);
        buf[off + 8..off + 24].copy_from_slice(&PASSWD[i].salt);
        buf[off + 24..off + 56].copy_from_slice(&PASSWD[i].pw_hash);
        buf[off + 56] = PASSWD[i].uid;
        i += 1;
    }
    buf
}

/// Provision `/data/shadow` from the compiled seed as a root-owned, owner-only
/// credential store. MUST be called once at boot from a known-good VFS context
/// (e.g. alongside `dlclose_selftest`, NOT lazily from the login syscall path —
/// VFS create/write is unreliable from there). Idempotent: if the file already
/// exists (carried over on a persistent disk, possibly admin-modified) it is
/// left untouched, so the file — not the compiled seed — is the source of truth.
pub(crate) unsafe fn cred_store_provision() {
    if !crate::vfs::vfs_ready() {
        return;
    }
    if crate::vfs::vfs_lookup(SHADOW_PATH).is_some() {
        return;
    }
    let idx = match crate::vfs::vfs_open(SHADOW_PATH, true) {
        Some(i) => i,
        None => return,
    };
    let seed = shadow_seed_bytes();
    crate::vfs::vfs_write(idx, 0, &seed);
    crate::vfs::set_node_owner(idx, 0);
    crate::vfs::set_node_mode(idx, SHADOW_MODE);
}

/// Verify a username/password against the credential store; returns the account
/// uid on a match. Credentials are read from `/data/shadow` (the runtime source
/// of truth, provisioned at boot), falling back to the compiled seed only if the
/// store is unreadable. The matched record's salt derives
/// `PBKDF2-HMAC-SHA256(pw, salt, PBKDF2_ITERS)`, compared to the stored key, so
/// the cleartext is never held and each guess costs the full iterated KDF. After
/// `LOGIN_LOCKOUT` consecutive failures the account is locked and even the
/// correct password is refused (a successful login resets the counter).
pub(crate) unsafe fn login_verify(name: &[u8; 8], pw: &[u8]) -> Option<u8> {
    let mut store = [0u8; REC_LEN * 2];
    let read = match crate::vfs::vfs_lookup(SHADOW_PATH) {
        Some(idx) => crate::vfs::vfs_read(idx, 0, &mut store),
        None => 0,
    };
    let from_file = read == REC_LEN * PASSWD.len();
    let mut i = 0usize;
    while i < PASSWD.len() {
        let mut rec_name = [0u8; 8];
        let mut rec_salt = [0u8; 16];
        let mut rec_hash = [0u8; 32];
        let rec_uid;
        if from_file {
            let off = i * REC_LEN;
            rec_name.copy_from_slice(&store[off..off + 8]);
            rec_salt.copy_from_slice(&store[off + 8..off + 24]);
            rec_hash.copy_from_slice(&store[off + 24..off + 56]);
            rec_uid = store[off + 56];
        } else {
            rec_name = PASSWD[i].name;
            rec_salt = PASSWD[i].salt;
            rec_hash = PASSWD[i].pw_hash;
            rec_uid = PASSWD[i].uid;
        }
        if rec_name == *name {
            if LOGIN_FAILS[i] >= LOGIN_LOCKOUT {
                return None; // locked: refuse regardless of the password
            }
            if rec_hash == pbkdf2_sha256_dk32(pw, &rec_salt) {
                LOGIN_FAILS[i] = 0;
                return Some(rec_uid);
            }
            LOGIN_FAILS[i] += 1;
            return None;
        }
        i += 1;
    }
    None
}
