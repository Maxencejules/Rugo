#!/usr/bin/env python3
# Lamport one-time signature keygen + sign (full-os guide Part IV.10, public-key
# package signing). Deterministic from a fixed seed so the committed key/sig
# binaries are reproducible. This runs OFFLINE: it holds the private key (256
# preimage pairs) and emits ONLY the public key (their hashes) + one signature.
# The kernel embeds the public key and verifies -- it never sees the private
# key, so unlike the old symmetric HMAC scheme it cannot forge a signature.
#
# Layout (SHA-256, n = 32 bytes, 256 message-hash bits):
#   private key : sk[i][b]  = SHA256(seed || "sk" || i || b)        (256x2x32)
#   public  key : pk[i][b]  = SHA256(sk[i][b])                       (256x2x32 = 16384 B)
#   signature   : sig[i]    = sk[i][ bit i of SHA256(message) ]      (256x32   = 8192 B)
# Verify (kernel): for each bit i of SHA256(message), SHA256(sig[i]) == pk[i][bit].

import hashlib
import os
import struct
import sys

SEED = b"rugo-lamport-v1-fixed-seed-do-not-use-in-prod"
MESSAGE = b"rugo-pkg-sign-v1"  # must match LAMPORT_MSG in the kernel
N = 32
BITS = 256


def H(*parts):
    h = hashlib.sha256()
    for p in parts:
        h.update(p)
    return h.digest()


def sk_elem(i, b):
    return H(SEED, b"sk", struct.pack("<H", i), bytes([b]))


def main():
    out_dir = sys.argv[1] if len(sys.argv) > 1 else os.path.join(
        os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "kernel_rs", "src"
    )
    # Public key: 256 pairs of SHA256(sk).
    pub = bytearray()
    for i in range(BITS):
        for b in (0, 1):
            pub += H(sk_elem(i, b))
    assert len(pub) == BITS * 2 * N

    # Signature over MESSAGE: reveal the preimage selected by each message-hash bit.
    mh = hashlib.sha256(MESSAGE).digest()
    sig = bytearray()
    for i in range(BITS):
        bit = (mh[i // 8] >> (7 - (i % 8))) & 1
        sig += sk_elem(i, bit)
    assert len(sig) == BITS * N

    # Self-check the verify relation before writing anything.
    for i in range(BITS):
        bit = (mh[i // 8] >> (7 - (i % 8))) & 1
        assert H(bytes(sig[i * N:(i + 1) * N])) == bytes(pub[(i * 2 + bit) * N:(i * 2 + bit + 1) * N])

    pub_path = os.path.join(out_dir, "lamport_pub.bin")
    sig_path = os.path.join(out_dir, "lamport_sig.bin")
    with open(pub_path, "wb") as f:
        f.write(pub)
    with open(sig_path, "wb") as f:
        f.write(sig)
    print(f"wrote {pub_path} ({len(pub)} B) and {sig_path} ({len(sig)} B)")
    print(f"message = {MESSAGE!r}  sha256(pub) = {hashlib.sha256(pub).hexdigest()}")


if __name__ == "__main__":
    main()
